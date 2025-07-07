// src/components/hmi/HMIDesigner.tsx

import { useState, useCallback, useRef } from 'react'
import { Stage, Layer } from 'react-konva'
import { KonvaEventObject } from 'konva/lib/Node'
import HMISidebar from './HMISidebar'
import HMIPropertiesPanel from './HMIPropertiesPanel'
import HMIToolbar from './HMIToolbar'
import { useHMIStore } from '@/store/hmiStore'
import { HMIComponentRenderer } from './components/HMIComponentRenderer'
import { nanoid } from 'nanoid'
import type { HMIComponent } from '@/types/hmi'
import GridOverlay from './GridOverlay'

import { useHMIKeyboardShortcuts } from '@/hooks/useHMIKeyboardShortcuts'

import MQTTTestDisplay from './MQTTTestDisplay'

export default function HMIDesigner() {
  const stageRef = useRef<any>(null)
  const [showMQTTTest, setShowMQTTTest] = useState(false)
  const {
    components,
    selectedComponentId,
    addComponent,
    selectComponent,
    clearSelection,
    updateComponent,
    deleteComponent,
    showGrid,
    snapToGrid,
  } = useHMIStore()

  // Enable keyboard shortcuts
  useHMIKeyboardShortcuts()

  const [stageSize, setStageSize] = useState({ width: 1920, height: 1080 })
  const [scale, setScale] = useState(1)

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    const type = e.dataTransfer.getData('hmi-component')
    if (!type) return

    const stage = stageRef.current
    if (!stage) return

    // Get stage position
    const stageBox = stage.container().getBoundingClientRect()
    const pointerPosition = {
      x: (e.clientX - stageBox.left) / scale,
      y: (e.clientY - stageBox.top) / scale,
    }

    // Snap to grid if enabled
    let x = pointerPosition.x
    let y = pointerPosition.y
    if (snapToGrid) {
      x = Math.round(x / 20) * 20
      y = Math.round(y / 20) * 20
    }

    const newComponent: HMIComponent = {
      id: nanoid(),
      type: type as any,
      position: { x, y },
      size: getDefaultSize(type),
      rotation: 0,
      bindings: [],
      animations: [],
      interactions: [],
      style: getDefaultStyle(type),
      properties: getDefaultProperties(type),
    }

    addComponent(newComponent)
  }, [scale, snapToGrid, addComponent])

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.dataTransfer.dropEffect = 'copy'
  }, [])

  const handleStageClick = useCallback((e: KonvaEventObject<MouseEvent>) => {
    // Check if clicked on empty space
    const clickedOnEmpty = e.target === e.target.getStage()
    if (clickedOnEmpty) {
      clearSelection()
    }
  }, [clearSelection])

  const handleWheel = useCallback((e: KonvaEventObject<WheelEvent>) => {
    e.evt.preventDefault()
    const stage = stageRef.current
    if (!stage) return

    const oldScale = scale
    const pointer = stage.getPointerPosition()
    const mousePointTo = {
      x: (pointer.x - stage.x()) / oldScale,
      y: (pointer.y - stage.y()) / oldScale,
    }

    const newScale = e.evt.deltaY > 0 ? oldScale * 0.9 : oldScale * 1.1
    const clampedScale = Math.max(0.1, Math.min(5, newScale))
    
    setScale(clampedScale)

    const newPos = {
      x: pointer.x - mousePointTo.x * clampedScale,
      y: pointer.y - mousePointTo.y * clampedScale,
    }
    stage.position(newPos)
    stage.batchDraw()
  }, [scale])

  const selectedComponent = components.find(c => c.id === selectedComponentId)

  return (
    <div className="flex-1 flex bg-gray-50">
      <HMISidebar />

      <div className="flex-1 flex flex-col">
        <HMIToolbar 
          scale={scale}
          onScaleChange={setScale}
          stageSize={stageSize}
          onStageSizeChange={setStageSize}
        />

        <div 
          className="flex-1 relative overflow-hidden bg-gray-100"
          onDrop={handleDrop}
          onDragOver={handleDragOver}
        >
          <Stage
            ref={stageRef}
            width={window.innerWidth - 600} // Account for sidebars
            height={window.innerHeight - 120} // Account for toolbar
            onMouseDown={handleStageClick}
            onWheel={handleWheel}
            scaleX={scale}
            scaleY={scale}
          >
            <Layer>
              {/* Grid */}
              {showGrid && (
                <GridOverlay 
                  width={stageSize.width} 
                  height={stageSize.height}
                  gridSize={20}
                />
              )}

              {/* HMI Components */}
              {components.map((component) => (
                <HMIComponentRenderer
                  key={component.id}
                  component={component}
                  isSelected={component.id === selectedComponentId}
                  onSelect={() => selectComponent(component.id)}
                  onUpdate={(updates) => updateComponent(component.id, updates)}
                />
              ))}
            </Layer>
          </Stage>

          {/* Canvas size indicator */}
          <div className="absolute bottom-4 left-4 bg-white px-3 py-2 rounded shadow text-sm text-gray-600">
            Canvas: {stageSize.width} × {stageSize.height}px | Zoom: {Math.round(scale * 100)}%
          </div>
          
          {/* MQTT Test Toggle */}
          <button
            onClick={() => setShowMQTTTest(!showMQTTTest)}
            className="absolute bottom-4 right-4 bg-petra-500 text-white px-3 py-2 rounded shadow hover:bg-petra-600 transition-colors text-sm"
          >
            {showMQTTTest ? 'Hide' : 'Show'} MQTT Test
          </button>
        </div>
      </div>

      {selectedComponent && (
        <HMIPropertiesPanel 
          component={selectedComponent}
          onUpdate={(updates) => updateComponent(selectedComponent.id, updates)}
          onDelete={() => deleteComponent(selectedComponent.id)}
        />
      )}
      
      {/* MQTT Test Display */}
      {showMQTTTest && <MQTTTestDisplay />}
    </div>
  )
}

