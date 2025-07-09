// File: petra-designer/src/components/EnhancedToolbar.tsx
// Enhanced PETRA Designer Toolbar with modern UI/UX
import { useState } from 'react'
import {
  FaSave,
  FaFolderOpen,
  FaDownload,
  FaUndo,
  FaRedo,
  FaTrash,
  FaPlay,
  FaPause,
  FaCog,
  FaSearchPlus,
  FaSearchMinus,
  FaExpand,
  FaCrosshairs,
  FaLayerGroup,
  FaMagic,
  FaPalette,
  FaRuler,
  FaMousePointer,
  FaHandPaper,
  FaCopy,
  FaPaste,
  FaCut,
  FaAlignCenter,
  FaTh,
  FaEye,
  FaEyeSlash,
  FaLock,
  FaUnlock,
  FaBars,
  FaChartLine,
  FaServer,
  FaWifi,
  FaWifiSlash,
  FaMoon,
  FaSun
} from 'react-icons/fa'
import { useFlowStore } from '../../store/flowStore'
import { usePetra } from '../../contexts/PetraContext'

interface ToolbarProps {
  onSave?: () => void
  onLoad?: () => void
  onExport?: () => void
  onClear?: () => void
  isDarkMode?: boolean
  onThemeToggle?: () => void
}

