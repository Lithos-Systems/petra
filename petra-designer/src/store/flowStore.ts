// petra-designer/src/store/optimizedFlowStore.ts
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
import { nanoid } from 'nanoid'
import { validateConnection } from '@/utils/validation'
import { generateYaml } from '@/utils/yamlGenerator'
import { 
  getDefaultNodeData, 
  getBlockInputsOutputs, 
  getDefaultBlockParams,
  BLOCK_CONFIGS 
} from '@/utils/nodeUtils'
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
  updateTimer: NodeJS.Timeout | null
  
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
      
      const newEdge = {
        ...connection,
        id: `${connection.source}_${connection.sourceHandle}-${connection.target}_${connection.targetHandle}`,
        type: 'default',
        animated: false,
      }
      
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
