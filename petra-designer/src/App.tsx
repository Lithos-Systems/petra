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
import ISA101Sidebar from './components/ISA101Sidebar'
import PropertiesPanel from './components/PropertiesPanel'
import YamlPreview from './components/YamlPreview'
import ISA101Toolbar from './components/ISA101Toolbar'
import { ErrorBoundary } from './components/ErrorBoundary'
import HMIDesigner from './components/hmi/HMIDesigner'
import ISA101WaterPlantDemo from './components/hmi/ISA101WaterPlantDemo'
import { FaProjectDiagram, FaDesktop } from 'react-icons/fa'
import { PetraProvider } from './contexts/PetraContext'
import { usePetraConnection } from './hooks/usePetraConnection'
import type { ConnectionInfo } from './types/hmi'
import './styles/isa101-theme.css'

type DesignerMode = 'logic' | 'graphics'

function Flow() {
  const [mode, setMode] = useState<DesignerMode>('logic')
  
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

  // Add keyboard shortcuts
  useKeyboardShortcuts()

  // Apply ISA-101 mode class to body
  useEffect(() => {
    document.body.classList.add('isa101-mode')
  }, [])

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
    <div className="h-screen flex flex-col isa101-mode bg-[#D3D3D3]">
      <Toaster 
        position="top-right"
        toastOptions={{
          duration: 3000,
          style: {
            background: '#404040',
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
      
      {/* Minimal ISA-101 Toolbar */}
      <div className="isa101-toolbar">
        <div className="flex items-center justify-between px-4 py-2">
          {/* Left: Logo/Title */}
          <div className="flex items-center gap-4">
            <h1 className="text-lg font-medium text-black">
              PETRA Designer
            </h1>
            <div className="h-4 w-px bg-[#606060]" />
            <ISA101Toolbar />
          </div>
          
          {/* Center: Mode Switcher - Minimal ISA-101 Style */}
          <div className="flex items-center bg-[#B8B8B8] p-1">
            <button
              onClick={() => setMode('logic')}
              className={`flex items-center gap-2 px-3 py-1 transition-colors ${
                mode === 'logic' 
                  ? 'bg-[#E0E0E0] text-black' 
                  : 'text-[#404040] hover:text-black'
              }`}
            >
              <FaProjectDiagram className="w-3 h-3" />
              <span className="text-sm">Logic</span>
            </button>
            <button
              onClick={() => setMode('graphics')}
              className={`flex items-center gap-2 px-3 py-1 transition-colors ${
                mode === 'graphics' 
                  ? 'bg-[#E0E0E0] text-black' 
                  : 'text-[#404040] hover:text-black'
              }`}
            >
              <FaDesktop className="w-3 h-3" />
              <span className="text-sm">Graphics</span>
            </button>
          </div>

          {/* Right: Connection Status - Simple Indicator */}
          <div className="isa101-connection-status">
            <div className={`isa101-connection-indicator ${connected ? 'connected' : ''}`} />
            <span className="text-xs">{connected ? 'ONLINE' : 'OFFLINE'}</span>
          </div>
        </div>
      </div>

      {/* Conditional rendering based on mode */}
      {mode === 'logic' ? (
        <div className="flex flex-1">
          {/* Sidebar */}
          <ISA101Sidebar />
          
          {/* Main Flow Area */}
          <div className="flex-1 flex flex-col">
            <ReactFlow
              nodes={nodes}
              edges={edges}
              onNodesChange={onNodesChange}
              onEdgesChange={onEdgesChange}
              onConnect={onConnect}
              onDrop={onDrop}
              onDragOver={onDragOver}
              onNodeClick={onNodeClick}
              onPaneClick={onPaneClick}
              onEdgeClick={onEdgeClick}
              nodeTypes={nodeTypes}
              fitView
              className="bg-[#D3D3D3]"
            >
              <Background 
                variant={BackgroundVariant.Lines} 
                gap={20} 
                size={1} 
                color="#B0B0B0"
              />
              <Controls 
                className="isa101-controls"
                showZoom={true}
                showFitView={true}
                showInteractive={false}
              />
              <MiniMap 
                className="isa101-minimap"
                nodeColor="#404040"
                maskColor="rgba(0, 0, 0, 0.1)"
              />
            </ReactFlow>
          </div>
          
          {/* Properties Panel */}
          <PropertiesPanel />
          
          {/* YAML Preview */}
          <YamlPreview />
        </div>
      ) : (
        <HMIDesigner />
      )}
    </div>
  )
}

export default function App() {
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