// Helper functions
function getDefaultSize(type: string) {
  const sizes: Record<string, { width: number; height: number }> = {
    tank: { width: 120, height: 160 },
    pump: { width: 80, height: 80 },
    valve: { width: 60, height: 60 },
    gauge: { width: 150, height: 150 },
    trend: { width: 400, height: 200 },
    text: { width: 200, height: 40 },
    button: { width: 120, height: 40 },
    pipe: { width: 100, height: 20 },
    motor: { width: 100, height: 100 },
  }
  return sizes[type] || { width: 100, height: 100 }
}

function getDefaultStyle(type: string) {
  const styles: Record<string, any> = {
    tank: {
      fill: '#4444ff',
      stroke: '#333333',
      strokeWidth: 2,
      opacity: 0.8,
    },
    pump: {
      fill: '#10b981',
      stroke: '#333333',
      strokeWidth: 2,
    },
    valve: {
      fill: '#666666',
      stroke: '#333333',
      strokeWidth: 2,
    },
    gauge: {
      fill: '#ffffff',
      stroke: '#333333',
      strokeWidth: 2,
    },
    trend: {
      fill: '#1a1a1a',
      stroke: '#333333',
      strokeWidth: 1,
    },
    text: {
      fontSize: 16,
      fontFamily: 'Arial',
      fill: '#000000',
    },
    button: {
      fill: '#3b82f6',
      stroke: '#2563eb',
      strokeWidth: 1,
    },
  }
  return styles[type] || {}
}

function getDefaultProperties(type: string) {
  const properties: Record<string, any> = {
    tank: {
      maxLevel: 100,
      currentLevel: 50,
      alarmHigh: 80,
      alarmLow: 20,
      showLabel: true,
      units: '%',
      showWaveAnimation: true,
    },
    pump: {
      running: false,
      fault: false,
      speed: 0,
      showStatus: true,
      runAnimation: true,
    },
    valve: {
      open: false,
      fault: false,
      position: 0,
      valveType: 'gate',
      showPosition: true,
    },
    gauge: {
      min: 0,
      max: 100,
      value: 50,
      units: '',
      showScale: true,
      majorTicks: 5,
    },
    trend: {
      timeRange: '1h',
      yMin: 0,
      yMax: 100,
      showGrid: true,
      showLegend: true,
    },
    text: {
      text: 'Label',
      align: 'center',
    },
    button: {
      text: 'Button',
      action: 'toggle',
      confirmRequired: false,
    },
    'heat-exchanger': {
      hotInletTemp: 80,
      hotOutletTemp: 60,
      coldInletTemp: 20,
      coldOutletTemp: 40,
      efficiency: 85,
      showTemperatures: true,
    },
    conveyor: {
      running: false,
      speed: 50,
      direction: 'forward',
      material: false,
    },
    mixer: {
      running: false,
      speed: 60,
      level: 75,
      agitatorType: 'paddle',
      temperature: 25,
    },
  }
  return properties[type] || {}
}