export default function EnhancedToolbar({
  onSave,
  onLoad,
  onExport,
  onClear,
  isDarkMode = false,
  onThemeToggle
}: ToolbarProps) {
  const { connected } = usePetra()
  const { canUndo, canRedo, undo, redo } = useFlowStore()
  const [activeTool, setActiveTool] = useState('select')
  const [showGrid, setShowGrid] = useState(true)
  const [snapToGrid, setSnapToGrid] = useState(true)
  const [showRulers, setShowRulers] = useState(false)
  const [showLayers, setShowLayers] = useState(false)
  const [simulationRunning, setSimulationRunning] = useState(false)
  const [zoomLevel, setZoomLevel] = useState(100)

  const handleZoomIn = () => {
    setZoomLevel(prev => Math.min(prev + 10, 200))
  }

  const handleZoomOut = () => {
    setZoomLevel(prev => Math.max(prev - 10, 25))
  }

  const handleZoomReset = () => {
    setZoomLevel(100)
  }

  const handleZoomFit = () => {
    // Implement zoom to fit logic
    console.log('Zoom to fit')
  }

  return (
    <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-4 py-2">
      {/* Main Toolbar */}
      <div className="flex items-center justify-between gap-4">
        {/* Left Section - File Operations */}
        <div className="flex items-center gap-2">
          {/* Logo/Brand */}
          <div className="flex items-center gap-2 mr-4">
            <div className="w-8 h-8 bg-gradient-to-br from-petra-primary-500 to-petra-primary-700 rounded flex items-center justify-center">
              <span className="text-white font-bold text-sm">P</span>
            </div>
            <span className="font-semibold text-gray-800 dark:text-gray-200">PETRA Designer</span>
          </div>

          <div className="h-6 w-px bg-gray-300 dark:bg-gray-600" />

          {/* File Operations */}
          <div className="flex items-center gap-1">
            <button
              onClick={onSave}
              className="petra-btn-toolbar"
              title="Save (Ctrl+S)"
            >
              <FaSave className="w-4 h-4" />
            </button>
            <button
              onClick={onLoad}
              className="petra-btn-toolbar"
              title="Open (Ctrl+O)"
            >
              <FaFolderOpen className="w-4 h-4" />
            </button>
            <button
              onClick={onExport}
              className="petra-btn-toolbar"
              title="Export"
            >
              <FaDownload className="w-4 h-4" />
            </button>
          </div>

          <div className="h-6 w-px bg-gray-300 dark:bg-gray-600" />

          {/* Edit Operations */}
          <div className="flex items-center gap-1">
            <button
              onClick={undo}
              disabled={!canUndo()}
              className={`petra-btn-toolbar ${!canUndo() ? 'opacity-50 cursor-not-allowed' : ''}`}
              title="Undo (Ctrl+Z)"
            >
              <FaUndo className="w-4 h-4" />
            </button>
            <button
              onClick={redo}
              disabled={!canRedo()}
              className={`petra-btn-toolbar ${!canRedo() ? 'opacity-50 cursor-not-allowed' : ''}`}
              title="Redo (Ctrl+Y)"
            >
              <FaRedo className="w-4 h-4" />
            </button>
            
            <div className="h-6 w-px bg-gray-300 dark:bg-gray-600 mx-1" />
            
            <button
              className="petra-btn-toolbar"
              title="Cut (Ctrl+X)"
            >
              <FaCut className="w-4 h-4" />
            </button>
            <button
              className="petra-btn-toolbar"
              title="Copy (Ctrl+C)"
            >
              <FaCopy className="w-4 h-4" />
            </button>
            <button
              className="petra-btn-toolbar"
              title="Paste (Ctrl+V)"
            >
              <FaPaste className="w-4 h-4" />
            </button>
            <button
              onClick={onClear}
              className="petra-btn-toolbar text-red-600 hover:text-red-700"
              title="Clear All"
            >
              <FaTrash className="w-4 h-4" />
            </button>
          </div>
        </div>

        {/* Center Section - Tools */}
        <div className="flex items-center gap-2">
          {/* Tool Selection */}
          <div className="flex items-center bg-gray-100 dark:bg-gray-700 rounded-lg p-1">
            <button
              onClick={() => setActiveTool('select')}
              className={`petra-tool-btn ${activeTool === 'select' ? 'petra-tool-btn-active' : ''}`}
              title="Select Tool (V)"
            >
              <FaMousePointer className="w-4 h-4" />
            </button>
            <button
              onClick={() => setActiveTool('pan')}
              className={`petra-tool-btn ${activeTool === 'pan' ? 'petra-tool-btn-active' : ''}`}
              title="Pan Tool (H)"
            >
              <FaHandPaper className="w-4 h-4" />
            </button>
          </div>

          <div className="h-6 w-px bg-gray-300 dark:bg-gray-600" />

          {/* View Controls */}
          <div className="flex items-center gap-1">
            <button
              onClick={handleZoomOut}
              className="petra-btn-toolbar"
              title="Zoom Out (-)"
            >
              <FaSearchMinus className="w-4 h-4" />
            </button>
            <div className="px-3 py-1 bg-gray-100 dark:bg-gray-700 rounded text-sm min-w-[60px] text-center">
              {zoomLevel}%
            </div>
            <button
              onClick={handleZoomIn}
              className="petra-btn-toolbar"
              title="Zoom In (+)"
            >
              <FaSearchPlus className="w-4 h-4" />
            </button>
            <button
              onClick={handleZoomFit}
              className="petra-btn-toolbar"
              title="Zoom to Fit"
            >
              <FaExpand className="w-4 h-4" />
            </button>
            <button
              onClick={handleZoomReset}
              className="petra-btn-toolbar"
              title="Reset Zoom (100%)"
            >
              <FaCrosshairs className="w-4 h-4" />
            </button>
          </div>

          <div className="h-6 w-px bg-gray-300 dark:bg-gray-600" />

          {/* Layout Tools */}
          <div className="flex items-center gap-1">
            <button
              onClick={() => setShowGrid(!showGrid)}
              className={`petra-btn-toolbar ${showGrid ? 'petra-btn-toolbar-active' : ''}`}
              title="Toggle Grid"
            >
              <FaTh className="w-4 h-4" />
            </button>
            <button
              onClick={() => setSnapToGrid(!snapToGrid)}
              className={`petra-btn-toolbar ${snapToGrid ? 'petra-btn-toolbar-active' : ''}`}
              title="Snap to Grid"
            >
              <FaMagic className="w-4 h-4" />
            </button>
            <button
              onClick={() => setShowRulers(!showRulers)}
              className={`petra-btn-toolbar ${showRulers ? 'petra-btn-toolbar-active' : ''}`}
              title="Show Rulers"
            >
              <FaRuler className="w-4 h-4" />
            </button>
            <button
              className="petra-btn-toolbar"
              title="Align Objects"
            >
              <FaAlignCenter className="w-4 h-4" />
            </button>
            <button
              onClick={() => setShowLayers(!showLayers)}
              className={`petra-btn-toolbar ${showLayers ? 'petra-btn-toolbar-active' : ''}`}
              title="Layers Panel"
            >
              <FaLayerGroup className="w-4 h-4" />
            </button>
          </div>
        </div>

        {/* Right Section - Status & Controls */}
        <div className="flex items-center gap-4">
          {/* Simulation Controls */}
          <div className="flex items-center gap-2">
            <button
              onClick={() => setSimulationRunning(!simulationRunning)}
              className={`petra-btn px-3 py-1.5 ${
                simulationRunning 
                  ? 'petra-btn-danger' 
                  : 'petra-btn-success'
              }`}
              title={simulationRunning ? 'Stop Simulation' : 'Start Simulation'}
            >
              {simulationRunning ? (
                <>
                  <FaPause className="w-4 h-4" />
                  <span className="text-sm">Stop</span>
                </>
              ) : (
                <>
                  <FaPlay className="w-4 h-4" />
                  <span className="text-sm">Run</span>
                </>
              )}
            </button>
          </div>

          <div className="h-6 w-px bg-gray-300 dark:bg-gray-600" />

          {/* Connection Status */}
          <div className="flex items-center gap-2">
            <div className={`flex items-center gap-2 px-3 py-1 rounded-full text-sm ${
              connected 
                ? 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300' 
                : 'bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300'
            }`}>
              {connected ? (
                <>
                  <FaWifi className="w-3 h-3" />
                  <span>Connected</span>
                </>
              ) : (
                <>
                  <FaWifiSlash className="w-3 h-3" />
                  <span>Disconnected</span>
                </>
              )}
            </div>
            <button
              className="petra-btn-toolbar"
              title="PETRA Settings"
            >
              <FaServer className="w-4 h-4" />
            </button>
          </div>

          <div className="h-6 w-px bg-gray-300 dark:bg-gray-600" />

          {/* Theme Toggle */}
          <button
            onClick={onThemeToggle}
            className="petra-btn-toolbar"
            title={isDarkMode ? 'Light Mode' : 'Dark Mode'}
          >
            {isDarkMode ? (
              <FaSun className="w-4 h-4 text-yellow-500" />
            ) : (
              <FaMoon className="w-4 h-4 text-gray-600" />
            )}
          </button>

          {/* Settings */}
          <button
            className="petra-btn-toolbar"
            title="Settings"
          >
            <FaCog className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Secondary Toolbar (Contextual) */}
      {showLayers && (
        <div className="mt-2 pt-2 border-t border-gray-200 dark:border-gray-700">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Layers:</span>
              <div className="flex items-center gap-1">
                <button className="petra-layer-btn petra-layer-active">
                  <FaEye className="w-3 h-3" />
                  <span>Equipment</span>
                </button>
                <button className="petra-layer-btn">
                  <FaEye className="w-3 h-3" />
                  <span>Piping</span>
                </button>
                <button className="petra-layer-btn">
                  <FaEye className="w-3 h-3" />
                  <span>Instruments</span>
                </button>
                <button className="petra-layer-btn petra-layer-locked">
                  <FaLock className="w-3 h-3" />
                  <span>Background</span>
                </button>
              </div>
            </div>
            <button
              onClick={() => setShowLayers(false)}
              className="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
            >
              <FaBars className="w-4 h-4" />
            </button>
          </div>
        </div>
      )}
    </div>
  )
}

