// @ts-nocheck
import { useState, useEffect, useRef } from 'react'
import { Stage, Layer, Rect, Line } from 'react-konva'
import { 
  FaBars, FaTimes, FaIndustry, FaChartLine, FaFont, FaShapes,
  FaPalette, FaEye, FaEyeSlash, FaLock, FaUnlock, FaTrash,
  FaCog, FaSave, FaFolder, FaPlay, FaPause, FaExclamationTriangle,
  FaLink, FaUnlink
} from 'react-icons/fa'
import ISA101TankComponent from './components/ISA101TankComponent'
import ISA101PumpComponent from './components/ISA101PumpComponent'
import ISA101ValveComponent from './components/ISA101ValveComponent'
import ISA101GaugeComponent from './components/ISA101GaugeComponent'
import ISA101MotorComponent from './components/ISA101MotorComponent'
import ISA101MixerComponent from './components/ISA101MixerComponent'
import ISA101TextComponent from './components/ISA101TextComponent'
import ISA101ButtonComponent from './components/ISA101ButtonComponent'
import ISA101IndicatorComponent from './components/ISA101IndicatorComponent'
import ISA101PipeComponent from './components/ISA101PipeComponent'
import ISA101ShapeComponent from './components/ISA101ShapeComponent'
import { usePetra } from '../../contexts/PetraContext'

// ISA-101 Standard Colors
const ISA101Colors = {
  background: '#F0F0F0',
  toolbarBg: '#E6E6E6',
  toolbarBorder: '#C0C0C0',
  buttonBg: '#FFFFFF',
  buttonHover: '#D6D6D6',
  buttonActive: '#B0B0B0',
  text: '#000000',
  textSecondary: '#666666',
  accent: '#0080FF',
  gridLine: '#D0D0D0',
  categoryHeader: '#D0D0D0',
}

// Component categories following ISA-101 organization
const componentCategories = [
  {
    name: 'Process Equipment',
    icon: <FaIndustry />,
    components: [
      { 
        type: 'tank', 
        label: 'Tank',
        defaultProps: {
          tagName: 'TK-101',
          currentLevel: 50,
          levelUnits: '%',
          maxLevel: 100,
          minLevel: 0,
          alarmHigh: 80,
          alarmLow: 20,
          showAlarmLimits: true,
          materialType: 'water'
        }
      },
      { 
        type: 'pump', 
        label: 'Pump',
        defaultProps: {
          tagName: 'P-101',
          status: 'stopped',
          flowRate: 0,
          flowUnits: 'GPM',
          dischargePressure: 0,
          pressureUnits: 'PSI',
          controlMode: 'auto',
          showDetailedStatus: true
        }
      },
      { 
        type: 'valve', 
        label: 'Valve',
        defaultProps: {
          tagName: 'V-101',
          position: 0,
          status: 'closed',
          valveType: 'gate',
          controlMode: 'auto',
          showPosition: true,
          orientation: 'horizontal'
        }
      },
      { 
        type: 'motor', 
        label: 'Motor',
        defaultProps: {
          tagName: 'M-101',
          running: false,
          speed: 0,
          fault: false,
          controlMode: 'auto'
        }
      },
      { 
        type: 'mixer', 
        label: 'Mixer',
        defaultProps: {
          tagName: 'MX-101',
          running: false,
          speed: 0,
          level: 50,
          agitatorType: 'turbine'
        }
      }
    ]
  },
  {
    name: 'Instrumentation',
    icon: <FaChartLine />,
    components: [
      { 
        type: 'gauge', 
        label: 'Gauge',
        defaultProps: {
          tagName: 'PI-101',
          currentValue: 0,
          units: 'PSI',
          minValue: 0,
          maxValue: 100,
          showDigitalValue: true,
          gaugeType: 'pressure'
        }
      },
      { 
        type: 'trend', 
        label: 'Trend',
        defaultProps: {
          tagName: 'TREND-101',
          signals: [],
          timeRange: '1h',
          yMin: 0,
          yMax: 100,
          showGrid: true,
          showLegend: true
        }
      },
      { 
        type: 'indicator', 
        label: 'Indicator',
        defaultProps: {
          tagName: 'IND-101',
          on: false,
          onColor: '#00FF00',
          offColor: '#808080'
        }
      }
    ]
  },
  {
    name: 'Controls',
    icon: <FaCog />,
    components: [
      { 
        type: 'button', 
        label: 'Button',
        defaultProps: {
          text: 'START',
          action: 'momentary',
          confirmRequired: false,
          activeColor: '#00FF00',
          inactiveColor: '#808080'
        }
      },
      { 
        type: 'setpoint', 
        label: 'Setpoint',
        defaultProps: {
          tagName: 'SP-101',
          value: 50,
          min: 0,
          max: 100,
          units: '%'
        }
      }
    ]
  },
  {
    name: 'Annotation',
    icon: <FaFont />,
    components: [
      { 
        type: 'text', 
        label: 'Text',
        defaultProps: {
          text: 'Label',
          fontSize: 12,
          fontWeight: 'normal',
          textAlign: 'left'
        }
      },
      { 
        type: 'title', 
        label: 'Title',
        defaultProps: {
          text: 'Process Area',
          fontSize: 18,
          fontWeight: 'bold',
          textAlign: 'center'
        }
      }
    ]
  },
  {
    name: 'Graphics',
    icon: <FaShapes />,
    components: [
      { 
        type: 'pipe', 
        label: 'Pipe',
        defaultProps: {
          points: [0, 0, 100, 0],
          flowAnimation: false,
          pipeSize: 6
        }
      },
      { 
        type: 'shape', 
        label: 'Rectangle',
        defaultProps: {
          shapeType: 'rectangle',
          fill: 'transparent',
          stroke: '#000000',
          strokeWidth: 2
        }
      }
    ]
  }
]

