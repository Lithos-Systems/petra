// petra-designer/src/store/flowStore.ts
import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import {
  type Node,
  type Edge,
  type NodeChange,
  type EdgeChange,
  type Connection,
  applyNodeChanges,
  applyEdgeChanges,
  addEdge
} from '@xyflow/react'
import { validateConnection } from '@/utils/validation'
import { generateYaml } from '@/utils/yamlGenerator'
import type { BlockNodeData } from '@/types/nodes'
import toast from 'react-hot-toast'

interface FlowState {
  nodes: Node[]
  edges: Edge[]
  selectedNode: Node | null
  // Reduced history size to prevent memory leaks
  history: Array<{ nodes: Node[]; edges: Edge[] }>
  historyIndex: number
  // Performance optimizations
  updateQueued: boolean
  updateTimer: ReturnType<typeof setTimeout> | null
  
  onNodesChange: (changes: NodeChange[]) => void
  onEdgesChange: (changes: EdgeChange[]) => void
  onConnect: (conn: Connection) => void
  addNode: (type: string, position: { x: number; y: number }, data?: any) => void
  updateNode: (id: string, data: any) => void
  updateNodeData: (id: string, data: any) => void
  deleteNode: (id: string) => void
  deleteEdge: (id: string) => void
  deleteSelectedNode: () => void
  setSelectedNode: (node: Node | null) => void
  clearFlow: () => void
  loadFlow: (nodes: Node[], edges: Edge[]) => void
  undo: () => void
  redo: () => void
  canUndo: () => boolean
  canRedo: () => boolean
  exportToYAML: () => string
  validateLogic: () => { valid: boolean; nodeCount: number; connectionCount: number; errors: string[] }
}

// Block configurations matching PETRA backend
interface BlockConfig {
  inputs: string[]
  outputs: string[]
  symbol: string
  minInputs?: number
  maxInputs?: number
  defaultInputCount?: number
}

