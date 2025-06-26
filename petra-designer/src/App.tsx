// Add this to your App.tsx in the Flow component
import { useCallback, useEffect, DragEvent, MouseEvent } from 'react'
import {
  ReactFlow,
  Background,
  BackgroundVariant,
  Controls,
  MiniMap,
  ReactFlowProvider,
  type Node,
  useKeyPress,
} from '@xyflow/react'

import { Toaster } from 'react-hot-toast'
import { useFlowStore } from './store/flowStore'
import { nodeTypes } from './nodes'
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts'
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
    deleteEdge, 
  } = useFlowStore()

  // Add keyboard shortcuts
  useKeyboardShortcuts()

  useEffect(() => {
    console.log('ReactFlow mounted. nodes:', nodes.length, 'edges:', edges.length)
  }, [nodes, edges])
  const onEdgeClick = useCallback(
    (event: MouseEvent, edge: any) => {
      event.stopPropagation() // Prevent node selection
      deleteEdge(edge.id)
      toast.success('Connection deleted')
    },
    [deleteEdge]
  )
  const deletePressed = useKeyPress(['Delete', 'Backspace'])
  useEffect(() => {
    if (deletePressed && selectedNode) {
      // Only delete nodes when a node is selected
      // Edges are deleted by clicking on them
    }
  }, [deletePressed, selectedNode])
  const onDragOver = useCallback((e: DragEvent) => {
    e.preventDefault()
    e.dataTransfer.dropEffect = 'move'
  }, [])

  const onDrop = useCallback(
    (e: DragEvent) => {
      e.preventDefault()
      const type = e.dataTransfer.getData('application/reactflow')
      if (!type) return

      const reactFlowBounds = e.currentTarget.getBoundingClientRect()
      const position = {
        x: e.clientX - reactFlowBounds.left - 75,
        y: e.clientY - reactFlowBounds.top - 25,
      }

      addNode(type, position)
    },
    [addNode],
  )

  const onNodeClick = useCallback(
    (_evt: MouseEvent, node: Node) => {
      setSelectedNode(node)
    },
    [setSelectedNode],
  )

  const onPaneClick = useCallback(() => {
    setSelectedNode(null)
  }, [setSelectedNode])

  return (
    <div className="h-screen flex flex-col bg-gray-50">
      <Toaster 
        position="top-right"
        toastOptions={{
          duration: 3000,
          style: {
            background: '#363636',
            color: '#fff',
          },
          success: {
            iconTheme: {
              primary: '#10b981',
              secondary: '#fff',
            },
          },
          error: {
            iconTheme: {
              primary: '#ef4444',
              secondary: '#fff',
            },
          },
        }}
      />
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
            onPaneClick={onPaneClick}
            onEdgeClick={onEdgeClick} 
            onDragOver={onDragOver}
            onDrop={onDrop}
            nodeTypes={nodeTypes}
            deleteKeyCode={null}
            fitView
            fitViewOptions={{ padding: 0.2 }}
            className="bg-gray-50"
            defaultEdgeOptions={{
              animated: true,
              style: { strokeWidth: 2 },
            }}
          >
            <Background
              variant={BackgroundVariant.Dots}
              gap={20}
              size={1}
              className="bg-gray-50"
            />
            <Controls />
            <MiniMap 
              className="bg-white"
              nodeStrokeColor={(n) => {
                if (n.type === 'signal') return '#3b82f6'
                if (n.type === 'block') return '#10b981'
                if (n.type === 'twilio') return '#9333ea'
                if (n.type === 'mqtt') return '#f97316'
                if (n.type === 's7') return '#ef4444'
                return '#6b7280'
              }}
              nodeColor={(n) => {
                if (n.type === 'signal') return '#dbeafe'
                if (n.type === 'block') return '#d1fae5'
                if (n.type === 'twilio') return '#f3e8ff'
                if (n.type === 'mqtt') return '#fed7aa'
                if (n.type === 's7') return '#fee2e2'
                return '#f3f4f6'
              }}
            />
          </ReactFlow>

          {/* Keyboard shortcuts help */}
          <div className="absolute bottom-4 left-4 bg-white p-3 rounded-lg shadow-md text-xs text-gray-600">
            <div className="font-semibold mb-1">Keyboard Shortcuts:</div>
            <div>Delete/Backspace - Delete selected</div>
            <div>Ctrl/Cmd + S - Save flow</div>
            <div>Ctrl/Cmd + Shift + D - Clear all</div>
          </div>
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
