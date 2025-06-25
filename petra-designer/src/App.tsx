import { useCallback, useEffect, DragEvent, MouseEvent } from 'react'
import {
  ReactFlow,
  Background,
  BackgroundVariant,
  Controls,
  MiniMap,
  ReactFlowProvider,
  type Node,
  type Edge,
} from '@xyflow/react'
import { Toaster } from 'react-hot-toast'

import { useFlowStore } from './store/flowStore'
import { nodeTypes } from './nodes'
import Sidebar from './components/Sidebar'
import PropertiesPanel from './components/PropertiesPanel'
import YamlPreview from './components/YamlPreview'
import Toolbar from './components/Toolbar'
import { ErrorBoundary } from './components/ErrorBoundary'

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

  useEffect(() => {
    console.log('ReactFlow mounted. nodes:', nodes.length, 'edges:', edges.length)
  }, [nodes, edges])

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

  const onNodeClick = useCallback(
    (_evt: MouseEvent, node: Node) => setSelectedNode(node),
    [setSelectedNode],
  )

  return (
    <div className="h-screen flex flex-col bg-gray-50">
      <Toaster position="top-right" />
      <Toolbar />

      <div className="flex-1 flex">
        <Sidebar />

        <div className="flex-1 relative">
          <ReactFlow
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