export const BLOCK_CONFIGS: Record<string, BlockConfig> = {
  // Logic blocks
  AND: { symbol: '&', inputs: ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'], outputs: ['out'], minInputs: 2, maxInputs: 8, defaultInputCount: 2 },
  OR: { symbol: '≥1', inputs: ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'], outputs: ['out'], minInputs: 2, maxInputs: 8, defaultInputCount: 2 },
  NOT: { symbol: '1', inputs: ['in'], outputs: ['out'] },
  XOR: { symbol: '=1', inputs: ['a', 'b'], outputs: ['out'] },
  
  // Comparison blocks
  GT: { symbol: '>', inputs: ['a', 'b'], outputs: ['out'] },
  LT: { symbol: '<', inputs: ['a', 'b'], outputs: ['out'] },
  GTE: { symbol: '≥', inputs: ['a', 'b'], outputs: ['out'] },
  LTE: { symbol: '≤', inputs: ['a', 'b'], outputs: ['out'] },
  EQ: { symbol: '=', inputs: ['a', 'b'], outputs: ['out'] },
  NEQ: { symbol: '≠', inputs: ['a', 'b'], outputs: ['out'] },
  
  // Math blocks
  ADD: { symbol: '+', inputs: ['a', 'b'], outputs: ['out'] },
  SUB: { symbol: '-', inputs: ['a', 'b'], outputs: ['out'] },
  MUL: { symbol: '×', inputs: ['a', 'b'], outputs: ['out'] },
  DIV: { symbol: '÷', inputs: ['a', 'b'], outputs: ['out'] },
  ABS: { symbol: '|x|', inputs: ['in'], outputs: ['out'] },
  SQRT: { symbol: '√', inputs: ['in'], outputs: ['out'] },
  
  // Timer blocks
  ON_DELAY: { symbol: 'TON', inputs: ['in', 'reset'], outputs: ['out', 'elapsed'] },
  OFF_DELAY: { symbol: 'TOF', inputs: ['in', 'reset'], outputs: ['out', 'elapsed'] },
  PULSE: { symbol: 'TP', inputs: ['trigger'], outputs: ['out'] },
  
  // Edge detection
  RISING_EDGE: { symbol: '↑', inputs: ['in'], outputs: ['out'] },
  FALLING_EDGE: { symbol: '↓', inputs: ['in'], outputs: ['out'] },
  
  // Counter blocks
  UP_COUNTER: { symbol: 'CTU', inputs: ['count_up', 'reset'], outputs: ['count', 'done'] },
  DOWN_COUNTER: { symbol: 'CTD', inputs: ['count_down', 'reset'], outputs: ['count', 'done'] },
  
  // Advanced blocks
  PID: { symbol: 'PID', inputs: ['setpoint', 'feedback', 'enable'], outputs: ['output'] },
  SELECT: { symbol: 'SEL', inputs: ['selector', 'a', 'b'], outputs: ['out'] },
  LIMIT: { symbol: 'LIM', inputs: ['min', 'in', 'max'], outputs: ['out'] },
  SCALE: { symbol: 'SCL', inputs: ['in'], outputs: ['out'] },
}

// Get input/output configuration for a block type
function getBlockInputsOutputs(blockType: string, inputCount?: number) {
  const config = BLOCK_CONFIGS[blockType]
  if (!config) {
    return { inputs: [], outputs: [] }
  }
  
  // For AND/OR gates, use dynamic input count
  if ((blockType === 'AND' || blockType === 'OR') && inputCount && config.maxInputs) {
    const actualCount = Math.min(inputCount, config.maxInputs)
    return {
      inputs: config.inputs.slice(0, actualCount).map(name => ({ name, type: 'bool' })),
      outputs: config.outputs.map(name => ({ name, type: 'bool' }))
    }
  }
  
  // Default behavior for other blocks
  return {
    inputs: config.inputs.map(name => ({ name, type: 'float' })),
    outputs: config.outputs.map(name => ({ name, type: 'float' }))
  }
}

// Get default parameters for block types
function getDefaultBlockParams(blockType: string) {
  switch (blockType) {
    case 'ON_DELAY':
    case 'OFF_DELAY':
    case 'PULSE':
      return { preset: 1.0, units: 'seconds' }
    case 'PID':
      return { 
        kp: 1.0, 
        ki: 0.1, 
        kd: 0.01, 
        output_min: 0.0, 
        output_max: 100.0 
      }
    case 'GT':
    case 'LT':
    case 'EQ':
    case 'GTE':
    case 'LTE':
    case 'NEQ':
      return { threshold: 0.0, deadband: 0.1 }
    default:
      return {}
  }
}

// Get default node data based on type
function getDefaultNodeData(type: string, customData?: any): any {
  // For block types with custom data
  if (type === 'block' && customData?.blockType) {
    const blockType = customData.blockType
    const config = BLOCK_CONFIGS[blockType]
    
    // Generate inputs based on block type
    let inputs: { name: string; type: string }[] = []
    if ((blockType === 'AND' || blockType === 'OR') && config?.maxInputs) {
      // Variable inputs for AND/OR
      const inputCount = customData.inputCount || config.defaultInputCount || 2
      for (let i = 0; i < inputCount; i++) {
        inputs.push({ name: String.fromCharCode(97 + i), type: 'bool' })
      }
    } else if (config?.inputs) {
      // Fixed inputs for other blocks
      inputs = config.inputs.map(name => ({ name, type: 'float' }))
    }
    
    const outputs = config?.outputs?.map(name => ({ name, type: 'float' })) || []
    const params = getDefaultBlockParams(blockType)
    
    return {
      label: customData.label || `${blockType}_${Date.now()}`,
      blockType,
      inputs,
      outputs,
      params,
      inputCount: customData.inputCount,
      status: 'stopped',
      ...customData
    } as BlockNodeData
  }
  
  // Default node data for other types
  switch (type) {
    case 'signal':
      return {
        label: customData?.label || 'New Signal',
        signalName: customData?.signalName || `signal_${Date.now()}`,
        signalType: customData?.signalType || 'float',
        initial: customData?.initial || 0,
        mode: customData?.mode || 'write',
        value: undefined,
        ...customData
      }
      
    case 'mqtt':
      return {
        label: customData?.label || `MQTT_${Date.now()}`,
        configured: false,
        brokerHost: customData?.brokerHost || 'localhost',
        brokerPort: customData?.brokerPort || 1883,
        clientId: customData?.clientId || `petra_${Date.now()}`,
        topicPrefix: customData?.topicPrefix || 'petra',
        mode: customData?.mode || 'read_write',
        publishOnChange: true,
        subscriptions: [],
        publications: [],
        ...customData
      }
      
    case 's7':
      return {
        label: customData?.label || `S7_${Date.now()}`,
        configured: false,
        ip: '192.168.0.1',
        rack: 0,
        slot: 2,
        area: 'DB',
        dbNumber: 1,
        address: 0,
        dataType: 'real',
        direction: 'read',
        signal: '',
        ...customData
      }
      
    case 'modbus':
      return {
        label: customData?.label || `Modbus_${Date.now()}`,
        configured: false,
        host: '192.168.0.1',
        port: 502,
        unitId: 1,
        address: 0,
        dataType: 'holding_register',
        direction: 'read',
        signal: '',
        ...customData
      }
      
    default:
      return {
        label: customData?.label || `${type}_${Date.now()}`,
        ...customData
      }
  }
}

// Debounced history save to prevent excessive updates
function saveHistoryDebounced(get: () => FlowState, set: (s: Partial<FlowState>) => void) {
  const state = get()
  
  // Clear existing timer
  if (state.updateTimer) {
    clearTimeout(state.updateTimer)
  }
  
  // Set new timer
  const timer = setTimeout(() => {
    const currentState = get()
    const snap = {
      nodes: currentState.nodes.map(n => ({ ...n })),
      edges: currentState.edges.map(e => ({ ...e }))
    }
    
    const newHistory = currentState.history.slice(0, currentState.historyIndex + 1)
    newHistory.push(snap)
    
    // Keep only last 20 history items to prevent memory issues
    if (newHistory.length > 20) {
      newHistory.splice(0, newHistory.length - 20)
    }
    
    set({ 
      history: newHistory, 
      historyIndex: newHistory.length - 1,
      updateQueued: false,
      updateTimer: null
    })
  }, 300) // 300ms debounce
  
  set({ updateQueued: true, updateTimer: timer })
}

const useFlowStoreInternal = create<FlowState>()(
  subscribeWithSelector((set, get) => ({
    nodes: [],
    edges: [],
    selectedNode: null,
    history: [{ nodes: [], edges: [] }],
    historyIndex: 0,
    updateQueued: false,
    updateTimer: null,

    onNodesChange: (changes) => {
      // Filter out select changes to reduce updates
      const filteredChanges = changes.filter(change => 
        change.type !== 'select' || (change.type === 'select' && change.selected)
      )
      
      if (filteredChanges.length > 0) {
        set(state => {
          const newNodes = applyNodeChanges(filteredChanges, state.nodes)
          return { nodes: newNodes }
        })
        
        // Only save history for position changes
        const hasPositionChange = filteredChanges.some(c => c.type === 'position')
        if (hasPositionChange) {
          saveHistoryDebounced(get, set)
        }
      }
    },

    onEdgesChange: (changes) => {
      set(state => ({ 
        edges: applyEdgeChanges(changes, state.edges) 
      }))
    },

    onConnect: (connection) => {
      if (!connection.source || !connection.target) return
      
      const validation = validateConnection(
        connection,
        get().nodes,
        get().edges
      )
      
      if (!validation.valid) {
        toast.error(validation.error || 'Invalid connection')
        return
      }
      
      const newEdge: Edge = {
        ...connection,
        id: `${connection.source}_${connection.sourceHandle}-${connection.target}_${connection.targetHandle}`,
        type: 'default',
        animated: false,
      } as Edge
      
      set(state => ({
        edges: addEdge(newEdge, state.edges)
      }))
      
      saveHistoryDebounced(get, set)
    },

    addNode: (type: string, position: { x: number; y: number }, data: any = {}) => {
      const nodeId = `${type}_${Date.now()}`
      const nodeData = getDefaultNodeData(type, data)
      
      const newNode: Node = {
        id: nodeId,
        type,
        position,
        data: {
          ...nodeData,
          id: nodeId,
        },
      }
      
      set(state => ({
        nodes: [...state.nodes, newNode]
      }))
      
      saveHistoryDebounced(get, set)
    },

    updateNode: (nodeId: string, data: any) => {
      get().updateNodeData(nodeId, data)
    },

    updateNodeData: (nodeId: string, updates: any) => {
      set(state => {
        const nodes = state.nodes.map((node) => {
          if (node.id === nodeId) {
            // Handle block type changes
            if (updates.blockType && updates.blockType !== node.data.blockType) {
              const inputCount = updates.inputCount || (node.data as BlockNodeData).inputCount || 2
              const { inputs, outputs } = getBlockInputsOutputs(updates.blockType, inputCount)
              return {
                ...node,
                data: { 
                  ...node.data, 
                  ...updates, 
                  inputs, 
                  outputs,
                  params: getDefaultBlockParams(updates.blockType)
                },
              }
            }
            
            // Handle input count changes for AND/OR gates
            if (updates.inputCount && (node.data.blockType === 'AND' || node.data.blockType === 'OR')) {
              const { inputs, outputs } = getBlockInputsOutputs(node.data.blockType, updates.inputCount)
              return {
                ...node,
                data: {
                  ...node.data,
                  ...updates,
                  inputs,
                  outputs,
                },
              }
            }
            
            // Normal update
            return { 
              ...node, 
              data: { ...node.data, ...updates } 
            }
          }
          return node
        })
        
        // Update selected node if it was the one updated
        const selectedNode = state.selectedNode
        if (selectedNode?.id === nodeId) {
          const updatedNode = nodes.find(n => n.id === nodeId)
          return { nodes, selectedNode: updatedNode || null }
        }
        
        return { nodes }
      })
      
      saveHistoryDebounced(get, set)
    },

    deleteNode: (nodeId: string) => {
      set(state => ({
        nodes: state.nodes.filter((n) => n.id !== nodeId),
        edges: state.edges.filter((e) => e.source !== nodeId && e.target !== nodeId),
        selectedNode: state.selectedNode?.id === nodeId ? null : state.selectedNode,
      }))
      saveHistoryDebounced(get, set)
    },

    deleteEdge: (edgeId: string) => {
      set(state => ({
        edges: state.edges.filter(e => e.id !== edgeId),
      }))
      saveHistoryDebounced(get, set)
    },

    deleteSelectedNode: () => {
      const selectedNode = get().selectedNode
      if (selectedNode) {
        get().deleteNode(selectedNode.id)
      }
    },

    setSelectedNode: (node: Node | null) => {
      set({ selectedNode: node })
    },

    clearFlow: () => {
      // Clear any pending history saves
      const state = get()
      if (state.updateTimer) {
        clearTimeout(state.updateTimer)
      }
      
      set({
        nodes: [],
        edges: [],
        selectedNode: null,
        history: [{ nodes: [], edges: [] }],
        historyIndex: 0,
        updateQueued: false,
        updateTimer: null
      })
    },

    loadFlow: (nodes: Node[], edges: Edge[]) => {
      // Clear any pending history saves
      const state = get()
      if (state.updateTimer) {
        clearTimeout(state.updateTimer)
      }
      
      set({
        nodes,
        edges,
        selectedNode: null,
        history: [{ nodes: [...nodes], edges: [...edges] }],
        historyIndex: 0,
        updateQueued: false,
        updateTimer: null
      })
    },

    undo: () => {
      const state = get()
      if (state.historyIndex > 0) {
        const newIndex = state.historyIndex - 1
        const snap = state.history[newIndex]
        set({
          nodes: snap.nodes.map(n => ({ ...n })),
          edges: snap.edges.map(e => ({ ...e })),
          historyIndex: newIndex,
          selectedNode: null,
        })
      }
    },

    redo: () => {
      const state = get()
      if (state.historyIndex < state.history.length - 1) {
        const newIndex = state.historyIndex + 1
        const snap = state.history[newIndex]
        set({
          nodes: snap.nodes.map(n => ({ ...n })),
          edges: snap.edges.map(e => ({ ...e })),
          historyIndex: newIndex,
          selectedNode: null,
        })
      }
    },

    canUndo: () => get().historyIndex > 0,
    canRedo: () => get().historyIndex < get().history.length - 1,

    exportToYAML: () => {
      const { nodes, edges } = get()
      return generateYaml(nodes, edges)
    },

    validateLogic: () => {
      const { nodes, edges } = get()
      const errors: string[] = []
      
      // Basic validation
      if (nodes.length === 0) {
        errors.push('No blocks in design')
      }
      
      // Check for unconnected required inputs
      nodes.forEach(node => {
        if (node.type === 'block') {
          const blockData = node.data as BlockNodeData
          const blockType = blockData.blockType
          const config = BLOCK_CONFIGS[blockType as keyof typeof BLOCK_CONFIGS]
          
          if (config) {
            const requiredInputs = (blockType === 'AND' || blockType === 'OR') 
              ? Math.min(blockData.inputCount || 2, config.inputs.length)
              : config.inputs.length
              
            for (let i = 0; i < requiredInputs; i++) {
              const inputName = config.inputs[i]
              const hasConnection = edges.some(edge => 
                edge.target === node.id && edge.targetHandle === inputName
              )
              if (!hasConnection) {
                errors.push(`${blockData.label}: Input '${inputName}' not connected`)
              }
            }
          }
        }
      })
      
      return {
        valid: errors.length === 0,
        nodeCount: nodes.length,
        connectionCount: edges.length,
        errors
      }
    },
  }))
)

// Export with compatible aliases
export const useFlowStore = useFlowStoreInternal
export const useOptimizedFlowStore = useFlowStoreInternal
