import { useCallback, DragEvent, useEffect, MouseEvent } from 'react'
import {
  ReactFlow,
  Background,
  BackgroundVariant,
  Controls,
  MiniMap,
  ReactFlowProvider,
  Node,
  Edge,
} from '@xyflow/react'
import { Toaster } from 'react-hot-toast'

import { useFlowStore } from './store/flowStore'
import { nodeTypes } from './nodes'
import Sidebar from './components/Sidebar'
import PropertiesPanel from './components/PropertiesPanel'
import YamlPreview from './components/YamlPreview'
import Toolbar from './components/Toolbar'
import { ErrorBoundary } from './components/ErrorBoundary'

import { PetraNode } from '@/types/nodes'          // ⬅️ the union we created
/* top of the file, after other imports */
import ReactFlow, { ReactFlowProps } from '@xyflow/react'
import type { PetraNode } from '@/types/nodes'

type PetraEdge = Edge              // keep your custom edge alias if needed
const RF = ReactFlow as unknown as <N = PetraNode, E = PetraEdge>(
  props: ReactFlowProps<N, E>,
) => JSX.Element   // “RF” is now a JSX-friendly component with generics

// Edge data is still the default – adjust if you create a custom type later
type PetraEdge = Edge

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

  /* ---------- debug log ---------- */
  useEffect(() => {
    console.log(
      'ReactFlow mounted, nodes:',
      nodes.length,
      'edges:',
      edges.length,
    )
  }, [nodes, edges])

  /* ---------- drag-&-drop ---------- */
  const onDragOver = useCallback((event: DragEvent) => {
    event.preventDefault()
    event.dataTransfer.dropEffect = 'move'
  }, [])

  const onDrop = useCallback(
    (event: DragEvent) => {
      event.preventDefault()

      const type = event.dataTransfer.getData('application/reactflow')
      if (!type) return

      const bounds = event.currentTarget.getBoundingClientRect()
      const position = {
        x: event.clientX - bounds.left - 100,
        y: event.clientY - bounds.top - 20,
      }

      addNode(type, position)
    },
    [addNode],
  )

  /* ---------- node click ---------- */
  const onNodeClick = useCallback(
    (_e: MouseEvent, node: PetraNode) => {
      setSelectedNode(node)
    },
    [setSelectedNode],
  )

  return (
    <div className="h-screen flex flex-col bg-gray-50">
      <Toaster position="top-right" />
      <Toolbar />

      <div className="flex-1 flex">
        <Sidebar />

        <div className="flex-1 relative">
          <RF<PetraNode, PetraEdge>       {/* ✅ typed generics */}
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            onConnect={onConnect}
            onNodeClick={onNodeClick}
            onDrop={onDrop}
            onDragOver={onDragOver}
            nodeTypes={nodeTypes}
            fitView
            className="bg-gray-50"
          >
            {/* ✅ enum instead of "dots" string */}
            <Background
              variant={BackgroundVariant.Dots}
              gap={20}
              className="bg-gray-50"
            />
            <Controls />
            <MiniMap className="bg-white" />
          </ReactFlow>
        </div>

        <div className="flex">
          {selectedNode && <PropertiesPanel />}
          <YamlPreview />
        </div>
      </div>
    </div>
  )
}

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
