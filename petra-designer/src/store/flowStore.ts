// src/store/flowStore.ts
import { create } from 'zustand'
import {
  Node,
  Edge,
  Connection,
  applyNodeChanges,
  applyEdgeChanges,
  OnNodesChange,
  OnEdgesChange,
  NodeChange,
  EdgeChange,
} from '@xyflow/react'
import { nanoid } from 'nanoid'
import { validateConnection } from '@/utils/validation'
import toast from 'react-hot-toast'

interface FlowState {
  nodes: Node[]
  edges: Edge[]
  selectedNode: Node | null
  onNodesChange: OnNodesChange
  onEdgesChange: OnEdgesChange
  onConnect: (connection: Connection) => void
  addNode: (type: string, position: { x: number; y: number }) => void
  updateNode: (nodeId: string, data: any) => void
  updateNodeData: (nodeId: string, data: any) => void
  deleteNode: (nodeId: string) => void
  deleteEdge: (edgeId: string) => void
  setSelectedNode: (node: Node | null) => void
  clearFlow: () => void
  loadFlow: (nodes: Node[], edges: Edge[]) => void
  // History for undo/redo
  history: Array<{ nodes: Node[]; edges: Edge[] }>
  historyIndex: number
  undo: () => void
  redo: () => void
  canUndo: () => boolean
  canRedo: () => boolean
  // Additional methods
  deleteSelectedNode: () => void
  exportToYAML: () => string
  validateLogic: () => { valid: boolean; nodeCount: number; connectionCount: number; errors: string[] }
}

type FlowStore = FlowState

// Helper to save current nodes/edges to history
const saveHistory = (get: () => FlowStore, set: (state: Partial<FlowStore>) => void) => {
  const state = get()
  const snapshot = {
    nodes: JSON.parse(JSON.stringify(state.nodes)),
    edges: JSON.parse(JSON.stringify(state.edges)),
  }
  const newHistory = state.history.slice(0, state.historyIndex + 1)
  newHistory.push(snapshot)
  if (newHistory.length > 50) newHistory.shift()
  set({ history: newHistory, historyIndex: newHistory.length - 1 })
}

// Get default inputs/outputs for block types
function getBlockInputsOutputs(blockType: string) {
  switch (blockType) {
    case 'and':
    case 'or':
    case 'xor':
      return {
        inputs: [
          { name: 'A', type: 'bool' },
          { name: 'B', type: 'bool' }
        ],
        outputs: [
          { name: 'out', type: 'bool' }
        ]
      }
    case 'not':
      return {
        inputs: [
          { name: 'in', type: 'bool' }
        ],
        outputs: [
          { name: 'out', type: 'bool' }
        ]
      }
    case 'greater_than':
    case 'less_than':
    case 'equal':
      return {
        inputs: [
          { name: 'A', type: 'float' },
          { name: 'B', type: 'float' }
        ],
        outputs: [
          { name: 'out', type: 'bool' }
        ]
      }
    case 'add':
    case 'subtract':
    case 'multiply':
    case 'divide':
      return {
        inputs: [
          { name: 'A', type: 'float' },
          { name: 'B', type: 'float' }
        ],
        outputs: [
          { name: 'out', type: 'float' }
        ]
      }
    case 'timer':
    case 'delay':
      return {
        inputs: [
          { name: 'enable', type: 'bool' },
          { name: 'preset', type: 'float' }
        ],
        outputs: [
          { name: 'done', type: 'bool' },
          { name: 'elapsed', type: 'float' }
        ]
      }
    case 'pid':
      return {
        inputs: [
          { name: 'setpoint', type: 'float' },
          { name: 'process_var', type: 'float' },
          { name: 'enable', type: 'bool' }
        ],
        outputs: [
          { name: 'output', type: 'float' }
        ]
      }
    case 'controller':
      return {
        inputs: [
          { name: 'input', type: 'float' },
          { name: 'enable', type: 'bool' }
        ],
        outputs: [
          { name: 'output', type: 'float' }
        ]
      }
    case 'input':
    case 'analog_input':
    case 'digital_input':
      return {
        inputs: [],
        outputs: [
          { name: 'value', type: blockType.includes('digital') ? 'bool' : 'float' }
        ]
      }
    case 'output':
      return {
        inputs: [
          { name: 'value', type: 'float' }
        ],
        outputs: []
      }
    case 'modbus':
    case 's7':
    case 'mqtt':
      return {
        inputs: [
          { name: 'enable', type: 'bool' },
          { name: 'value', type: 'float' }
        ],
        outputs: [
          { name: 'data', type: 'float' },
          { name: 'status', type: 'bool' }
        ]
      }
    case 'signal':
      return {
        inputs: [
          { name: 'in', type: 'float' }
        ],
        outputs: [
          { name: 'out', type: 'float' }
        ]
      }
    default:
      return {
        inputs: [
          { name: 'in', type: 'float' }
        ],
        outputs: [
          { name: 'out', type: 'float' }
        ]
      }
  }
}

