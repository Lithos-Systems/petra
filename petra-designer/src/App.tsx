// petra-designer/src/App.tsx
import { useCallback, useEffect, DragEvent, MouseEvent, useState } from 'react'
import {
  ReactFlowProvider,
  type Node,
  useKeyPress,
  Connection,
  Edge,
  EdgeChange,
  NodeChange,
  getBezierPath,
  EdgeProps// petra-designer/src/App.tsx
import { useCallback, useEffect, DragEvent, MouseEvent, useState } from 'react'
import {
  ReactFlowProvider,
  type Node,
  useKeyPress,
  Connection,
  Edge,
  EdgeChange,
  NodeChange,
  getBezierPath,
  EdgeProps,
  BaseEdge,
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
import ISA101WaterPlantDemo from './components/hmi/ISA101WaterPlantDemo'
import { FaProjectDiagram, FaDesktop } from 'react-icons/fa'
import { PetraProvider } from './contexts/PetraContext'
import { usePetraConnection } from './hooks/usePetraConnection'
import type { ConnectionInfo } from './types/hmi'
import ConnectionStatus from './components/ConnectionStatus'
import './styles/isa101-theme.css'
import './styles/performance.css'

type DesignerMode = 'logic' | 'graphics'

// Custom edge component with bezier curves
const CustomEdge = ({ 
  id, 
  sourceX, 
  sourceY, 
  targetX, 
  targetY, 
  sourcePosition, 
  targetPosition, 
  data,
  selected
}: EdgeProps) => {
  const [edgePath] = getBezierPath({
    sourceX,
    sourceY,
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
  })

  return (
    <>
      <BaseEdge 
        id={id} 
        path={edgePath} 
        style={{
          stroke: selected ? '#000080' : '#000000',
          strokeWidth: selected ? 3 : 2,
        }}
      />
    </>
  )
}

// Define edge types
const edgeTypes = {
  default: CustomEdge,
}

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
  } = useOptimizedFlowStore()

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
      // Removed the disconnect toast to avoid popup nuisance
    },
  })

  // Add keyboard shortcuts
  useKeyboardShortcuts()

  // Clean up any queued timers/listeners to avoid memory leaks
  useEffect(() => {
    const cleanup = () => {
      const highestId = window.setTimeout(() => {}, 0)
      for (let i = 0; i <= highestId; i++) {
        window.clearTimeout(i)
      }
    }
    window.addEventListener('beforeunload', cleanup)
    return () => {
      window.removeEventListener('beforeunload', cleanup)
      cleanup()
    }
  }, [])

  // Apply ISA-101 mode class to body
  useEffect(() => {
    document.body.classList.add('isa101-mode')
  }, [])

  const onDrop = useCallback(
    (event: DragEvent) => {
      event.preventDefault()

      const type = event.dataTransfer.getData('application/reactflow')
      
      if (!type) return

      const position = {
        x: event.clientX - 250,
        y: event.clientY - 50,
      }

      addNode(type, position)
    },
    [addNode]
  )

  const onDragOver = useCallback((event: DragEvent) => {
    event.preventDefault()
    event.dataTransfer.dropEffect = 'move'
  }, [])

  const onNodeClick = useCallback(
    (event: MouseEvent, node: Node) => {
      event.stopPropagation()
      setSelectedNode(node)
    },
    [setSelectedNode]
  )

  const onPaneClick = useCallback(() => {
    setSelectedNode(null)
  }, [setSelectedNode])

  const onEdgeClick = useCallback(
    (event: MouseEvent, edge: Edge) => {
      event.stopPropagation()
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

          {/* Right: Spacer */}
          <div className="flex-1" />
        </div>
      </div>

      {/* Conditional rendering based on mode */}
      {mode === 'logic' ? (
        <div className="flex flex-1">
          {/* Sidebar */}
          <Sidebar />
          
          {/* Main Flow Area */}
          <div className="flex-1 flex flex-col">
            <OptimizedReactFlow
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
              edgeTypes={edgeTypes}
              defaultEdgeOptions={{
                type: 'default',
                animated: false,
                style: { strokeWidth: 2 }
              }}
              className="bg-[#D3D3D3]"
            />
          </div>
          
          {/* Properties Panel */}
          <PropertiesPanel />
          
          {/* YAML Preview */}
          <YamlPreview />
        </div>
      ) : (
        <HMIDesigner />
      )}
      <ConnectionStatus />
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
  BaseEdge,
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
import ISA101WaterPlantDemo from './components/hmi/ISA101WaterPlantDemo'
import { FaProjectDiagram, FaDesktop } from 'react-icons/fa'
import { PetraProvider } from './contexts/PetraContext'
import { usePetraConnection } from './hooks/usePetraConnection'
import type { ConnectionInfo } from './types/hmi'
import ConnectionStatus from './components/ConnectionStatus'
import './styles/isa101-theme.css'
import './styles/performance.css'

type DesignerMode = 'logic' | 'graphics'

// Custom edge component with bezier curves
const CustomEdge = ({ 
  id, 
  sourceX, 
  sourceY, 
  targetX, 
  targetY, 
  sourcePosition, 
  targetPosition, 
  data,
  selected
}: EdgeProps) => {
  const [edgePath] = getBezierPath({
    sourceX,
    sourceY,
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
  })

  return (
    <>
      <BaseEdge 
        id={id} 
        path={edgePath} 
        style={{
          stroke: selected ? '#000080' : '#000000',
          strokeWidth: selected ? 3 : 2,
        }}
      />
    </>
  )
}

// Define edge types
const edgeTypes = {
  default: CustomEdge,
}

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
  } = useOptimizedFlowStore()

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
      // Removed the disconnect toast to avoid popup nuisance
    },
  })

  // Add keyboard shortcuts
  useKeyboardShortcuts()

  // Clean up any queued timers/listeners to avoid memory leaks
  useEffect(() => {
    const cleanup = () => {
      const highestId = window.setTimeout(() => {}, 0)
      for (let i = 0; i <= highestId; i++) {
        window.clearTimeout(i)
      }
    }
    window.addEventListener('beforeunload', cleanup)
    return () => {
      window.removeEventListener('beforeunload', cleanup)
      cleanup()
    }
  }, [])

  // Apply ISA-101 mode class to body
  useEffect(() => {
    document.body.classList.add('isa101-mode')
  }, [])

  const onDrop = useCallback(
    (event: DragEvent) => {
      event.preventDefault()

      const type = event.dataTransfer.getData('application/reactflow')
      const customData = event.dataTransfer.getData('custom-data')
      
      if (!type) return

      const position = {
        x: event.clientX - 250,
        y: event.clientY - 50,
      }

      // Parse custom data if present
      let parsedData = {}
      if (customData) {
        try {
          parsedData = JSON.parse(customData)
        } catch (e) {
          console.error('Failed to parse custom data:', e)
        }
      }

      addNode(type, position, parsedData)
    },
    [addNode]
  )

  const onDragOver = useCallback((event: DragEvent) => {
    event.preventDefault()
    event.dataTransfer.dropEffect = 'move'
  }, [])

  const onNodeClick = useCallback(
    (event: MouseEvent, node: Node) => {
      event.stopPropagation()
      setSelectedNode(node)
    },
    [setSelectedNode]
  )

  const onPaneClick = useCallback(() => {
    setSelectedNode(null)
  }, [setSelectedNode])

  const onEdgeClick = useCallback(
    (event: MouseEvent, edge: Edge) => {
      event.stopPropagation()
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

          {/* Right: Spacer */}
          <div className="flex-1" />
        </div>
      </div>

      {/* Conditional rendering based on mode */}
      {mode === 'logic' ? (
        <div className="flex flex-1">
          {/* Sidebar */}
          <Sidebar />
          
          {/* Main Flow Area */}
          <div className="flex-1 flex flex-col">
            <OptimizedReactFlow
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
              edgeTypes={edgeTypes}
              defaultEdgeOptions={{
                type: 'default',
                animated: false,
                style: { strokeWidth: 2 }
              }}
              className="bg-[#D3D3D3]"
            />
          </div>
          
          {/* Properties Panel */}
          <PropertiesPanel />
          
          {/* YAML Preview */}
          <YamlPreview />
        </div>
      ) : (
        <HMIDesigner />
      )}
      <ConnectionStatus />
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