export default function ISA101HMIDesigner() {
  const { connected, signals, subscribeSignal, unsubscribeSignal } = usePetra()
  const [components, setComponents] = useState([])
  const [selectedId, setSelectedId] = useState(null)
  const [showGrid, setShowGrid] = useState(true)
  const [gridSize, setGridSize] = useState(20)
  const [snapToGrid, setSnapToGrid] = useState(true)
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false)
  const [activeCategory, setActiveCategory] = useState(0)
  const [stageSize, setStageSize] = useState({ width: 1200, height: 800 })
  const [isDragging, setIsDragging] = useState(false)
  const [showBindingPanel, setShowBindingPanel] = useState(false)
  const [availableSignals, setAvailableSignals] = useState([])
  const stageRef = useRef()
  const draggedComponent = useRef(null)

  // Update component properties based on signal bindings
  useEffect(() => {
    if (!connected) return

    // Update components when signals change
    const updateComponentsFromSignals = () => {
      setComponents(prevComponents => 
        prevComponents.map(component => {
          if (!component.bindings || component.bindings.length === 0) return component
          
          const updatedProperties = { ...component.properties }
          let hasChanges = false

          component.bindings.forEach(binding => {
            const signalValue = signals.get(binding.signal)
            if (signalValue !== undefined) {
              let processedValue = signalValue
              
              // Apply transformation if specified
              if (binding.transform) {
                try {
                  // Create a safe evaluation context
                  const evalFunction = new Function('value', `return ${binding.transform}`)
                  processedValue = evalFunction(signalValue)
                } catch (e) {
                  console.error(`Transform error for ${binding.signal}:`, e)
                }
              }
              
              // Update the property if it changed
              if (updatedProperties[binding.property] !== processedValue) {
                updatedProperties[binding.property] = processedValue
                hasChanges = true
              }
            }
          })

          return hasChanges ? { ...component, properties: updatedProperties } : component
        })
      )
    }

    // Update on signal changes
    const interval = setInterval(updateComponentsFromSignals, 100)
    
    return () => clearInterval(interval)
  }, [connected, signals])

  // Update stage size on window resize
  useEffect(() => {
    const updateSize = () => {
      const container = document.getElementById('canvas-container')
      if (container) {
        setStageSize({
          width: container.offsetWidth,
          height: container.offsetHeight
        })
      }
    }
    updateSize()
    window.addEventListener('resize', updateSize)
    return () => window.removeEventListener('resize', updateSize)
  }, [sidebarCollapsed])

  // Add component to canvas
  const addComponent = (componentType) => {
    const category = componentCategories.find(cat => 
      cat.components.some(comp => comp.type === componentType)
    )
    const componentDef = category?.components.find(comp => comp.type === componentType)
    
    if (!componentDef) return

    // Component-specific default sizes
    const getDefaultSize = (type) => {
      switch (type) {
        case 'tank': return { width: 120, height: 150 }
        case 'mixer': return { width: 100, height: 120 }
        case 'gauge': return { width: 100, height: 100 }
        case 'text':
        case 'title': return { width: 200, height: 30 }
        case 'button': return { width: 100, height: 40 }
        case 'indicator': return { width: 40, height: 40 }
        case 'pipe': return { width: 100, height: 20 }
        case 'shape': return { width: 80, height: 80 }
        default: return { width: 80, height: 80 }
      }
    }

    const newComponent = {
      id: `${componentType}-${Date.now()}`,
      type: componentType,
      position: { x: 100, y: 100 },
      size: getDefaultSize(componentType),
      rotation: 0,
      properties: { ...componentDef.defaultProps },
      bindings: [],
      animations: [],
      interactions: [],
      style: {},
      locked: false,
      visible: true
    }

    setComponents([...components, newComponent])
    setSelectedId(newComponent.id)
  }

  // Update component
  const updateComponent = (id, updates) => {
    setComponents(components.map(comp => 
      comp.id === id ? { ...comp, ...updates } : comp
    ))
  }

  // Delete selected component
  const deleteSelected = () => {
    if (selectedId) {
      setComponents(components.filter(comp => comp.id !== selectedId))
      setSelectedId(null)
    }
  }

  // Render component based on type
  const renderComponent = (component) => {
    const commonProps = {
      x: component.position.x,
      y: component.position.y,
      width: component.size.width,
      height: component.size.height,
      properties: component.properties,
      style: component.style,
      selected: component.id === selectedId,
      onClick: () => setSelectedId(component.id),
      draggable: !component.locked,
      onDragEnd: (e) => {
        const node = e.target
        let x = node.x()
        let y = node.y()
        
        if (snapToGrid) {
          x = Math.round(x / gridSize) * gridSize
          y = Math.round(y / gridSize) * gridSize
          node.x(x)
          node.y(y)
        }
        
        updateComponent(component.id, {
          position: { x, y }
        })
      },
      onDragStart: (e) => {
        e.target.moveToTop()
        setIsDragging(true)
      }
    }

    switch (component.type) {
      case 'tank':
        return <ISA101TankComponent key={component.id} {...commonProps} />
      case 'pump':
        return <ISA101PumpComponent key={component.id} {...commonProps} />
      case 'valve':
        return <ISA101ValveComponent key={component.id} {...commonProps} />
      case 'gauge':
        return <ISA101GaugeComponent key={component.id} {...commonProps} />
      case 'motor':
        return <ISA101MotorComponent key={component.id} {...commonProps} />
      case 'mixer':
        return <ISA101MixerComponent key={component.id} {...commonProps} />
      case 'text':
      case 'title':
        return <ISA101TextComponent key={component.id} {...commonProps} />
      case 'button':
        return <ISA101ButtonComponent key={component.id} {...commonProps} />
      case 'indicator':
        return <ISA101IndicatorComponent key={component.id} {...commonProps} />
      case 'pipe':
        return <ISA101PipeComponent key={component.id} {...commonProps} />
      case 'shape':
        return <ISA101ShapeComponent key={component.id} {...commonProps} />
      default:
        console.warn(`Unknown component type: ${component.type}`)
        return null
    }
  }

  return (
    <div className="flex h-screen bg-gray-100">
      {/* ISA-101 Compliant Sidebar */}
      <div 
        className={`${
          sidebarCollapsed ? 'w-12' : 'w-64'
        } transition-all duration-300 flex flex-col border-r-2`}
        style={{ 
          backgroundColor: ISA101Colors.toolbarBg,
          borderColor: ISA101Colors.toolbarBorder 
        }}
      >
        {/* Sidebar Header */}
        <div className="flex items-center justify-between p-2 border-b"
          style={{ borderColor: ISA101Colors.toolbarBorder }}>
          {!sidebarCollapsed && (
            <h3 className="font-bold text-sm" style={{ color: ISA101Colors.text }}>
              Components
            </h3>
          )}
          <button
            onClick={() => setSidebarCollapsed(!sidebarCollapsed)}
            className="p-1 rounded hover:bg-gray-300"
            style={{ color: ISA101Colors.text }}
          >
            {sidebarCollapsed ? <FaBars /> : <FaTimes />}
          </button>
        </div>

        {/* Component Categories */}
        {!sidebarCollapsed && (
          <div className="flex-1 overflow-y-auto">
            {componentCategories.map((category, idx) => (
              <div key={category.name} className="border-b"
                style={{ borderColor: ISA101Colors.toolbarBorder }}>
                <button
                  onClick={() => setActiveCategory(activeCategory === idx ? -1 : idx)}
                  className="w-full flex items-center justify-between p-3 hover:bg-gray-300"
                  style={{ 
                    backgroundColor: activeCategory === idx ? ISA101Colors.categoryHeader : 'transparent',
                    color: ISA101Colors.text 
                  }}
                >
                  <div className="flex items-center gap-2">
                    <span className="text-sm">{category.icon}</span>
                    <span className="text-sm font-medium">{category.name}</span>
                  </div>
                  <span className="text-xs">
                    {activeCategory === idx ? '−' : '+'}
                  </span>
                </button>
                
                {activeCategory === idx && (
                  <div className="grid grid-cols-2 gap-1 p-2">
                    {category.components.map(comp => (
                      <button
                        key={comp.type}
                        onClick={() => addComponent(comp.type)}
                        className="p-2 text-xs border rounded hover:bg-gray-200 flex flex-col items-center gap-1"
                        style={{ 
                          backgroundColor: ISA101Colors.buttonBg,
                          borderColor: ISA101Colors.toolbarBorder,
                          color: ISA101Colors.text 
                        }}
                      >
                        <span className="font-medium">{comp.label}</span>
                      </button>
                    ))}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Main Canvas Area */}
      <div className="flex-1 flex flex-col">
        {/* Top Toolbar - ISA-101 Style */}
        <div className="h-12 flex items-center justify-between px-4 border-b-2"
          style={{ 
            backgroundColor: ISA101Colors.toolbarBg,
            borderColor: ISA101Colors.toolbarBorder 
          }}>
          <div className="flex items-center gap-2">
            {/* Display Options */}
            <button
              onClick={() => setShowGrid(!showGrid)}
              className={`p-2 rounded text-sm ${showGrid ? 'bg-blue-100' : ''}`}
              style={{ color: ISA101Colors.text }}
              title="Toggle Grid"
            >
              {showGrid ? <FaEye /> : <FaEyeSlash />}
            </button>
            
            <button
              onClick={() => setSnapToGrid(!snapToGrid)}
              className={`p-2 rounded text-sm ${snapToGrid ? 'bg-blue-100' : ''}`}
              style={{ color: ISA101Colors.text }}
              title="Snap to Grid"
            >
              <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <rect x="1" y="1" width="2" height="2" />
                <rect x="7" y="1" width="2" height="2" />
                <rect x="13" y="1" width="2" height="2" />
                <rect x="1" y="7" width="2" height="2" />
                <rect x="7" y="7" width="2" height="2" />
                <rect x="13" y="7" width="2" height="2" />
                <rect x="1" y="13" width="2" height="2" />
                <rect x="7" y="13" width="2" height="2" />
                <rect x="13" y="13" width="2" height="2" />
              </svg>
            </button>

            <div className="w-px h-6 bg-gray-400 mx-1" />

            {/* Component Actions */}
            {selectedId && (
              <>
                <button
                  onClick={() => {
                    const comp = components.find(c => c.id === selectedId)
                    if (comp) {
                      updateComponent(selectedId, { locked: !comp.locked })
                    }
                  }}
                  className="p-2 rounded text-sm"
                  style={{ color: ISA101Colors.text }}
                  title="Lock/Unlock"
                >
                  {components.find(c => c.id === selectedId)?.locked ? <FaLock /> : <FaUnlock />}
                </button>
                
                <button
                  onClick={deleteSelected}
                  className="p-2 rounded text-sm text-red-600"
                  title="Delete"
                >
                  <FaTrash />
                </button>
              </>
            )}
          </div>

          <div className="flex items-center gap-2">
            {/* Status Indicators */}
            <div className="flex items-center gap-2 text-xs"
              style={{ color: ISA101Colors.textSecondary }}>
              <span>Grid: {gridSize}px</span>
              <span>•</span>
              <span>{components.length} components</span>
            </div>

            <div className="w-px h-6 bg-gray-400 mx-1" />

            {/* Action Buttons */}
            <button className="px-3 py-1 text-sm border rounded"
              style={{ 
                backgroundColor: ISA101Colors.buttonBg,
                borderColor: ISA101Colors.toolbarBorder,
                color: ISA101Colors.text 
              }}>
              <FaSave className="inline mr-1" /> Save
            </button>
            
            <button className="px-3 py-1 text-sm border rounded"
              style={{ 
                backgroundColor: ISA101Colors.buttonBg,
                borderColor: ISA101Colors.toolbarBorder,
                color: ISA101Colors.text 
              }}>
              <FaPlay className="inline mr-1" /> Preview
            </button>
          </div>
        </div>

        {/* Canvas */}
        <div id="canvas-container" className="flex-1 relative overflow-hidden"
          style={{ backgroundColor: ISA101Colors.background }}>
          <Stage
            ref={stageRef}
            width={stageSize.width}
            height={stageSize.height}
            onMouseDown={(e) => {
              // Deselect when clicking on empty space
              const clickedOnEmpty = e.target === e.target.getStage()
              if (clickedOnEmpty) {
                setSelectedId(null)
              }
            }}
          >
            <Layer>
              {/* Grid */}
              {showGrid && (
                <>
                  {/* Vertical lines */}
                  {Array.from({ length: Math.ceil(stageSize.width / gridSize) }, (_, i) => (
                    <Line
                      key={`v-${i}`}
                      points={[i * gridSize, 0, i * gridSize, stageSize.height]}
                      stroke={ISA101Colors.gridLine}
                      strokeWidth={1}
                      opacity={0.5}
                    />
                  ))}
                  {/* Horizontal lines */}
                  {Array.from({ length: Math.ceil(stageSize.height / gridSize) }, (_, i) => (
                    <Line
                      key={`h-${i}`}
                      points={[0, i * gridSize, stageSize.width, i * gridSize]}
                      stroke={ISA101Colors.gridLine}
                      strokeWidth={1}
                      opacity={0.5}
                    />
                  ))}
                </>
              )}

              {/* Render all components */}
              {components.map(component => renderComponent(component))}
            </Layer>
          </Stage>

          {/* Properties Panel (when component selected) */}
          {selectedId && (
            <div className="absolute right-0 top-0 h-full w-80 shadow-lg overflow-y-auto"
              style={{ 
                backgroundColor: ISA101Colors.toolbarBg,
                borderLeft: `2px solid ${ISA101Colors.toolbarBorder}`
              }}>
              <div className="p-4">
                <div className="flex items-center justify-between mb-4">
                  <h3 className="font-bold" style={{ color: ISA101Colors.text }}>
                    Properties
                  </h3>
                  <button
                    onClick={() => setSelectedId(null)}
                    className="p-1 rounded hover:bg-gray-300"
                    style={{ color: ISA101Colors.text }}
                  >
                    <FaTimes />
                  </button>
                </div>

                {(() => {
                  const component = components.find(c => c.id === selectedId)
                  if (!component) return null

                  return (
                    <div className="space-y-4">
                      {/* Basic Properties */}
                      <div>
                        <h4 className="font-medium text-sm mb-2" 
                          style={{ color: ISA101Colors.text }}>
                          General
                        </h4>
                        <div className="space-y-2">
                          <div>
                            <label className="text-xs" style={{ color: ISA101Colors.textSecondary }}>
                              ID
                            </label>
                            <input
                              type="text"
                              value={component.id}
                              disabled
                              className="w-full px-2 py-1 text-sm border rounded bg-gray-100"
                              style={{ borderColor: ISA101Colors.toolbarBorder }}
                            />
                          </div>
                          <div>
                            <label className="text-xs" style={{ color: ISA101Colors.textSecondary }}>
                              Type
                            </label>
                            <input
                              type="text"
                              value={component.type}
                              disabled
                              className="w-full px-2 py-1 text-sm border rounded bg-gray-100"
                              style={{ borderColor: ISA101Colors.toolbarBorder }}
                            />
                          </div>
                        </div>
                      </div>

                      {/* Position & Size */}
                      <div>
                        <h4 className="font-medium text-sm mb-2" 
                          style={{ color: ISA101Colors.text }}>
                          Position & Size
                        </h4>
                        <div className="grid grid-cols-2 gap-2">
                          <div>
                            <label className="text-xs" style={{ color: ISA101Colors.textSecondary }}>
                              X
                            </label>
                            <input
                              type="number"
                              value={component.position.x}
                              onChange={(e) => updateComponent(component.id, {
                                position: { ...component.position, x: parseInt(e.target.value) || 0 }
                              })}
                              className="w-full px-2 py-1 text-sm border rounded"
                              style={{ 
                                backgroundColor: ISA101Colors.buttonBg,
                                borderColor: ISA101Colors.toolbarBorder 
                              }}
                            />
                          </div>
                          <div>
                            <label className="text-xs" style={{ color: ISA101Colors.textSecondary }}>
                              Y
                            </label>
                            <input
                              type="number"
                              value={component.position.y}
                              onChange={(e) => updateComponent(component.id, {
                                position: { ...component.position, y: parseInt(e.target.value) || 0 }
                              })}
                              className="w-full px-2 py-1 text-sm border rounded"
                              style={{ 
                                backgroundColor: ISA101Colors.buttonBg,
                                borderColor: ISA101Colors.toolbarBorder 
                              }}
                            />
                          </div>
                          <div>
                            <label className="text-xs" style={{ color: ISA101Colors.textSecondary }}>
                              Width
                            </label>
                            <input
                              type="number"
                              value={component.size.width}
                              onChange={(e) => updateComponent(component.id, {
                                size: { ...component.size, width: parseInt(e.target.value) || 50 }
                              })}
                              className="w-full px-2 py-1 text-sm border rounded"
                              style={{ 
                                backgroundColor: ISA101Colors.buttonBg,
                                borderColor: ISA101Colors.toolbarBorder 
                              }}
                            />
                          </div>
                          <div>
                            <label className="text-xs" style={{ color: ISA101Colors.textSecondary }}>
                              Height
                            </label>
                            <input
                              type="number"
                              value={component.size.height}
                              onChange={(e) => updateComponent(component.id, {
                                size: { ...component.size, height: parseInt(e.target.value) || 50 }
                              })}
                              className="w-full px-2 py-1 text-sm border rounded"
                              style={{ 
                                backgroundColor: ISA101Colors.buttonBg,
                                borderColor: ISA101Colors.toolbarBorder 
                              }}
                            />
                          </div>
                        </div>
                      </div>

                      {/* Component-specific Properties */}
                      <div>
                        <h4 className="font-medium text-sm mb-2" 
                          style={{ color: ISA101Colors.text }}>
                          Component Properties
                        </h4>
                        <div className="space-y-2">
                          {component.type === 'tank' && (
                            <>
                              <div>
                                <label className="text-xs" style={{ color: ISA101Colors.textSecondary }}>
                                  Tag Name
                                </label>
                                <input
                                  type="text"
                                  value={component.properties.tagName}
                                  onChange={(e) => updateComponent(component.id, {
                                    properties: { ...component.properties, tagName: e.target.value }
                                  })}
                                  className="w-full px-2 py-1 text-sm border rounded"
                                  style={{ 
                                    backgroundColor: ISA101Colors.buttonBg,
                                    borderColor: ISA101Colors.toolbarBorder 
                                  }}
                                />
                              </div>
                              <div>
                                <label className="text-xs" style={{ color: ISA101Colors.textSecondary }}>
                                  Current Level
                                </label>
                                <input
                                  type="number"
                                  value={component.properties.currentLevel}
                                  onChange={(e) => updateComponent(component.id, {
                                    properties: { ...component.properties, currentLevel: parseFloat(e.target.value) || 0 }
                                  })}
                                  className="w-full px-2 py-1 text-sm border rounded"
                                  style={{ 
                                    backgroundColor: ISA101Colors.buttonBg,
                                    borderColor: ISA101Colors.toolbarBorder 
                                  }}
                                />
                              </div>
                              <div>
                                <label className="text-xs" style={{ color: ISA101Colors.textSecondary }}>
                                  Material Type
                                </label>
                                <select
                                  value={component.properties.materialType}
                                  onChange={(e) => updateComponent(component.id, {
                                    properties: { ...component.properties, materialType: e.target.value }
                                  })}
                                  className="w-full px-2 py-1 text-sm border rounded"
                                  style={{ 
                                    backgroundColor: ISA101Colors.buttonBg,
                                    borderColor: ISA101Colors.toolbarBorder 
                                  }}
                                >
                                  <option value="water">Water</option>
                                  <option value="chemical">Chemical</option>
                                  <option value="hot">Hot Liquid</option>
                                  <option value="oil">Oil</option>
                                </select>
                              </div>
                            </>
                          )}
                          
                          {component.type === 'pump' && (
                            <>
                              <div>
                                <label className="text-xs" style={{ color: ISA101Colors.textSecondary }}>
                                  Tag Name
                                </label>
                                <input
                                  type="text"
                                  value={component.properties.tagName}
                                  onChange={(e) => updateComponent(component.id, {
                                    properties: { ...component.properties, tagName: e.target.value }
                                  })}
                                  className="w-full px-2 py-1 text-sm border rounded"
                                  style={{ 
                                    backgroundColor: ISA101Colors.buttonBg,
                                    borderColor: ISA101Colors.toolbarBorder 
                                  }}
                                />
                              </div>
                              <div>
                                <label className="text-xs" style={{ color: ISA101Colors.textSecondary }}>
                                  Status
                                </label>
                                <select
                                  value={component.properties.status}
                                  onChange={(e) => updateComponent(component.id, {
                                    properties: { ...component.properties, status: e.target.value }
                                  })}
                                  className="w-full px-2 py-1 text-sm border rounded"
                                  style={{ 
                                    backgroundColor: ISA101Colors.buttonBg,
                                    borderColor: ISA101Colors.toolbarBorder 
                                  }}
                                >
                                  <option value="stopped">Stopped</option>
                                  <option value="running">Running</option>
                                  <option value="fault">Fault</option>
                                  <option value="transitioning">Transitioning</option>
                                </select>
                              </div>
                              <div>
                                <label className="text-xs" style={{ color: ISA101Colors.textSecondary }}>
                                  Control Mode
                                </label>
                                <select
                                  value={component.properties.controlMode}
                                  onChange={(e) => updateComponent(component.id, {
                                    properties: { ...component.properties, controlMode: e.target.value }
                                  })}
                                  className="w-full px-2 py-1 text-sm border rounded"
                                  style={{ 
                                    backgroundColor: ISA101Colors.buttonBg,
                                    borderColor: ISA101Colors.toolbarBorder 
                                  }}
                                >
                                  <option value="auto">Auto</option>
                                  <option value="manual">Manual</option>
                                  <option value="cascade">Cascade</option>
                                </select>
                              </div>
                            </>
                          )}
                          
                          {component.type === 'valve' && (
                            <>
                              <div>
                                <label className="text-xs" style={{ color: ISA101Colors.textSecondary }}>
                                  Tag Name
                                </label>
                                <input
                                  type="text"
                                  value={component.properties.tagName}
                                  onChange={(e) => updateComponent(component.id, {
                                    properties: { ...component.properties, tagName: e.target.value }
                                  })}
                                  className="w-full px-2 py-1 text-sm border rounded"
                                  style={{ 
                                    backgroundColor: ISA101Colors.buttonBg,
                                    borderColor: ISA101Colors.toolbarBorder 
                                  }}
                                />
                              </div>
                              <div>
                                <label className="text-xs" style={{ color: ISA101Colors.textSecondary }}>
                                  Position (%)
                                </label>
                                <input
                                  type="number"
                                  min="0"
                                  max="100"
                                  value={component.properties.position}
                                  onChange={(e) => updateComponent(component.id, {
                                    properties: { ...component.properties, position: parseInt(e.target.value) || 0 }
                                  })}
                                  className="w-full px-2 py-1 text-sm border rounded"
                                  style={{ 
                                    backgroundColor: ISA101Colors.buttonBg,
                                    borderColor: ISA101Colors.toolbarBorder 
                                  }}
                                />
                              </div>
                              <div>
                                <label className="text-xs" style={{ color: ISA101Colors.textSecondary }}>
                                  Valve Type
                                </label>
                                <select
                                  value={component.properties.valveType}
                                  onChange={(e) => updateComponent(component.id, {
                                    properties: { ...component.properties, valveType: e.target.value }
                                  })}
                                  className="w-full px-2 py-1 text-sm border rounded"
                                  style={{ 
                                    backgroundColor: ISA101Colors.buttonBg,
                                    borderColor: ISA101Colors.toolbarBorder 
                                  }}
                                >
                                  <option value="gate">Gate</option>
                                  <option value="ball">Ball</option>
                                  <option value="butterfly">Butterfly</option>
                                  <option value="control">Control</option>
                                </select>
                              </div>
                            </>
                          )}
                        </div>
                      </div>

                      {/* Signal Bindings */}
                      <div>
                        <h4 className="font-medium text-sm mb-2 flex items-center justify-between" 
                          style={{ color: ISA101Colors.text }}>
                          Signal Bindings
                          <button
                            onClick={() => setShowBindingPanel(!showBindingPanel)}
                            className="p-1 rounded text-xs flex items-center gap-1"
                            style={{ 
                              backgroundColor: connected ? ISA101Colors.running : ISA101Colors.stopped,
                              color: '#FFFFFF'
                            }}
                          >
                            {connected ? <FaLink /> : <FaUnlink />}
                            {connected ? 'Connected' : 'Disconnected'}
                          </button>
                        </h4>
                        
                        {showBindingPanel && connected && (
                          <div className="space-y-2">
                            {/* Add binding for currentLevel/value based on component type */}
                            {(component.type === 'tank' || component.type === 'gauge') && (
                              <div>
                                <label className="text-xs" style={{ color: ISA101Colors.textSecondary }}>
                                  {component.type === 'tank' ? 'Level Signal' : 'Value Signal'}
                                </label>
                                <select
                                  value={component.bindings?.find(b => b.property === (component.type === 'tank' ? 'currentLevel' : 'currentValue'))?.signal || ''}
                                  onChange={(e) => {
                                    const newBindings = component.bindings?.filter(b => b.property !== (component.type === 'tank' ? 'currentLevel' : 'currentValue')) || []
                                    if (e.target.value) {
                                      newBindings.push({
                                        property: component.type === 'tank' ? 'currentLevel' : 'currentValue',
                                        signal: e.target.value
                                      })
                                    }
                                    updateComponent(component.id, { bindings: newBindings })
                                  }}
                                  className="w-full px-2 py-1 text-sm border rounded"
                                  style={{ 
                                    backgroundColor: ISA101Colors.buttonBg,
                                    borderColor: ISA101Colors.toolbarBorder 
                                  }}
                                >
                                  <option value="">-- No Binding --</option>
                                  {Array.from(signals.keys()).map(signal => (
                                    <option key={signal} value={signal}>{signal}</option>
                                  ))}
                                </select>
                              </div>
                            )}
                            
                            {(component.type === 'pump' || component.type === 'valve' || component.type === 'motor') && (
                              <>
                                <div>
                                  <label className="text-xs" style={{ color: ISA101Colors.textSecondary }}>
                                    Status Signal
                                  </label>
                                  <select
                                    value={component.bindings?.find(b => b.property === 'status')?.signal || ''}
                                    onChange={(e) => {
                                      const newBindings = component.bindings?.filter(b => b.property !== 'status') || []
                                      if (e.target.value) {
                                        newBindings.push({
                                          property: 'status',
                                          signal: e.target.value,
                                          transform: component.type === 'pump' ? 
                                            "value ? 'running' : 'stopped'" : 
                                            component.type === 'valve' ?
                                            "value > 50 ? 'open' : 'closed'" :
                                            "value ? 'running' : 'stopped'"
                                        })
                                      }
                                      updateComponent(component.id, { bindings: newBindings })
                                    }}
                                    className="w-full px-2 py-1 text-sm border rounded"
                                    style={{ 
                                      backgroundColor: ISA101Colors.buttonBg,
                                      borderColor: ISA101Colors.toolbarBorder 
                                    }}
                                  >
                                    <option value="">-- No Binding --</option>
                                    {Array.from(signals.keys()).map(signal => (
                                      <option key={signal} value={signal}>{signal}</option>
                                    ))}
                                  </select>
                                </div>
                                
                                {component.type === 'valve' && (
                                  <div>
                                    <label className="text-xs" style={{ color: ISA101Colors.textSecondary }}>
                                      Position Signal
                                    </label>
                                    <select
                                      value={component.bindings?.find(b => b.property === 'position')?.signal || ''}
                                      onChange={(e) => {
                                        const newBindings = component.bindings?.filter(b => b.property !== 'position') || []
                                        if (e.target.value) {
                                          newBindings.push({
                                            property: 'position',
                                            signal: e.target.value
                                          })
                                        }
                                        updateComponent(component.id, { bindings: newBindings })
                                      }}
                                      className="w-full px-2 py-1 text-sm border rounded"
                                      style={{ 
                                        backgroundColor: ISA101Colors.buttonBg,
                                        borderColor: ISA101Colors.toolbarBorder 
                                      }}
                                    >
                                      <option value="">-- No Binding --</option>
                                      {Array.from(signals.keys()).map(signal => (
                                        <option key={signal} value={signal}>{signal}</option>
                                      ))}
                                    </select>
                                  </div>
                                )}
                              </>
                            )}

                            {/* Alarm bindings for components that support them */}
                            {(component.type === 'tank' || component.type === 'gauge') && (
                              <div className="pt-2 border-t" style={{ borderColor: ISA101Colors.toolbarBorder }}>
                                <label className="text-xs font-medium" style={{ color: ISA101Colors.textSecondary }}>
                                  Alarm Signals
                                </label>
                                <div className="space-y-1 mt-1">
                                  <div>
                                    <label className="text-xs" style={{ color: ISA101Colors.textSecondary }}>
                                      High Alarm
                                    </label>
                                    <select
                                      value={component.bindings?.find(b => b.property === 'alarmHigh')?.signal || ''}
                                      onChange={(e) => {
                                        const newBindings = component.bindings?.filter(b => b.property !== 'alarmHigh') || []
                                        if (e.target.value) {
                                          newBindings.push({
                                            property: 'alarmHigh',
                                            signal: e.target.value
                                          })
                                        }
                                        updateComponent(component.id, { bindings: newBindings })
                                      }}
                                      className="w-full px-2 py-1 text-sm border rounded"
                                      style={{ 
                                        backgroundColor: ISA101Colors.buttonBg,
                                        borderColor: ISA101Colors.toolbarBorder 
                                      }}
                                    >
                                      <option value="">-- No Binding --</option>
                                      {Array.from(signals.keys()).map(signal => (
                                        <option key={signal} value={signal}>{signal}</option>
                                      ))}
                                    </select>
                                  </div>
                                </div>
                              </div>
                            )}

                            {/* Current bindings list */}
                            {component.bindings?.length > 0 && (
                              <div className="pt-2 border-t" style={{ borderColor: ISA101Colors.toolbarBorder }}>
                                <label className="text-xs font-medium" style={{ color: ISA101Colors.textSecondary }}>
                                  Active Bindings
                                </label>
                                <div className="space-y-1 mt-1">
                                  {component.bindings.map((binding, idx) => (
                                    <div key={idx} className="flex items-center justify-between text-xs p-1 rounded"
                                      style={{ backgroundColor: ISA101Colors.background }}>
                                      <span>{binding.property} → {binding.signal}</span>
                                      <button
                                        onClick={() => {
                                          updateComponent(component.id, {
                                            bindings: component.bindings.filter((_, i) => i !== idx)
                                          })
                                        }}
                                        className="text-red-600 hover:text-red-800"
                                      >
                                        <FaTimes />
                                      </button>
                                    </div>
                                  ))}
                                </div>
                              </div>
                            )}
                          </div>
                        )}
                        
                        {!connected && (
                          <div className="text-xs p-2 rounded" 
                            style={{ 
                              backgroundColor: ISA101Colors.alarmLow + '20',
                              color: ISA101Colors.textSecondary 
                            }}>
                            Connect to PETRA to enable signal bindings
                          </div>
                        )}
                      </div>
                    </div>
                  )
                })()}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