// Get default parameters for block types
function getDefaultBlockParams(blockType: string) {
  switch (blockType) {
    case 'timer':
    case 'delay':
      return { preset: 1.0, units: 'seconds' }
    case 'pid':
      return { 
        kp: 1.0, 
        ki: 0.1, 
        kd: 0.01, 
        output_min: 0.0, 
        output_max: 100.0 
      }
    case 'greater_than':
    case 'less_than':
    case 'equal':
      return { threshold: 0.0, deadband: 0.1 }
    default:
      return {}
  }
}

function getDefaultNodeData(type: string): any {
  // Handle legacy type mappings - map sidebar types to node types
  const nodeTypeMap: Record<string, string> = {
    // Logic blocks -> block node type
    'and': 'block',
    'or': 'block', 
    'not': 'block',
    'xor': 'block',
    'greater_than': 'block',
    'less_than': 'block',
    'equal': 'block',
    'add': 'block',
    'subtract': 'block',
    'multiply': 'block',
    'divide': 'block',
    'timer': 'block',
    'delay': 'block',
    'pid': 'block',
    'controller': 'block',
    'input': 'block',
    'output': 'block',
    'analog_input': 'block',
    'digital_input': 'block',
    // Communication blocks -> their specific node types
    'modbus': 'modbus',
    's7': 's7',
    'mqtt': 'mqtt', 
    'twilio': 'twilio',
    // Signals -> signal node type
    'signal': 'signal'
  }

  const actualNodeType = nodeTypeMap[type] || type
  const blockType = actualNodeType === 'block' ? type : undefined

  switch (actualNodeType) {
    case 'signal':
      return {
        label: 'New Signal',
        signalType: 'float',
        initial: 0,
        mode: 'write',
      }
    case 'block':
      const { inputs, outputs } = getBlockInputsOutputs(blockType!)
      const params = getDefaultBlockParams(blockType!)
      return {
        label: `New ${blockType?.toUpperCase()}`,
        blockType: blockType!,
        inputs,
        outputs,
        params,
      }
    case 'twilio':
      return {
        label: 'New Twilio',
        configured: false,
        actionType: 'sms',
        toNumber: '+1234567890',
        content: 'Alert from PETRA',
      }
    case 'mqtt':
      return {
        label: 'New MQTT',
        configured: false,
        brokerHost: 'localhost',
        brokerPort: 1883,
        clientId: 'petra_client',
        topicPrefix: 'petra',
        mode: 'read_write',
        publishOnChange: true,
        signalName: 'sensor_data',
        signalType: 'float',
      }
    case 's7':
      return {
        label: 'New S7',
        configured: false,
        ip: '192.168.1.100',
        rack: 0,
        slot: 1,
        area: 'DB',
        dbNumber: 1,
        address: 0,
        dataType: 'real',
        direction: 'read',
        signal: 'plc_data',
      }
    case 'modbus':
      return {
        label: 'New Modbus',
        configured: false,
        host: 'localhost',
        port: 502,
        unitId: 1,
        address: 0,
        dataType: 'holding_register',
        direction: 'read',
        signal: 'modbus_data',
      }
    default:
      return {
        label: 'New Node',
      }
  }
}

