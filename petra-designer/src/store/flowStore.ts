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
  addEdge,
} from '@xyflow/react'
import { validateConnection } from '@/utils/validation'
import { generateYaml } from '@/utils/yamlGenerator'
import toast from 'react-hot-toast'
import type { 
  BlockNodeData, 
  SignalNodeData, 
  MqttNodeData,
  S7NodeData,
  ModbusNodeData,
  ProtocolNodeData 
} from '@/types/nodes'

interface FlowState {
  nodes: Node[]
  edges: Edge[]
  selectedNode: Node | null
  history: Array<{ nodes: Node[]; edges: Edge[] }>
  historyIndex: number
  
  // Node operations
  onNodesChange: (changes: NodeChange[]) => void
  onEdgesChange: (changes: EdgeChange[]) => void
  onConnect: (connection: Connection) => void
  addNode: (type: string, position: { x: number; y: number }, data?: any) => void
  updateNode: (nodeId: string, data: any) => void
  updateNodeData: (nodeId: string, updates: any) => void
  deleteNode: (nodeId: string) => void
  deleteSelectedNode: () => void
  setSelectedNode: (node: Node | null) => void
  
  // Flow operations
  clearFlow: () => void
  loadFlow: (nodes: Node[], edges: Edge[]) => void
  exportToYAML: () => string
  validateLogic: () => { valid: boolean; nodeCount: number; connectionCount: number; errors: string[] }
  
  // History
  undo: () => void
  redo: () => void
  canUndo: () => boolean
  canRedo: () => boolean
}

// Enhanced block configurations matching PETRA backend
const BLOCK_CONFIGS = {
  // Logic blocks
  'AND': { 
    inputs: ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'], 
    outputs: ['out'], 
    symbol: '&',
    minInputs: 2,
    maxInputs: 8,
    defaultInputCount: 2
  },
  'OR': { 
    inputs: ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'], 
    outputs: ['out'], 
    symbol: '≥1',
    minInputs: 2,
    maxInputs: 8,
    defaultInputCount: 2
  },
  'NOT': { inputs: ['in'], outputs: ['out'], symbol: '1' },
  'XOR': { inputs: ['a', 'b'], outputs: ['out'], symbol: '=1' },
  
  // Comparison blocks
  'GT': { inputs: ['a', 'b'], outputs: ['out'], symbol: '>' },
  'LT': { inputs: ['a', 'b'], outputs: ['out'], symbol: '<' },
  'EQ': { inputs: ['a', 'b'], outputs: ['out'], symbol: '=' },
  'GTE': { inputs: ['a', 'b'], outputs: ['out'], symbol: '≥' },
  'LTE': { inputs: ['a', 'b'], outputs: ['out'], symbol: '≤' },
  'NEQ': { inputs: ['a', 'b'], outputs: ['out'], symbol: '≠' },
  
  // Math blocks
  'ADD': { inputs: ['a', 'b'], outputs: ['out'], symbol: '+' },
  'SUB': { inputs: ['a', 'b'], outputs: ['out'], symbol: '-' },
  'MUL': { inputs: ['a', 'b'], outputs: ['out'], symbol: '×' },
  'DIV': { inputs: ['a', 'b'], outputs: ['out'], symbol: '÷' },
  
  // Timer blocks
  'ON_DELAY': { inputs: ['in', 'pt'], outputs: ['out', 'et'], symbol: 'TON' },
  'OFF_DELAY': { inputs: ['in', 'pt'], outputs: ['out', 'et'], symbol: 'TOF' },
  'PULSE': { inputs: ['in', 'pt'], outputs: ['out', 'et'], symbol: 'TP' },
  
  // Control blocks
  'PID': { 
    inputs: ['pv', 'sp', 'man', 'reset'], 
    outputs: ['out', 'err'], 
    symbol: 'PID' 
  },
}

