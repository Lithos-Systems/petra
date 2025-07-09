import { useCallback, useEffect, DragEvent, MouseEvent, useState } from 'react'
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
import toast from 'react-hot-toast'
import { useFlowStore } from './store/flowStore'
import { nodeTypes } from './nodes'
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts'
import Sidebar from './components/Sidebar'
import PropertiesPanel from './components/PropertiesPanel'
import YamlPreview from './components/YamlPreview'
import Toolbar from './components/Toolbar'
import { ErrorBoundary } from './components/ErrorBoundary'
import HMIDesigner from './components/hmi/HMIDesigner'
import ISA101WaterPlantDemo from './components/hmi/ISA101WaterPlantDemo'
import { FaProjectDiagram, FaDesktop } from 'react-icons/fa'
import { PetraProvider } from './contexts/PetraContext'
import ConnectionStatusSidebar from './components/ConnectionStatusSidebar'
import { usePetraConnection } from './hooks/usePetraConnection'
import './styles/isa101-theme.css'

type DesignerMode = 'logic' | 'graphics'

function Flow() {
  const [mode, setMode] = useState<DesignerMode>('logic')
  const [isISA101Mode, setIsISA101Mode] = useState(true)
  const [connectionSidebarOpen, setConnectionSidebarOpen] = useState(false)
  
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

  // PETRA connection
  const {
    connected,
    connectionState,
    signals,
    performance,
    lastError,
  } = usePetraConnection({
    onConnect: () => {
      toast.success('Connected to PETRA')
    },
    onDisconnect: () => {
      toast.error('Disconnected from PETRA')
    },
  })

  // Connection info for sidebar
  const connectionInfo = {
    status: connectionState,
    latency: performance.latency,
    uptime: Math.floor((Date.now() - (performance.connectedAt || Date.now())) / 1000),
    lastError: lastError,
    messageRate: performance.messageRate,
    reconnectAttempts: performance.reconnectAttempts || 0
  }

  // Add keyboard shortcuts
  useKeyboardShortcuts()

  // Apply ISA-101 mode class to body
  useEffect(() => {
    if (isISA101Mode) {
      document.body.classList.add('isa101-mode')
    } else {
      document.body.classList.remove('isa101-mode')
    }
  }, [isISA101Mode])

  useEffect(() => {
    console.log('ReactFlow mounted. nodes:', nodes.length, 'edges:', edges.length)
  }, [nodes, edges])

  const onEdgeClick = useCallback(
    (event: MouseEvent, edge: any) => {
      event.stopPropagation()
      deleteEdge(edge.id)
      toast.success('Connection deleted')
    },
    [deleteEdge]
  )

  const deletePressed = useKeyPress(['Delete', 'Backspace'])
  useEffect(() => {
    if (deletePressed && selectedNode) {
      // Only delete nodes when a node is selected
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
    <div className={`h-screen flex flex-col ${isISA101Mode ? 'isa101-mode bg-[#D3D3D3]' : 'bg-gray-50'}`}>
      <Toaster 
        position="top-right"
        toastOptions={{
          duration: 3000,
          style: {
            background: isISA101Mode ? '#404040' : '#363636',
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
      
      {/* Enhanced Toolbar with mode switcher */}
      <div className={isISA101Mode ? "isa101-toolbar" : "bg-white border-b border-gray-200 shadow-sm"}>
        <div className="flex items-center justify-between px-4 py-2">
          {/* Left: Logo/Title and Toolbar */}
          <div className="flex items-center gap-6">
            <h1 className={`text-xl font-bold ${isISA101Mode ? 'text-black' : 'text-gray-800'}`}>
              PETRA Designer
            </h1>
            <div className={`h-6 w-px ${isISA101Mode ? 'bg-[#606060]' : 'bg-gray-300'}`} />
            <Toolbar />
          </div>
          
          {/* Center: Mode Switcher */}
          <div className={`flex items-center ${isISA101Mode ? 'bg-[#B8B8B8]' : 'bg-gray-100'} rounded-lg p-1`}>
            <button
              onClick={() => setMode('logic')}
              className={`flex items-center gap-2 px-4 py-2 rounded transition-colors ${
                mode === 'logic' 
                  ? isISA101Mode ? 'bg-[#E0E0E0] text-black shadow-sm' : 'bg-white text-petra-600 shadow-sm'
                  : isISA101Mode ? 'text-[#404040] hover:text-black' : 'text-gray-600 hover:text-gray-800'
              }`}
            >
              <FaProjectDiagram className="w-4 h-4" />
              <span className="text-sm font-medium">Logic Designer</span>
            </button>
            <button
              onClick={() => setMode('graphics')}
              className={`flex items-center gap-2 px-4 py-2 rounded transition-colors ${
                mode === 'graphics' 
                  ? isISA101Mode ? 'bg-[#E0E0E0] text-black shadow-sm' : 'bg-white text-petra-600 shadow-sm'
                  : isISA101Mode ? 'text-[#404040] hover:text-black' : 'text-gray-600 hover:text-gray-800'
              }`}
            >
              <FaDesktop className="w-4 h-4" />
              <span className="text-sm font-medium">HMI Graphics</span>
            </button>
          </div>

          {/* Right: ISA-101 Toggle and Connection Status */}
          <div className="flex items-center gap-2">
            <button
              onClick={() => setIsISA101Mode(!isISA101Mode)}
              className={isISA101Mode ? "isa101-button text-xs px-3 py-1" : "px-3 py-1 text-xs border rounded hover:bg-gray-100"}
            >
              {isISA101Mode ? 'ISA-101' : 'Modern'} Theme
            </button>
            <button
              onClick={() => setConnectionSidebarOpen(!connectionSidebarOpen)}
              className={isISA101Mode ? "isa101-button text-xs px-3 py-1" : "px-3 py-1 text-xs border rounded hover:bg-gray-100"}
            >
              <div className="flex items-center gap-2">
                <div className={`w-2 h-2 rounded-full ${connected ? 'bg-green-500' : 'bg-red-500'}`} />
                {connected ? 'Connected' : 'Disconnected'}
              </div>
            </button>
          </div>
        </div>
      </div>

      {/* Conditional rendering based on mode */}
      {mode === 'logic' ? (
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
              className={isISA101Mode ? "bg-[#D3D3D3]" : "bg-gray-50"}
              defaultEdgeOptions={{
                animated: !isISA101Mode, // No animation in ISA-101 mode
                style: { 
                  strokeWidth: 2,
                  stroke: isISA101Mode ? '#000000' : undefined
                },
              }}
            >
              <Background
                variant={BackgroundVariant.Dots}
                gap={20}
                size={1}
                className={isISA101Mode ? "bg-[#D3D3D3]" : "bg-gray-50"}
                color={isISA101Mode ? "#A0A0A0" : undefined}
              />
              <Controls 
                style={isISA101Mode ? {
                  backgroundColor: '#E0E0E0',
                  border: '1px solid #606060',
                  borderRadius: 0
                } : undefined}
              />
              <MiniMap 
                className={isISA101Mode ? "bg-[#E0E0E0] border border-[#606060] rounded-none" : "bg-white"}
                nodeStrokeColor={(n) => {
                  if (isISA101Mode) return '#404040'
                  if (n.type === 'signal') return '#3b82f6'
                  if (n.type === 'block') return '#10b981'
                  if (n.type === 'twilio') return '#9333ea'
                  if (n.type === 'mqtt') return '#f97316'
                  if (n.type === 's7') return '#ef4444'
                  return '#6b7280'
                }}
                nodeColor={(n) => {
                  if (isISA101Mode) return '#E0E0E0'
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
            <div className={`absolute bottom-4 left-4 p-3 rounded-lg shadow-md text-xs ${
              isISA101Mode ? 'isa101-panel text-black' : 'bg-white text-gray-600'
            }`}>
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
      ) : (
        <HMIDesigner />
      )}

      {/* Connection Status Sidebar */}
      <ConnectionStatusSidebar
        connectionInfo={connectionInfo}
        isOpen={connectionSidebarOpen}
        onToggle={() => setConnectionSidebarOpen(!connectionSidebarOpen)}
      />
    </div>
  )
}

function App() {
  // Check if we're loading a specific demo
  const isWaterPlantDemo = window.location.hash === '#/water-plant-demo'
  
  if (isWaterPlantDemo) {
    return (
      <ErrorBoundary>
        <PetraProvider>
          <ISA101WaterPlantDemo />
        </PetraProvider>
      </ErrorBoundary>
    )
  }
  
  return (
    <ErrorBoundary>
      <PetraProvider>
        <ReactFlowProvider>
          <Flow />
        </ReactFlowProvider>
      </PetraProvider>
    </ErrorBoundary>
  )
}

export default App