export const useFlowStore = create<FlowState>((set, get) => ({
  nodes: [],
  edges: [],
  selectedNode: null,
  history: [{ nodes: [], edges: [] }],
  historyIndex: 0,
  
  deleteEdge: (edgeId: string) => {
    set({
      edges: get().edges.filter((e) => e.id !== edgeId),
    })
    saveHistory(get, set)
  },

  onNodesChange: (changes: NodeChange[]) => {
    set({
      nodes: applyNodeChanges(changes, get().nodes),
    })

    // Update selected node if it was changed
    const selectedId = get().selectedNode?.id
    if (selectedId) {
      const updatedNode = get().nodes.find(n => n.id === selectedId)
      if (updatedNode) {
        set({ selectedNode: updatedNode })
      }
    }
  },

  onEdgesChange: (changes: EdgeChange[]) => {
    set({
      edges: applyEdgeChanges(changes, get().edges),
    })
  },

  onConnect: (connection: Connection) => {
    const { nodes, edges } = get()

    const validation = validateConnection(connection, nodes, edges)
    if (!validation.valid) {
      toast.error(validation.error || 'Invalid connection')
      return
    }

    const newEdge: Edge = {
      id: nanoid(),
      source: connection.source!,
      target: connection.target!,
      sourceHandle: connection.sourceHandle,
      targetHandle: connection.targetHandle,
      type: 'default',
      animated: true,
    }

    set({
      edges: [...edges, newEdge],
    })

    saveHistory(get, set)

    toast.success('Connected successfully')
  },

  addNode: (type: string, position: { x: number; y: number }) => {
    const nodeId = nanoid()
    const nodeData = getDefaultNodeData(type)

    // Map sidebar types to actual node types
    const nodeTypeMap: Record<string, string> = {
      'and': 'block',
      'or': 'block', 
      'not': 'block',
      'xor': 'block',
      'greater_than': 'block',
      'less_than': 'block',
      'equal': 'block',
      'add': 'block',
      'subtract': 'block',
      'multiply': 'block',
      'divide': 'block',
      'timer': 'block',
      'delay': 'block',
      'pid': 'block',
      'controller': 'block',
      'input': 'block',
      'output': 'block',
      'analog_input': 'block',
      'digital_input': 'block',
      'modbus': 'block',
      's7': 'block',
      'mqtt': 'block',
      'signal': 'signal'
    }

    const actualNodeType = nodeTypeMap[type] || type

    const newNode: Node = {
      id: nodeId,
      type: actualNodeType,
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

  // Deprecated - use updateNodeData instead
  updateNode: (nodeId: string, data: any) => {
    get().updateNodeData(nodeId, data)
  },

  // New method with better handling
  updateNodeData: (nodeId: string, updates: any) => {
    const nodes = get().nodes.map((node) => {
      if (node.id === nodeId) {
        // Handle block type changes
        if (updates.blockType && updates.blockType !== node.data.blockType) {
          const { inputs, outputs } = getBlockInputsOutputs(updates.blockType)
          return {
            ...node,
            data: { 
              ...node.data, 
              ...updates, 
              inputs, 
              outputs,
              // Reset params when block type changes
              params: getDefaultBlockParams(updates.blockType)
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
    // Simplified YAML export
    const { nodes, edges } = get()
    return `# PETRA Configuration
signals: []
blocks: []
scan_time_ms: 100
# Generated from ${nodes.length} nodes and ${edges.length} connections`
  },

  validateLogic: () => {
    const { nodes, edges } = get()
    const errors: string[] = []
    
    // Basic validation
    if (nodes.length === 0) {
      errors.push('No blocks in design')
    }
    
    return {
      valid: errors.length === 0,
      nodeCount: nodes.length,
      connectionCount: edges.length,
      errors
    }
  },
}))