// Helper to get block inputs/outputs
function getBlockInputsOutputs(blockType: string, inputCount?: number) {
  const config = BLOCK_CONFIGS[blockType as keyof typeof BLOCK_CONFIGS]
  if (!config) {
    return { inputs: [], outputs: [] }
  }
  
  // For AND/OR gates, use dynamic input count
  if ((blockType === 'AND' || blockType === 'OR') && inputCount) {
    const actualCount = Math.min(inputCount, config.maxInputs || 8)
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
  switch (type) {
    case 'signal':
      return {
        label: 'New Signal',
        signalName: `signal_${Date.now()}`,
        signalType: 'float',
        initial: 0,
        mode: 'write',
        ...customData
      } as SignalNodeData
      
    case 'block':
      const blockType = customData?.blockType || 'AND'
      const config = BLOCK_CONFIGS[blockType as keyof typeof BLOCK_CONFIGS]
      const inputCount = customData?.inputCount || config?.defaultInputCount || 2
      const { inputs, outputs } = getBlockInputsOutputs(blockType, inputCount)
      const params = getDefaultBlockParams(blockType)
      
      return {
        label: customData?.label || `${blockType}_${Date.now()}`,
        blockType,
        inputs,
        outputs,
        params,
        inputCount,
        status: 'stopped',
        ...customData
      } as BlockNodeData
      
    case 'mqtt':
      return {
        label: customData?.label || `MQTT_${Date.now()}`,
        configured: false,
        brokerHost: 'localhost',
        brokerPort: 1883,
        clientId: `petra_${Date.now()}`,
        topicPrefix: 'petra',
        mode: 'read_write',
        publishOnChange: true,
        ...customData
      } as MqttNodeData
      
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
      } as S7NodeData
      
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
      } as ModbusNodeData
      
    case 'protocol':
      const protocolType = customData?.protocolType || 'MQTT'
      return {
        label: customData?.label || `${protocolType}_${Date.now()}`,
        protocolType,
        configured: false,
        config: {
          broker: protocolType === 'MQTT' ? 'localhost' : undefined,
          port: protocolType === 'MQTT' ? 1883 : undefined,
          topic: protocolType === 'MQTT' ? 'petra/+' : undefined,
          direction: 'subscribe',
          ...customData?.config
        },
        ...customData
      } as ProtocolNodeData
      
    default:
      return {
        label: `${type}_${Date.now()}`,
        ...customData
      }
  }
}

// Save state to history
function saveHistory(get: () => FlowState, set: (state: Partial<FlowState>) => void) {
  const state = get()
  const snap = {
    nodes: JSON.parse(JSON.stringify(state.nodes)),
    edges: JSON.parse(JSON.stringify(state.edges))
  }
  
  // Trim history after current index
  const newHistory = state.history.slice(0, state.historyIndex + 1)
  newHistory.push(snap)
  
  // Keep only last 50 states
  if (newHistory.length > 50) {
    newHistory.shift()
  }
  
  set({ 
    history: newHistory, 
    historyIndex: newHistory.length - 1 
  })
}

// Create the flow store
export const useFlowStore = create<FlowState>()(
  subscribeWithSelector((set, get) => ({
    nodes: [],
    edges: [],
    selectedNode: null,
    history: [{ nodes: [], edges: [] }],
    historyIndex: 0,
    
    onNodesChange: (changes) => {
      set({
        nodes: applyNodeChanges(changes, get().nodes),
      })
    },
    
    onEdgesChange: (changes) => {
      set({
        edges: applyEdgeChanges(changes, get().edges),
      })
    },
    
    onConnect: (connection) => {
      const validation = validateConnection(connection, get().nodes, get().edges)
      
      if (!validation.valid) {
        toast.error(validation.error || 'Invalid connection')
        return
      }
      
      const newEdge = {
        ...connection,
        type: 'default', // Use bezier curves
        animated: false,
        style: { strokeWidth: 2 }
      }
      
      set({
        edges: addEdge(newEdge, get().edges),
      })
      
      saveHistory(get, set)
      toast.success('Connected')
    },
    
    addNode: (type: string, position: { x: number; y: number }, data?: any) => {
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
      
      set({
        nodes: [...get().nodes, newNode],
      })
      
      saveHistory(get, set)
    },
    
    updateNode: (nodeId: string, data: any) => {
      get().updateNodeData(nodeId, data)
    },
    
    updateNodeData: (nodeId: string, updates: any) => {
      const nodes = get().nodes.map((node) => {
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
      
      set({ nodes })
      saveHistory(get, set)
      
      // Update selected node if it was the one updated
      if (get().selectedNode?.id === nodeId) {
        const updatedNode = nodes.find(n => n.id === nodeId)
        if (updatedNode) {
          set({ selectedNode: updatedNode })
        }
      }
    },
    
    deleteNode: (nodeId: string) => {
      set({
        nodes: get().nodes.filter((n) => n.id !== nodeId),
        edges: get().edges.filter((e) => e.source !== nodeId && e.target !== nodeId),
        selectedNode: get().selectedNode?.id === nodeId ? null : get().selectedNode,
      })
      saveHistory(get, set)
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
      set({
        nodes: [],
        edges: [],
        selectedNode: null,
      })
      saveHistory(get, set)
    },
    
    loadFlow: (nodes: Node[], edges: Edge[]) => {
      set({
        nodes,
        edges,
        selectedNode: null,
      })
      saveHistory(get, set)
    },
    
    undo: () => {
      const state = get()
      if (state.historyIndex > 0) {
        const newIndex = state.historyIndex - 1
        const snap = state.history[newIndex]
        set({
          nodes: JSON.parse(JSON.stringify(snap.nodes)),
          edges: JSON.parse(JSON.stringify(snap.edges)),
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
          nodes: JSON.parse(JSON.stringify(snap.nodes)),
          edges: JSON.parse(JSON.stringify(snap.edges)),
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

// Export compatibility with optimizedFlowStore
export const useOptimizedFlowStore = useFlowStore
