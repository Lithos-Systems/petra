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
  deleteEdge: (edgeId: string) => void  // Add this line
  setSelectedNode: (node: Node | null) => void
  clearFlow: () => void
  loadFlow: (nodes: Node[], edges: Edge[]) => void
}

export const useFlowStore = create<FlowState>((set, get) => ({
  nodes: [],
  edges: [],
  selectedNode: null,
  
  deleteEdge: (edgeId: string) => {
    set({
      edges: get().edges.filter((e) => e.id !== edgeId),
    })
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

    toast.success('Connected successfully')
  },

  addNode: (type: string, position: { x: number; y: number }) => {
    const nodeId = nanoid()
    const nodeData = getDefaultNodeData(type)

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
  },

  loadFlow: (nodes: Node[], edges: Edge[]) => {
    set({
      nodes,
      edges,
      selectedNode: null,
    })
  },
}))

function getDefaultNodeData(type: string): any {
  switch (type) {
    case 'signal':
      return {
        label: 'New Signal',
        signalType: 'float',
        initial: 0,
        mode: 'write',
      }
    case 'block':
      const blockType = 'AND'
      return {
        label: 'New Block',
        blockType,
        ...getBlockInputsOutputs(blockType),
        params: getDefaultBlockParams(blockType),
      }
    case 'twilio':
      return {
        label: 'Twilio Action',
        actionType: 'sms',
        toNumber: '',
        content: '',
        configured: false,
      }
    case 'mqtt':
      return {
        label: 'MQTT Config',
        brokerHost: 'mqtt.lithos.systems',
        brokerPort: 1883,
        clientId: 'petra-01',
        topicPrefix: 'petra/plc',
        configured: false,
      }
    case 's7':
      return {
        label: 'S7 Mapping',
        signal: '',
        area: 'DB',
        dbNumber: 0,
        address: 0,
        dataType: 'bool',
        direction: 'read',
        configured: false,
      }
    default:
      return { label: 'New Node' }
  }
}

function getBlockInputsOutputs(blockType: string) {
  switch (blockType) {
    case 'AND':
    case 'OR':
      return {
        inputs: [
          { name: 'in1', type: 'bool' },
          { name: 'in2', type: 'bool' },
        ],
        outputs: [{ name: 'out', type: 'bool' }],
      }
    case 'NOT':
      return {
        inputs: [{ name: 'in', type: 'bool' }],
        outputs: [{ name: 'out', type: 'bool' }],
      }
    case 'GT':
    case 'LT':
    case 'EQ':
      return {
        inputs: [
          { name: 'in1', type: 'float' },
          { name: 'in2', type: 'float' },
        ],
        outputs: [{ name: 'out', type: 'bool' }],
      }
    case 'TON':
    case 'TOF':
      return {
        inputs: [{ name: 'in', type: 'bool' }],
        outputs: [{ name: 'q', type: 'bool' }],
      }
    case 'R_TRIG':
    case 'F_TRIG':
      return {
        inputs: [{ name: 'clk', type: 'bool' }],
        outputs: [{ name: 'q', type: 'bool' }],
      }
    case 'SR_LATCH':
      return {
        inputs: [
          { name: 'set', type: 'bool' },
          { name: 'reset', type: 'bool' },
        ],
        outputs: [{ name: 'q', type: 'bool' }],
      }
    case 'COUNTER':
      return {
        inputs: [{ name: 'enable', type: 'bool' }],
        outputs: [{ name: 'count', type: 'int' }],
      }
    case 'MULTIPLY':
    case 'DIVIDE':
      return {
        inputs: [
          { name: 'in1', type: 'float' },
          { name: 'in2', type: 'float' },
        ],
        outputs: [{ name: 'out', type: 'float' }],
      }
    case 'DATA_GENERATOR':
      return {
        inputs: [{ name: 'enable', type: 'bool' }],
        outputs: [
          { name: 'sine_out', type: 'float' },
          { name: 'count_out', type: 'int' },
        ],
      }
    default:
      return {
        inputs: [{ name: 'in', type: 'float' }],
        outputs: [{ name: 'out', type: 'float' }],
      }
  }
}

function getDefaultBlockParams(blockType: string): Record<string, any> {
  switch (blockType) {
    case 'TON':
    case 'TOF':
      return { preset_ms: 1000 }
    case 'COUNTER':
      return { increment: 1 }
    case 'DATA_GENERATOR':
      return { frequency: 1.0, amplitude: 10.0 }
    default:
      return {}
  }
}
