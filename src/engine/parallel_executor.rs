use crate::{Block, Result, SignalBus, PlcError};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct BlockDependencyGraph {
    /// Map of block name to blocks that depend on it
    dependents: HashMap<String, Vec<String>>,
    /// Map of block name to blocks it depends on
    dependencies: HashMap<String, Vec<String>>,
    /// Blocks that can run in parallel
    parallel_groups: Vec<Vec<String>>,
}

impl BlockDependencyGraph {
    pub fn analyze(blocks: &[Box<dyn Block + Send + Sync>]) -> Self {
        let mut graph = Self {
            dependents: HashMap::new(),
            dependencies: HashMap::new(),
            parallel_groups: Vec::new(),
        };
        
        // Build dependency maps
        let mut output_to_block: HashMap<String, String> = HashMap::new();
        
        for block in blocks {
            let block_name = block.name().to_string();
            
            // Map outputs to this block
            for output in block.output_signals() {
                output_to_block.insert(output.to_string(), block_name.clone());
            }
            
            graph.dependencies.insert(block_name.clone(), Vec::new());
            graph.dependents.insert(block_name.clone(), Vec::new());
        }
        
        // Build dependency relationships
        for block in blocks {
            let block_name = block.name().to_string();
            
            for input in block.input_dependencies() {
                if let Some(producer) = output_to_block.get(input) {
                    // This block depends on the producer
                    graph.dependencies.get_mut(&block_name).unwrap().push(producer.clone());
                    // The producer has this block as a dependent
                    graph.dependents.get_mut(producer).unwrap().push(block_name.clone());
                }
            }
        }
        
        // Calculate parallel execution groups using topological sort
        graph.calculate_parallel_groups(blocks);
        
        graph
    }
    
    fn calculate_parallel_groups(&mut self, blocks: &[Box<dyn Block + Send + Sync>]) {
        let mut visited = HashSet::new();
        let mut groups = Vec::new();
        let mut current_level = Vec::new();
        
        // Find blocks with no dependencies
        for block in blocks {
            let name = block.name().to_string();
            if self.dependencies[&name].is_empty() && block.is_parallelizable() {
                current_level.push(name);
            }
        }
        
        // Process levels
        while !current_level.is_empty() {
            groups.push(current_level.clone());
            
            for name in &current_level {
                visited.insert(name.clone());
            }
            
            let mut next_level = Vec::new();
            
            // Find blocks whose dependencies are all satisfied
            for block in blocks {
                let name = block.name().to_string();
                if !visited.contains(&name) && block.is_parallelizable() {
                    let deps = &self.dependencies[&name];
                    if deps.iter().all(|dep| visited.contains(dep)) {
                        next_level.push(name);
                    }
                }
            }
            
            current_level = next_level;
        }
        
        // Add non-parallelizable blocks as individual groups
        for block in blocks {
            let name = block.name().to_string();
            if !block.is_parallelizable() || !visited.contains(&name) {
                groups.push(vec![name]);
            }
        }
        
        self.parallel_groups = groups;
        debug!("Calculated {} parallel execution groups", groups.len());
    }
}

pub struct ParallelExecutor {
    dependency_graph: BlockDependencyGraph,
    #[cfg(feature = "thread-pool")]
    thread_pool: Option<Arc<tokio::runtime::Runtime>>,
}

impl ParallelExecutor {
    pub fn new(blocks: &[Box<dyn Block + Send + Sync>]) -> Self {
        let dependency_graph = BlockDependencyGraph::analyze(blocks);
        
        #[cfg(feature = "thread-pool")]
        let thread_pool = Some(Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(num_cpus::get())
                .thread_name("petra-block-executor")
                .enable_all()
                .build()
                .unwrap()
        ));
        
        Self {
            dependency_graph,
            #[cfg(feature = "thread-pool")]
            thread_pool,
        }
    }
    
    pub async fn execute_parallel(
        &self,
        blocks: Arc<Mutex<Vec<Box<dyn Block + Send + Sync>>>>,
        bus: &SignalBus,
    ) -> Result<()> {
        let blocks_guard = blocks.lock().await;
        let block_map: HashMap<String, usize> = blocks_guard
            .iter()
            .enumerate()
            .map(|(i, b)| (b.name().to_string(), i))
            .collect();
        drop(blocks_guard);
        
        for group in &self.dependency_graph.parallel_groups {
            if group.len() == 1 {
                // Execute single block directly
                let mut blocks_guard = blocks.lock().await;
                if let Some(&idx) = block_map.get(&group[0]) {
                    if let Err(e) = blocks_guard[idx].execute(bus) {
                        warn!("Block '{}' execution failed: {}", group[0], e);
                    }
                }
            } else {
                // Execute group in parallel
                let mut join_set = JoinSet::new();
                
                for block_name in group {
                    if let Some(&idx) = block_map.get(block_name) {
                        let blocks_clone = Arc::clone(&blocks);
                        let bus_clone = bus.clone();
                        let block_name_clone = block_name.clone();
                        
                        join_set.spawn(async move {
                            let mut blocks_guard = blocks_clone.lock().await;
                            if let Err(e) = blocks_guard[idx].execute(&bus_clone) {
                                warn!("Block '{}' execution failed: {}", block_name_clone, e);
                            }
                        });
                    }
                }
                
                // Wait for all blocks in group to complete
                while let Some(result) = join_set.join_next().await {
                    if let Err(e) = result {
                        warn!("Parallel block execution task failed: {}", e);
                    }
                }
            }
        }
        
        Ok(())
    }
}

