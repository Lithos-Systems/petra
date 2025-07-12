// petra-designer/src/components/Sidebar.tsx
import React, { useState, useMemo } from 'react'
import { 
  FaSearch, 
  FaChevronRight, 
  FaChevronDown, 
  FaStar,
  FaInfoCircle,
  FaQuestionCircle,
  FaCube,
  FaNetworkWired,
  FaClock,
  FaChartLine,
  FaCogs,
  FaExchangeAlt,
  FaCalculator,
  FaWifi
} from 'react-icons/fa'
import { PETRA_BLOCKS } from '@/nodes/BlockNode'

// Complete PETRA block library
const componentCategories = [
  {
    id: 'logic',
    name: 'Logic Gates',
    icon: <FaNetworkWired className="w-4 h-4" />,
    components: [
      {
        id: 'and',
        type: 'block',
        blockType: 'AND',
        label: 'AND Gate',
        description: 'All inputs must be true (2-8 inputs)',
        tags: ['logic', 'and', 'boolean', 'gate'],
        complexity: 'basic'
      },
      {
        id: 'or',
        type: 'block',
        blockType: 'OR',
        label: 'OR Gate',
        description: 'At least one input true (2-8 inputs)',
        tags: ['logic', 'or', 'boolean', 'gate'],
        complexity: 'basic'
      },
      {
        id: 'not',
        type: 'block',
        blockType: 'NOT',
        label: 'NOT Gate',
        description: 'Inverts boolean input',
        tags: ['logic', 'not', 'invert', 'boolean'],
        complexity: 'basic'
      },
      {
        id: 'xor',
        type: 'block',
        blockType: 'XOR',
        label: 'XOR Gate',
        description: 'Odd number of true inputs',
        tags: ['logic', 'xor', 'exclusive', 'boolean'],
        complexity: 'basic'
      }
    ]
  },
  {
    id: 'compare',
    name: 'Comparison',
    icon: <FaExchangeAlt className="w-4 h-4" />,
    components: [
      {
        id: 'gt',
        type: 'block',
        blockType: 'GT',
        label: 'Greater Than',
        description: 'a > b comparison',
        tags: ['compare', 'greater', 'numeric'],
        complexity: 'basic'
      },
      {
        id: 'lt',
        type: 'block',
        blockType: 'LT',
        label: 'Less Than',
        description: 'a < b comparison',
        tags: ['compare', 'less', 'numeric'],
        complexity: 'basic'
      },
      {
        id: 'gte',
        type: 'block',
        blockType: 'GTE',
        label: 'Greater or Equal',
        description: 'a ≥ b comparison',
        tags: ['compare', 'greater', 'equal', 'numeric'],
        complexity: 'basic'
      },
      {
        id: 'lte',
        type: 'block',
        blockType: 'LTE',
        label: 'Less or Equal',
        description: 'a ≤ b comparison',
        tags: ['compare', 'less', 'equal', 'numeric'],
        complexity: 'basic'
      },
      {
        id: 'eq',
        type: 'block',
        blockType: 'EQ',
        label: 'Equal',
        description: 'a = b comparison',
        tags: ['compare', 'equal', 'numeric'],
        complexity: 'basic'
      },
      {
        id: 'neq',
        type: 'block',
        blockType: 'NEQ',
        label: 'Not Equal',
        description: 'a ≠ b comparison',
        tags: ['compare', 'not equal', 'numeric'],
        complexity: 'basic'
      }
    ]
  },
  {
    id: 'math',
    name: 'Math',
    icon: <FaCalculator className="w-4 h-4" />,
    components: [
      {
        id: 'add',
        type: 'block',
        blockType: 'ADD',
        label: 'Add',
        description: 'Addition (a + b)',
        tags: ['math', 'add', 'arithmetic'],
        complexity: 'basic'
      },
      {
        id: 'sub',
        type: 'block',
        blockType: 'SUB',
        label: 'Subtract',
        description: 'Subtraction (a - b)',
        tags: ['math', 'subtract', 'arithmetic'],
        complexity: 'basic'
      },
      {
        id: 'mul',
        type: 'block',
        blockType: 'MUL',
        label: 'Multiply',
        description: 'Multiplication (a × b)',
        tags: ['math', 'multiply', 'arithmetic'],
        complexity: 'basic'
      },
      {
        id: 'div',
        type: 'block',
        blockType: 'DIV',
        label: 'Divide',
        description: 'Division (a ÷ b)',
        tags: ['math', 'divide', 'arithmetic'],
        complexity: 'basic'
      }
    ]
  },
  {
    id: 'timer',
    name: 'Timers',
    icon: <FaClock className="w-4 h-4" />,
    components: [
      {
        id: 'on_delay',
        type: 'block',
        blockType: 'ON_DELAY',
        label: 'On Delay Timer',
        description: 'Delays turn-on signal',
        tags: ['timer', 'delay', 'on', 'TON'],
        complexity: 'intermediate'
      },
      {
        id: 'off_delay',
        type: 'block',
        blockType: 'OFF_DELAY',
        label: 'Off Delay Timer',
        description: 'Delays turn-off signal',
        tags: ['timer', 'delay', 'off', 'TOF'],
        complexity: 'intermediate'
      },
      {
        id: 'pulse',
        type: 'block',
        blockType: 'PULSE',
        label: 'Pulse Timer',
        description: 'Generates timed pulse',
        tags: ['timer', 'pulse', 'TP'],
        complexity: 'intermediate'
      }
    ]
  },
  {
    id: 'data',
    name: 'Data Processing',
    icon: <FaCube className="w-4 h-4" />,
    components: [
      {
        id: 'scale',
        type: 'block',
        blockType: 'SCALE',
        label: 'Scale',
        description: 'Scales input range to output range',
        tags: ['data', 'scale', 'range', 'convert'],
        complexity: 'intermediate'
      },
      {
        id: 'limit',
        type: 'block',
        blockType: 'LIMIT',
        label: 'Limit',
        description: 'Limits value to min/max range',
        tags: ['data', 'limit', 'clamp', 'range'],
        complexity: 'basic'
      },
      {
        id: 'select',
        type: 'block',
        blockType: 'SELECT',
        label: 'Select',
        description: 'Selects between two inputs',
        tags: ['data', 'select', 'switch', 'mux'],
        complexity: 'basic'
      },
      {
        id: 'mux',
        type: 'block',
        blockType: 'MUX',
        label: 'Multiplexer',
        description: '4-to-1 multiplexer',
        tags: ['data', 'mux', 'select', 'switch'],
        complexity: 'intermediate'
      },
      {
        id: 'demux',
        type: 'block',
        blockType: 'DEMUX',
        label: 'Demultiplexer',
        description: '1-to-4 demultiplexer',
        tags: ['data', 'demux', 'distribute'],
        complexity: 'intermediate'
      }
    ]
  },
  {
    id: 'control',
    name: 'Control',
    icon: <FaCogs className="w-4 h-4" />,
    components: [
      {
        id: 'pid',
        type: 'block',
        blockType: 'PID',
        label: 'PID Controller',
        description: 'Proportional-Integral-Derivative control',
        tags: ['control', 'pid', 'loop', 'process'],
        complexity: 'advanced'
      },
      {
        id: 'ramp',
        type: 'block',
        blockType: 'RAMP',
        label: 'Ramp',
        description: 'Ramps output up/down at set rate',
        tags: ['control', 'ramp', 'rate', 'limit'],
        complexity: 'intermediate'
      },
      {
        id: 'leadlag',
        type: 'block',
        blockType: 'LEADLAG',
        label: 'Lead/Lag',
        description: 'Lead-lag compensator',
        tags: ['control', 'lead', 'lag', 'compensator'],
        complexity: 'advanced'
      }
    ]
  },
  {
    id: 'statistics',
    name: 'Statistics',
    icon: <FaChartLine className="w-4 h-4" />,
    components: [
      {
        id: 'avg',
        type: 'block',
        blockType: 'AVG',
        label: 'Average',
        description: 'Moving average calculator',
        tags: ['stats', 'average', 'mean', 'filter'],
        complexity: 'intermediate'
      },
      {
        id: 'minmax',
        type: 'block',
        blockType: 'MIN_MAX',
        label: 'Min/Max',
        description: 'Tracks minimum and maximum values',
        tags: ['stats', 'min', 'max', 'range'],
        complexity: 'intermediate'
      },
      {
        id: 'stddev',
        type: 'block',
        blockType: 'STDDEV',
        label: 'Std Deviation',
        description: 'Standard deviation calculator',
        tags: ['stats', 'stddev', 'variance', 'statistics'],
        complexity: 'advanced'
      }
    ]
  },
  {
    id: 'communication',
    name: 'Communication',
    icon: <FaWifi className="w-4 h-4" />,
    components: [
      {
        id: 'mqtt',
        type: 'mqtt',
        label: 'MQTT Client',
        description: 'MQTT broker connection',
        tags: ['mqtt', 'iot', 'broker', 'publish', 'subscribe'],
        complexity: 'advanced'
      },
      {
        id: 'modbus',
        type: 'modbus',
        label: 'Modbus',
        description: 'Modbus TCP/RTU client',
        tags: ['modbus', 'industrial', 'tcp', 'rtu'],
        complexity: 'advanced'
      },
      {
        id: 's7',
        type: 's7',
        label: 'Siemens S7',
        description: 'S7 PLC communication',
        tags: ['s7', 'siemens', 'plc', 'industrial'],
        complexity: 'advanced'
      }
    ]
  },
  {
    id: 'signals',
    name: 'Signals',
    icon: <FaExchangeAlt className="w-4 h-4" />,
    components: [
      {
        id: 'signal',
        type: 'signal',
        label: 'Signal',
        description: 'Signal input/output point',
        tags: ['signal', 'io', 'variable'],
        complexity: 'basic'
      }
    ]
  }
]

