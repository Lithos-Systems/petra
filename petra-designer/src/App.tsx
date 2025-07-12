// petra-designer/src/App.tsx
import { useCallback, useEffect, useState, useRef } from 'react'
import type { DragEvent, MouseEvent } from 'react'
import {
  ReactFlowProvider,
  type Node,
  type Edge,
  type Connection,
  type EdgeChange,
  type NodeChange,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'

import { Toaster } from 'react-hot-toast'
import toast from 'react-hot-toast'
import { useOptimizedFlowStore } from './store/optimizedFlowStore'
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts'
import Sidebar from './components/Sidebar'
import PropertiesPanel from './components/PropertiesPanel'
import YamlPreview from './components/YamlPreview'
import Toolbar from './components/Toolbar'
import { OptimizedReactFlow } from './components/OptimizedReactFlow'
import { ErrorBoundary } from './components/ErrorBoundary'
import HMIDesigner from './components/hmi/HMIDesigner'
import { FaProjectDiagram, FaDesktop } from 'react-icons/fa'
import { PetraProvider } from './contexts/PetraContext'
import { usePetraConnection } from './hooks/usePetraConnection'
import ConnectionStatus from './components/ConnectionStatus'
import './styles/isa101-theme.css'
import './styles/performance.css'

type DesignerMode = 'logic' | 'graphics'

function Flow() {
  const [mode, setMode] = useState<DesignerMode>('logic')
  const dragTimeoutRef = useRef<NodeJS.Timeout | null>(null)
  
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
    clearFlow
  } = useOptimizedFlowStore()

  // PETRA connection with cleanup
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
      // Silent disconnect to avoid popup nuisance
    },
  })

  // Add keyboard shortcuts
  useKeyboardShortcuts()

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      // Clear any pending drag timeouts
      if (dragTimeoutRef.current) {
        clearTimeout(dragTimeoutRef.current)
      }
      
      // Clear store timers
      const store = useOptimizedFlowStore.getState()
      if (store.updateTimer) {
        clearTimeout(store.updateTimer)
      }
    }
  }, [])

  // Apply ISA-101 mode class to body
  useEffect(() => {
    document.body.classList.add('isa101-mode')
    return () => {
      document.body.classList.remove('isa101-mode')
    }
  }, [])

  const onDrop = useCallback(
    (event: DragEvent) => {
      event.preventDefault()
      event.stopPropagation()

      const type = event.dataTransfer.getData('application/reactflow')
      const customDataStr = event.dataTransfer.getData('custom-data')
      
      if (!type) return

      // Use RAF to ensure smooth drop
      requestAnimationFrame(() => {
        const rect = (event.target as HTMLElement).getBoundingClientRect()
        const position = {
          x: event.clientX - rect.left - 60,
          y: event.clientY - rect.top - 40,
        }

        // Parse custom data if available
        let customData = {}
        if (customDataStr) {
          try {
            customData = JSON.parse(customDataStr)
          } catch (e) {
            console.error('Failed to parse custom data:', e)
          }
        }

        addNode(type, position, customData)
      })
    },
    [addNode]
  )

  const onDragOver = useCallback((event: DragEvent) => {
    event.preventDefault()
    event.dataTransfer.dropEffect = 'move'
  }, [])

  const onNodeClick = useCallback(
    (_: MouseEvent, node: Node) => {
      setSelectedNode(node)
    },
    [setSelectedNode]
  )

  const onPaneClick = useCallback(() => {
    setSelectedNode(null)
  }, [setSelectedNode])

  const onEdgeClick = useCallback(
    (_: MouseEvent, edge: Edge) => {
      if (confirm('Delete this connection?')) {
        deleteEdge(edge.id)
      }
    },
    [deleteEdge]
  )

  return (
    <div className="h-screen flex flex-col bg-[#D3D3D3]">
      <Toaster 
        position="bottom-right"
        toastOptions={{
          duration: 3000,
          style: {
            background: '#333',
            color: '#fff',
          },
        }}
      />
      
      {/* Header with mode switch */}
      <div className="bg-[#404040] text-white p-2">
        <div className="flex items-center">
          {/* Left: Title */}
          <h1 className="text-lg font-bold">PETRA Designer</h1>
          
          {/* Center: Mode switch */}
          <div className="flex items-center gap-2 mx-auto bg-[#606060] rounded">
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
          
          {/* Right: Connection Status */}
          <ConnectionStatus 
            connected={connected}
            connectionState={connectionState}
            signals={signals}
            performance={performance}
            lastError={lastError}
          />
        </div>
      </div>

      {mode === 'logic' ? (
        <div className="flex-1 flex">
          <div className="w-60 bg-[#EAEAEA] border-r border-[#404040]">
            <Sidebar />
          </div>
          
          <div className="flex-1 flex flex-col">
            <Toolbar />
            
            <div className="flex-1 relative">
              <ErrorBoundary>
                <OptimizedReactFlow
                  nodes={nodes}
                  edges={edges}
                  onNodesChange={onNodesChange}
                  onEdgesChange={onEdgesChange}
                  onConnect={onConnect}
                  onNodeClick={onNodeClick}
                  onPaneClick={onPaneClick}
                  onDrop={onDrop}
                  onDragOver={onDragOver}
                  onEdgeClick={onEdgeClick}
                  className="bg-[#F5F5F5]"
                />
              </ErrorBoundary>
            </div>
          </div>
          
          <div className="w-80 bg-[#EAEAEA] border-l border-[#404040] flex flex-col">
            <PropertiesPanel />
            <div className="border-t border-[#404040]">
              <YamlPreview />
            </div>
          </div>
        </div>
      ) : (
        <HMIDesigner />
      )}
    </div>
  )
}

function App() {
  return (
    <PetraProvider>
      <ReactFlowProvider>
        <Flow />
      </ReactFlowProvider>
    </PetraProvider>
  )
}

export default App
