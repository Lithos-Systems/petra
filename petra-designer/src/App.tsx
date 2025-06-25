import { useCallback, DragEvent, useEffect } from 'react'
import { ReactFlow, Background, Controls, MiniMap, Panel, ReactFlowProvider } from '@xyflow/react'
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
  // Add this inside the Flow component to test if ReactFlow renders
  useEffect(() => {
    console.log('ReactFlow mounted, nodes:', nodes.length, 'edges:', edges.length)
  }, [nodes, edges])

  const onDragOver = useCallback((event: DragEvent) => {
    event.preventDefault()
    event.dataTransfer.dropEffect = 'move'
  }, [])

  const onDrop = useCallback(
    (event: DragEvent) => {
      event.preventDefault()
  
      const type = event.dataTransfer.getData('application/reactflow')
      if (!type) return
  
      const reactFlowBounds = event.currentTarget.getBoundingClientRect()
      const position = {
        x: event.clientX - reactFlowBounds.left - 100, // Center the node
        y: event.clientY - reactFlowBounds.top - 20,
      }
  
      addNode(type, position)
    },
    [addNode]
  )

  const onNodeClick = useCallback(
    (_: any, node: any) => {
      setSelectedNode(node)
    },
    [setSelectedNode]
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
            onDrop={onDrop}
            onDragOver={onDragOver}
            nodeTypes={nodeTypes}
            fitView
            className="bg-gray-50"
          >
            <Background variant="dots" gap={20} className="bg-gray-50" />
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