interface SidebarProps {
  className?: string
}

export default function Sidebar({ className = '' }: SidebarProps) {
  const [searchTerm, setSearchTerm] = useState('')
  const [collapsedCategories, setCollapsedCategories] = useState<Set<string>>(new Set())
  const [favoriteComponents, setFavoriteComponents] = useState<Set<string>>(new Set())

  const handleDragStart = (e: React.DragEvent, component: any) => {
    e.dataTransfer.setData('application/reactflow', component.type)
    e.dataTransfer.effectAllowed = 'move'
    
    // Add custom data for blocks
    if (component.type === 'block') {
      const customData = {
        blockType: component.blockType,
        label: component.label,
        inputCount: component.blockType === 'AND' || component.blockType === 'OR' ? 2 : undefined
      }
      e.dataTransfer.setData('custom-data', JSON.stringify(customData))
    }
  }

  const toggleCategory = (categoryId: string) => {
    setCollapsedCategories(prev => {
      const updated = new Set(prev)
      if (updated.has(categoryId)) {
        updated.delete(categoryId)
      } else {
        updated.add(categoryId)
      }
      return updated
    })
  }

  const toggleFavorite = (componentId: string) => {
    setFavoriteComponents(prev => {
      const updated = new Set(prev)
      if (updated.has(componentId)) {
        updated.delete(componentId)
      } else {
        updated.add(componentId)
      }
      return updated
    })
  }

  const filteredCategories = useMemo(() => {
    return componentCategories.map(category => ({
      ...category,
      components: category.components.filter(component => {
        const matchesSearch = searchTerm === '' || 
          component.label.toLowerCase().includes(searchTerm.toLowerCase()) ||
          component.description?.toLowerCase().includes(searchTerm.toLowerCase()) ||
          component.tags?.some(tag => tag.toLowerCase().includes(searchTerm.toLowerCase()))
        
        return matchesSearch
      })
    })).filter(category => category.components.length > 0)
  }, [searchTerm])

  return (
    <div className={`isa101-sidebar w-80 h-full ${className}`}>
      {/* Header */}
      <div className="isa101-panel-header">
        <span className="text-sm font-medium">PETRA Block Library</span>
      </div>

      {/* Search */}
      <div className="p-3 border-b border-[#606060]">
        <div className="relative">
          <FaSearch className="absolute left-2 top-2.5 w-3 h-3 text-[#606060]" />
          <input
            type="text"
            placeholder="Search blocks..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="isa101-input w-full pl-7 pr-3 py-2 text-xs"
            style={{
              backgroundColor: '#FFFFFF',
              border: '1px solid #606060',
              color: '#000000'
            }}
          />
        </div>
      </div>

      {/* Block Categories */}
      <div className="flex-1 overflow-y-auto">
        {filteredCategories.map(category => (
          <div key={category.id} className="border-b border-[#606060]">
            {/* Category Header */}
            <button
              onClick={() => toggleCategory(category.id)}
              className="w-full px-3 py-2 flex items-center justify-between hover:bg-[#C8C8C8] transition-colors"
            >
              <div className="flex items-center gap-2">
                {category.icon}
                <span className="text-xs font-medium text-black">{category.name}</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-xs text-[#606060]">{category.components.length}</span>
                {collapsedCategories.has(category.id) ? (
                  <FaChevronRight className="w-2 h-2 text-[#606060]" />
                ) : (
                  <FaChevronDown className="w-2 h-2 text-[#606060]" />
                )}
              </div>
            </button>

            {/* Category Components */}
            {!collapsedCategories.has(category.id) && (
              <div className="px-1 pb-2">
                {category.components.map(component => (
                  <div
                    key={component.id}
                    draggable
                    onDragStart={(e) => handleDragStart(e, component)}
                    className="isa101-block-item group relative"
                  >
                    <div className="flex items-center gap-2 p-2 hover:bg-[#C8C8C8] cursor-move transition-colors">
                      <div className="flex-shrink-0 w-8 h-8 bg-gray-200 rounded flex items-center justify-center text-xs font-bold">
                        {component.blockType ? PETRA_BLOCKS[component.blockType]?.symbol : '?'}
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="text-xs font-medium text-black">{component.label}</div>
                        <div className="text-xs text-[#606060] truncate">{component.description}</div>
                      </div>
                      <button
                        onClick={(e) => {
                          e.stopPropagation()
                          toggleFavorite(component.id)
                        }}
                        className="opacity-0 group-hover:opacity-100 transition-opacity"
                      >
                        <FaStar
                          className={`w-3 h-3 ${
                            favoriteComponents.has(component.id)
                              ? 'text-yellow-500'
                              : 'text-gray-400 hover:text-yellow-500'
                          }`}
                        />
                      </button>
                    </div>
                    
                    {/* Complexity indicator */}
                    {component.complexity && (
                      <div className="absolute top-2 right-8 opacity-0 group-hover:opacity-100 transition-opacity">
                        {component.complexity === 'basic' && (
                          <div className="w-1 h-3 bg-green-500" title="Basic" />
                        )}
                        {component.complexity === 'intermediate' && (
                          <div className="flex gap-0.5">
                            <div className="w-1 h-3 bg-yellow-500" title="Intermediate" />
                            <div className="w-1 h-3 bg-yellow-500" />
                          </div>
                        )}
                        {component.complexity === 'advanced' && (
                          <div className="flex gap-0.5">
                            <div className="w-1 h-3 bg-red-500" title="Advanced" />
                            <div className="w-1 h-3 bg-red-500" />
                            <div className="w-1 h-3 bg-red-500" />
                          </div>
                        )}
                      </div>
                    )}
                  </div>
                ))}
              </div>
            )}
          </div>
        ))}
        
        {filteredCategories.length === 0 && (
          <div className="p-8 text-center">
            <FaSearch className="w-12 h-12 mx-auto mb-3 text-[#808080]" />
            <p className="text-sm text-[#606060]">No components found</p>
            <p className="text-xs mt-1 text-[#808080]">Try adjusting your search</p>
          </div>
        )}
      </div>
      
      {/* Footer Help */}
      <div className="isa101-panel-header border-t border-[#606060]">
        <div className="flex items-center justify-between text-xs text-[#606060]">
          <div className="flex items-center gap-1">
            <FaInfoCircle className="w-3 h-3" />
            <span>Drag blocks to canvas</span>
          </div>
          <button className="hover:text-[#404040]">
            <FaQuestionCircle className="w-3 h-3" />
          </button>
        </div>
      </div>
    </div>
  )
}
