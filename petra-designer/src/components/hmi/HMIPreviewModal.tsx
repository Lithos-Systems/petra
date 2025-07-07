// src/components/hmi/HMIPreviewModal.tsx

import { useEffect, useState } from 'react'
import { Stage, Layer } from 'react-konva'
import { FaTimes, FaExpand, FaCompress } from 'react-icons/fa'
import { HMIComponentRenderer } from './components/HMIComponentRenderer'
import { useHMIStore } from '@/store/hmiStore'
import { usePetra } from '@/contexts/PetraContext'
import type { HMIComponent } from '@/types/hmi'

interface HMIPreviewModalProps {
  isOpen: boolean
  onClose: () => void
}

export default function HMIPreviewModal({ isOpen, onClose }: HMIPreviewModalProps) {
  const { components: storeComponents, currentDisplay } = useHMIStore()
  const [components, setComponents] = useState<HMIComponent[]>([])
  const [isFullscreen, setIsFullscreen] = useState(false)
  const { signals, connected } = usePetra()
  
  // Copy components for preview (no editing)
  useEffect(() => {
    setComponents(storeComponents.map(c => ({ ...c })))
  }, [storeComponents])

  // Update component properties based on signal bindings
  useEffect(() => {
    const updatedComponents = storeComponents.map(component => {
      const updatedComponent = { ...component }
      
      // Apply signal bindings
      component.bindings.forEach(binding => {
        const signalValue = signals.get(binding.signal)
        if (signalValue !== undefined) {
          let value = signalValue
          
          // Apply transformation if specified
          if (binding.transform) {
            try {
              // Safe evaluation of transform expression
              const func = new Function('value', `return ${binding.transform}`)
              value = func(signalValue)
            } catch (e) {
              console.error('Transform error:', e)
            }
          }
          
          // Update component property
          updatedComponent.properties = {
            ...updatedComponent.properties,
            [binding.property]: value
          }
        }
      })
      
      return updatedComponent
    })
    
    setComponents(updatedComponents)
  }, [signals, storeComponents])

  if (!isOpen) return null

  const toggleFullscreen = () => {
    if (!isFullscreen) {
      document.documentElement.requestFullscreen()
    } else {
      document.exitFullscreen()
    }
    setIsFullscreen(!isFullscreen)
  }

  const displaySize = currentDisplay?.size || { width: 1920, height: 1080 }
  const scale = isFullscreen ? 1 : Math.min(
    (window.innerWidth - 100) / displaySize.width,
    (window.innerHeight - 200) / displaySize.height,
    1
  )

  return (
    <div className="fixed inset-0 z-50 bg-black bg-opacity-90 flex items-center justify-center">
      {/* Header */}
      <div className="absolute top-0 left-0 right-0 bg-gray-900 text-white p-4 flex items-center justify-between">
        <div className="flex items-center gap-4">
          <h2 className="text-xl font-semibold">Preview Mode</h2>
          <div className="flex items-center gap-2">
            <div className={`w-3 h-3 rounded-full ${connected ? 'bg-green-500' : 'bg-red-500'}`} />
            <span className="text-sm">{connected ? 'Connected' : 'Disconnected'}</span>
          </div>
        </div>
        
        <div className="flex items-center gap-4">
          <button
            onClick={toggleFullscreen}
            className="p-2 hover:bg-gray-800 rounded transition-colors"
            title={isFullscreen ? 'Exit Fullscreen' : 'Fullscreen'}
          >
            {isFullscreen ? <FaCompress /> : <FaExpand />}
          </button>
          <button
            onClick={onClose}
            className="p-2 hover:bg-gray-800 rounded transition-colors"
            title="Close Preview"
          >
            <FaTimes />
          </button>
        </div>
      </div>

      {/* Display */}
      <div 
        className="relative bg-gray-100 shadow-2xl"
        style={{
          marginTop: '60px',
          transform: `scale(${scale})`,
          transformOrigin: 'center',
        }}
      >
        <Stage
          width={displaySize.width}
          height={displaySize.height}
          style={{
            backgroundColor: currentDisplay?.background || '#f0f0f0',
          }}
        >
          <Layer>
            {components.map((component) => (
              <HMIComponentRenderer
                key={component.id}
                component={component}
                isSelected={false}
                onSelect={() => {}} // No selection in preview
                onUpdate={() => {}} // No updates in preview
              />
            ))}
          </Layer>
        </Stage>
      </div>

      {/* Status bar */}
      <div className="absolute bottom-0 left-0 right-0 bg-gray-900 text-white p-2 text-sm">
        <div className="flex items-center justify-between">
          <span>Display: {currentDisplay?.name || 'Untitled'}</span>
          <span>Size: {displaySize.width} × {displaySize.height}px</span>
          <span>Components: {components.length}</span>
          <span>Scale: {Math.round(scale * 100)}%</span>
        </div>
      </div>
    </div>
  )
}
