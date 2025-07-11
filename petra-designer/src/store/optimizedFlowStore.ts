import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import {
  type Node,
  type Edge,
  type NodeChange,
  type EdgeChange,
  type Connection,
  applyNodeChanges,
  applyEdgeChanges
} from '@xyflow/react'
import { nanoid } from 'nanoid'
import { validateConnection } from '@/utils/validation'
import toast from 'react-hot-toast'

interface FlowState {
  nodes: Node[]
  edges: Edge[]
  selectedNode: Node | null
  history: Array<{ nodes: Node[]; edges: Edge[] }>
  historyIndex: number
  onNodesChange: (changes: NodeChange[]) => void
  onEdgesChange: (changes: EdgeChange[]) => void
  onConnect: (conn: Connection) => void
  addNode: (type: string, position: { x: number; y: number }) => void
  updateNode: (id: string, data: any) => void
  updateNodeData: (id: string, data: any) => void
  deleteNode: (id: string) => void
  deleteEdge: (id: string) => void
  deleteSelectedNode: () => void
  setSelectedNode: (node: Node | null) => void
  clearFlow: () => void
  loadFlow: (nodes: Node[], edges: Edge[]) => void
  exportToYAML: () => string
  validateLogic: () => { valid: boolean; nodeCount: number; connectionCount: number; errors: string[] }
}

function saveHistory(get: () => FlowState, set: (s: Partial<FlowState>) => void) {
  const state = get()
  const snap = {
    nodes: JSON.parse(JSON.stringify(state.nodes)),
    edges: JSON.parse(JSON.stringify(state.edges))
  }
  const newHistory = state.history.slice(0, state.historyIndex + 1)
  newHistory.push(snap)
  if (newHistory.length > 20) newHistory.shift()
  set({ history: newHistory, historyIndex: newHistory.length - 1 })
}

export const useOptimizedFlowStore = create<FlowState>()(
  subscribeWithSelector((set, get) => ({
    nodes: [],
    edges: [],
    selectedNode: null,
    history: [{ nodes: [], edges: [] }],
    historyIndex: 0,

    onNodesChange: (changes) => {
      set(state => ({ nodes: applyNodeChanges(changes, state.nodes) }))
    },

    onEdgesChange: (changes) => {
      set(state => ({ edges: applyEdgeChanges(changes, state.edges) }))
    },

    onConnect: (connection) => {
      const { nodes, edges } = get()
      const result = validateConnection(connection, nodes, edges)
      if (!result.valid) {
        toast.error(result.error || 'Invalid connection')
        return
      }
      const newEdge: Edge = {
        id: nanoid(),
        source: connection.source!,
        target: connection.target!,
        sourceHandle: connection.sourceHandle,
        targetHandle: connection.targetHandle
      }
      set(state => ({ edges: [...state.edges, newEdge] }))
      saveHistory(get, set)
      toast.success('Connected successfully')
    },

    addNode: (type, position) => {
      const node: Node = {
        id: nanoid(),
        type,
        position,
        data: { label: type }
      }
      set(state => ({ nodes: [...state.nodes, node] }))
      saveHistory(get, set)
    },

    updateNode: (id, data) => {
      set(state => ({
        nodes: state.nodes.map(n => n.id === id ? { ...n, data: { ...n.data, ...data } } : n)
      }))
      saveHistory(get, set)
    },

    updateNodeData: (id, data) => {
      get().updateNode(id, data)
    },

    deleteEdge: (id) => {
      set(state => ({ edges: state.edges.filter(e => e.id !== id) }))
      saveHistory(get, set)
    },

    deleteNode: (id) => {
      set(state => ({
        nodes: state.nodes.filter(n => n.id !== id),
        edges: state.edges.filter(e => e.source !== id && e.target !== id),
        selectedNode: state.selectedNode?.id === id ? null : state.selectedNode
      }))
      saveHistory(get, set)
    },

    deleteSelectedNode: () => {
      const node = get().selectedNode
      if (node) get().deleteNode(node.id)
    },

    setSelectedNode: (node) => set({ selectedNode: node }),

    clearFlow: () => {
      set({ nodes: [], edges: [], selectedNode: null })
      saveHistory(get, set)
    },

    loadFlow: (nodes, edges) => {
      set({ nodes, edges, selectedNode: null })
      saveHistory(get, set)
    },

    exportToYAML: () => {
      const { nodes, edges } = get()
      return `# PETRA Configuration\nsignals: []\nblocks: []\nscan_time_ms: 100\n# Generated from ${nodes.length} nodes and ${edges.length} connections`
    },

    validateLogic: () => {
      const { nodes, edges } = get()
      const errors: string[] = []
      if (nodes.length === 0) errors.push('No blocks in design')
      return {
        valid: errors.length === 0,
        nodeCount: nodes.length,
        connectionCount: edges.length,
        errors
      }
    }
  }))
)