// Add these styles to your CSS
const toolbarStyles = `
.petra-btn-toolbar {
  padding: 0.5rem;
  border-radius: 0.375rem;
  transition: all 0.2s ease;
  cursor: pointer;
  color: #4b5563;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}

.petra-btn-toolbar:hover {
  background-color: #f3f4f6;
  color: #1f2937;
}

.dark .petra-btn-toolbar {
  color: #9ca3af;
}

.dark .petra-btn-toolbar:hover {
  background-color: #374151;
  color: #f3f4f6;
}

.petra-btn-toolbar-active {
  background-color: #dbeafe;
  color: #2563eb;
}

.dark .petra-btn-toolbar-active {
  background-color: #1e3a8a;
  color: #60a5fa;
}

.petra-tool-btn {
  padding: 0.5rem;
  border-radius: 0.375rem;
  transition: all 0.15s ease;
  cursor: pointer;
  color: #6b7280;
}

.petra-tool-btn:hover {
  background-color: #e5e7eb;
  color: #1f2937;
}

.petra-tool-btn-active {
  background-color: #3b82f6;
  color: white;
}

.petra-tool-btn-active:hover {
  background-color: #2563eb;
}

.petra-layer-btn {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.25rem 0.75rem;
  border-radius: 0.25rem;
  font-size: 0.75rem;
  transition: all 0.15s ease;
  cursor: pointer;
  background-color: #f3f4f6;
  color: #4b5563;
}

.petra-layer-btn:hover {
  background-color: #e5e7eb;
}

.petra-layer-active {
  background-color: #dbeafe;
  color: #2563eb;
}

.petra-layer-locked {
  opacity: 0.5;
  cursor: not-allowed;
}

.petra-btn-success {
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
  color: white;
  box-shadow: 0 1px 3px 0 rgba(0, 0, 0, 0.1);
}

.petra-btn-success:hover {
  background: linear-gradient(135deg, #059669 0%, #047857 100%);
}

.petra-btn-danger {
  background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
  color: white;
  box-shadow: 0 1px 3px 0 rgba(0, 0, 0, 0.1);
}

.petra-btn-danger:hover {
  background: linear-gradient(135deg, #dc2626 0%, #b91c1c 100%);
}
`
