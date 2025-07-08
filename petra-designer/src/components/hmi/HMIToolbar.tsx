// src/components/hmi/HMIToolbar.tsx

import { useState } from 'react'
import { 
  FaSave, 
  FaFileExport, 
  FaPlay,
  FaUndo,
  FaRedo,
  FaSearchPlus,
  FaSearchMinus,
  FaExpand,
  FaTh,
  FaMagnet,
  FaAlignLeft,
  FaAlignCenter,
  FaAlignRight,
  FaCopy
} from 'react-icons/fa'
import { useHMIStore } from '@/store/hmiStore'
import toast from 'react-hot-toast'

import HMIPreviewModal from './HMIPreviewModal'

interface HMIToolbarProps {
  scale: number
  onScaleChange: (scale: number) => void
  stageSize: { width: number; height: number }
  onStageSizeChange: (size: { width: number; height: number }) => void
}

export default function HMIToolbar({
  scale,
  onScaleChange,
  stageSize,
  onStageSizeChange,
}: HMIToolbarProps) {
  const [showPreview, setShowPreview] = useState(false)
  const {
    showGrid,
    snapToGrid,
    toggleGrid,
    toggleSnapToGrid,
    saveDisplay,
    selectedComponentId,
    alignComponents,
    duplicateComponent,
    undo,
    redo,
    canUndo,
    canRedo,
  } = useHMIStore()

  const handleSave = () => {
    const display = saveDisplay()
    if (display) {
      localStorage.setItem('petra-hmi-display', JSON.stringify(display))
      toast.success('Display saved')
    }
  }

  const handleExport = () => {
    const display = saveDisplay()
    if (!display) return

    const exportData = {
      ...display,
      exportVersion: '1.0',
      exportDate: new Date().toISOString(),
    }

    const blob = new Blob([JSON.stringify(exportData, null, 2)], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `${display.name || 'hmi-display'}.json`
    a.click()
    URL.revokeObjectURL(url)
    toast.success('Display exported')
  }

  const handleZoomIn = () => {
    const newScale = Math.min(scale * 1.2, 5)
    onScaleChange(newScale)
  }

  const handleZoomOut = () => {
    const newScale = Math.max(scale * 0.8, 0.1)
    onScaleChange(newScale)
  }

  const handleZoomFit = () => {
    // Calculate scale to fit the stage
    const containerWidth = window.innerWidth - 600 // Account for sidebars
    const containerHeight = window.innerHeight - 120 // Account for toolbar
    
    const scaleX = containerWidth / stageSize.width
    const scaleY = containerHeight / stageSize.height
    const newScale = Math.min(scaleX, scaleY, 1) * 0.9 // 90% to add some padding
    
    onScaleChange(newScale)
  }

  const handleAlign = (alignment: 'left' | 'center-h' | 'right') => {
    // Get all selected components (for now just the one)
    if (selectedComponentId) {
      alignComponents([selectedComponentId], alignment)
      toast.success(`Aligned ${alignment}`)
    }
  }

  const handleDuplicate = () => {
    if (selectedComponentId) {
      duplicateComponent(selectedComponentId)
      toast.success('Component duplicated')
    }
  }

  const stageSizes = [
    { label: '1920×1080 (HD)', width: 1920, height: 1080 },
    { label: '1366×768', width: 1366, height: 768 },
    { label: '1024×768', width: 1024, height: 768 },
    { label: '800×600', width: 800, height: 600 },
    { label: 'Custom', width: 0, height: 0 },
  ]

  return (
    <div className="bg-white border-b border-gray-200 px-4 py-2">
      <div className="flex items-center justify-between">
        {/* Left section - File operations */}
        <div className="flex items-center gap-2">
          <button
            onClick={handleSave}
            className="p-2 text-gray-600 hover:text-gray-800 hover:bg-gray-100 rounded transition-colors"
            title="Save Display"
          >
            <FaSave className="w-4 h-4" />
          </button>
          <button
            onClick={handleExport}
            className="p-2 text-gray-600 hover:text-gray-800 hover:bg-gray-100 rounded transition-colors"
            title="Export Display"
          >
            <FaFileExport className="w-4 h-4" />
          </button>
          
          <div className="w-px h-6 bg-gray-300 mx-1" />
          
          <button
            onClick={undo}
            className={`p-2 rounded transition-colors ${
              canUndo() 
                ? 'text-gray-600 hover:text-gray-800 hover:bg-gray-100' 
                : 'text-gray-400 cursor-not-allowed'
            }`}
            title="Undo"
            disabled={!canUndo()}
          >
            <FaUndo className="w-4 h-4" />
          </button>
          <button
            onClick={redo}
            className={`p-2 rounded transition-colors ${
              canRedo() 
                ? 'text-gray-600 hover:text-gray-800 hover:bg-gray-100' 
                : 'text-gray-400 cursor-not-allowed'
            }`}
            title="Redo"
            disabled={!canRedo()}
          >
            <FaRedo className="w-4 h-4" />
          </button>
        </div>

        {/* Center section - Canvas controls */}
        <div className="flex items-center gap-2">
          {/* Stage size selector */}
          <select
            value={`${stageSize.width}x${stageSize.height}`}
            onChange={(e) => {
              const size = stageSizes.find(s => `${s.width}x${s.height}` === e.target.value)
              if (size && size.width > 0) {
                onStageSizeChange({ width: size.width, height: size.height })
              }
            }}
            className="px-3 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-petra-500"
          >
            {stageSizes.map((size) => (
              <option key={size.label} value={`${size.width}x${size.height}`}>
                {size.label}
              </option>
            ))}
          </select>

          <div className="w-px h-6 bg-gray-300 mx-1" />

          {/* Zoom controls */}
          <button
            onClick={handleZoomOut}
            className="p-2 text-gray-600 hover:text-gray-800 hover:bg-gray-100 rounded transition-colors"
            title="Zoom Out"
          >
            <FaSearchMinus className="w-4 h-4" />
          </button>
          <span className="text-sm text-gray-600 min-w-[50px] text-center">
            {Math.round(scale * 100)}%
          </span>
          <button
            onClick={handleZoomIn}
            className="p-2 text-gray-600 hover:text-gray-800 hover:bg-gray-100 rounded transition-colors"
            title="Zoom In"
          >
            <FaSearchPlus className="w-4 h-4" />
          </button>
          <button
            onClick={handleZoomFit}
            className="p-2 text-gray-600 hover:text-gray-800 hover:bg-gray-100 rounded transition-colors"
            title="Fit to Screen"
          >
            <FaExpand className="w-4 h-4" />
          </button>

          <div className="w-px h-6 bg-gray-300 mx-1" />

          {/* Grid controls */}
          <button
            onClick={toggleGrid}
            className={`p-2 rounded transition-colors ${
              showGrid 
                ? 'text-petra-600 bg-petra-50' 
                : 'text-gray-600 hover:text-gray-800 hover:bg-gray-100'
            }`}
            title="Toggle Grid"
          >
            <FaTh className="w-4 h-4" />
          </button>
          <button
            onClick={toggleSnapToGrid}
            className={`p-2 rounded transition-colors ${
              snapToGrid 
                ? 'text-petra-600 bg-petra-50' 
                : 'text-gray-600 hover:text-gray-800 hover:bg-gray-100'
            }`}
            title="Snap to Grid"
          >
            <FaMagnet className="w-4 h-4" />
          </button>
        </div>

        {/* Right section - Edit operations */}
        <div className="flex items-center gap-2">
          {/* Alignment tools */}
          <button
            onClick={() => handleAlign('left')}
            className="p-2 text-gray-600 hover:text-gray-800 hover:bg-gray-100 rounded transition-colors"
            title="Align Left"
            disabled={!selectedComponentId}
          >
            <FaAlignLeft className="w-4 h-4" />
          </button>
          <button
            onClick={() => handleAlign('center-h')}
            className="p-2 text-gray-600 hover:text-gray-800 hover:bg-gray-100 rounded transition-colors"
            title="Align Center"
            disabled={!selectedComponentId}
          >
            <FaAlignCenter className="w-4 h-4" />
          </button>
          <button
            onClick={() => handleAlign('right')}
            className="p-2 text-gray-600 hover:text-gray-800 hover:bg-gray-100 rounded transition-colors"
            title="Align Right"
            disabled={!selectedComponentId}
          >
            <FaAlignRight className="w-4 h-4" />
          </button>

          <div className="w-px h-6 bg-gray-300 mx-1" />

          {/* Copy/Paste/Duplicate */}
          <button
            onClick={handleDuplicate}
            className="p-2 text-gray-600 hover:text-gray-800 hover:bg-gray-100 rounded transition-colors"
            title="Duplicate"
            disabled={!selectedComponentId}
          >
            <FaCopy className="w-4 h-4" />
          </button>

          <div className="w-px h-6 bg-gray-300 mx-1" />

          {/* Runtime preview */}
          <button
            onClick={() => setShowPreview(true)}
            className="flex items-center gap-2 px-3 py-1 bg-green-500 text-white rounded hover:bg-green-600 transition-colors"
            title="Preview Display"
          >
            <FaPlay className="w-4 h-4" />
            <span className="text-sm font-medium">Preview</span>
          </button>
        </div>
      </div>
      
      {/* Preview Modal */}
      <HMIPreviewModal 
        isOpen={showPreview} 
        onClose={() => setShowPreview(false)} 
      />
    </div>
  )
}
