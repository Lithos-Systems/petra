import { useCallback, useEffect, DragEvent, MouseEvent } from 'react'
import {
  ReactFlow,
  Background,
  BackgroundVariant,
  Controls,
  MiniMap,
  ReactFlowProvider,
  Edge,
  type ReactFlowProps,
} from '@xyflow/react'
import { Toaster } from 'react-hot-toast'

import { useFlowStore } from './store/flowStore'
import { nodeTypes } from './nodes'
import Sidebar from './components/Sidebar'
import PropertiesPanel from './components/PropertiesPanel'
import YamlPreview from './components/YamlPreview'
import Toolbar from './components/Toolbar'
import { ErrorBoundary } from './components/ErrorBoundary'

import type { PetraNode } from '@/types/nodes'

/* ------------------------------------------------------------------ */
/* Typed wrapper so JSX can infer generics without <ReactFlow<…>>      */
/* ------------------------------------------------------------------ */
type PetraEdge = Edge
const RF = ReactFlow as unknown as (
  props: ReactFlowProps<PetraNode, PetraEdge>,
) => JSX.Element

/* ------------------------------------------------------------------ */
/* Flow component                                                     */
/* ------------------------------------------------------------------ */
function Flow() {
  const {
    nodes,
    edges,
    onNodesChange,
    onEdgesChange,
    onConnect,
    selectedNode,
    addNode,
    setSelectedNode,
  } = useFlowStore()

  /* Debug log on initial mount / updates */
  useEffect(() => {
    console.log('ReactFlow mounted. nodes:', nodes.length, 'edges:', edges.length)
  }, [nodes, edges])

  /* Drag-and-drop handlers */
  const onDragOver = useCallback((e: DragEvent) => {
    e.preventDefault()
    e.dataTransfer.dropEffect = 'move'
  }, [])

  const onDrop = useCallback(
    (e: DragEvent) => {
      e.preventDefault()
      const type = e.dataTransfer.getData('application/reactflow')
      if (!type) return

      const bounds = e.currentTarget.getBoundingClientRect()
      addNode(type, {
        x: e.clientX - bounds.left - 100,
        y: e.clientY - bounds.top - 20,
      })
    },
    [addNode],
  )

  /* Node click selects the node in the store */
  const onNodeClick = useCallback(
    (_evt: MouseEvent, node: PetraNode) => setSelectedNode(node),
    [setSelectedNode],
  )

  return (
    <div className="h-screen flex flex-col bg-gray-50">
      <Toaster position="top-right" />
      <Toolbar />

      <div className="flex-1 flex">
        <Sidebar />

        <div className="flex-1 relative">
          <RF
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            onConnect={onConnect}
            onNodeClick={onNodeClick}
            onDragOver={onDragOver}
            onDrop={onDrop}
            nodeTypes={nodeTypes}
            fitView
            className="bg-gray-50"
          >
            <Background
              variant={BackgroundVariant.Dots}
              gap={20}
              className="bg-gray-50"
            />
            <Controls />
            <MiniMap className="bg-white" />
          </RF>
        </div>

        <div className="flex">
          {selectedNode && <PropertiesPanel />}
          <YamlPreview />
        </div>
      </div>
    </div>
  )
}

/* ------------------------------------------------------------------ */
/* App root with providers & error boundary                           */
/* ------------------------------------------------------------------ */
function App() {
  return (
    <ErrorBoundary>
      <ReactFlowProvider>
        <Flow />
      </ReactFlowProvider>
    </ErrorBoundary>
  )
}

export default App
