import React, { useState, useMemo } from 'react'
import { FaSearch, FaChevronRight, FaChevronDown, FaInfoCircle, FaQuestionCircle, FaStar } from 'react-icons/fa'

// ISA-101 Compliant Block Graphics Components
const ISA101BlockGraphics = {
  // Logic Blocks
  AND: () => (
    <svg width="24" height="16" viewBox="0 0 24 16" className="isa101-block-icon">
      <path d="M2 2 L10 2 A6 6 0 0 1 10 14 L2 14 Z" fill="none" stroke="#000" strokeWidth="1"/>
      <text x="6" y="10" fontSize="8" textAnchor="middle" fill="#000">AND</text>
    </svg>
  ),
  
  OR: () => (
    <svg width="24" height="16" viewBox="0 0 24 16" className="isa101-block-icon">
      <path d="M2 2 Q8 2 10 8 Q8 14 2 14 Q6 8 2 2" fill="none" stroke="#000" strokeWidth="1"/>
      <text x="6" y="10" fontSize="8" textAnchor="middle" fill="#000">OR</text>
    </svg>
  ),
  
  NOT: () => (
    <svg width="24" height="16" viewBox="0 0 24 16" className="isa101-block-icon">
      <path d="M2 2 L2 14 L12 8 Z" fill="none" stroke="#000" strokeWidth="1"/>
      <circle cx="14" cy="8" r="2" fill="none" stroke="#000" strokeWidth="1"/>
      <text x="6" y="10" fontSize="6" textAnchor="middle" fill="#000">NOT</text>
    </svg>
  ),
  
  XOR: () => (
    <svg width="24" height="16" viewBox="0 0 24 16" className="isa101-block-icon">
      <path d="M2 2 Q8 2 10 8 Q8 14 2 14" fill="none" stroke="#000" strokeWidth="1"/>
      <path d="M1 2 Q7 2 9 8 Q7 14 1 14" fill="none" stroke="#000" strokeWidth="1"/>
      <text x="6" y="10" fontSize="7" textAnchor="middle" fill="#000">XOR</text>
    </svg>
  ),

  // Comparison Blocks
  GT: () => (
    <svg width="24" height="16" viewBox="0 0 24 16" className="isa101-block-icon">
      <rect x="2" y="2" width="16" height="12" fill="none" stroke="#000" strokeWidth="1"/>
      <text x="10" y="10" fontSize="8" textAnchor="middle" fill="#000">{">"}</text>
    </svg>
  ),
  
  LT: () => (
    <svg width="24" height="16" viewBox="0 0 24 16" className="isa101-block-icon">
      <rect x="2" y="2" width="16" height="12" fill="none" stroke="#000" strokeWidth="1"/>
      <text x="10" y="10" fontSize="8" textAnchor="middle" fill="#000">{"<"}</text>
    </svg>
  ),
  
  EQ: () => (
    <svg width="24" height="16" viewBox="0 0 24 16" className="isa101-block-icon">
      <rect x="2" y="2" width="16" height="12" fill="none" stroke="#000" strokeWidth="1"/>
      <text x="10" y="10" fontSize="8" textAnchor="middle" fill="#000">=</text>
    </svg>
  ),

  // Math Blocks
  ADD: () => (
    <svg width="24" height="16" viewBox="0 0 24 16" className="isa101-block-icon">
      <rect x="2" y="2" width="16" height="12" fill="none" stroke="#000" strokeWidth="1"/>
      <text x="10" y="10" fontSize="8" textAnchor="middle" fill="#000">+</text>
    </svg>
  ),
  
  SUB: () => (
    <svg width="24" height="16" viewBox="0 0 24 16" className="isa101-block-icon">
      <rect x="2" y="2" width="16" height="12" fill="none" stroke="#000" strokeWidth="1"/>
      <text x="10" y="10" fontSize="8" textAnchor="middle" fill="#000">-</text>
    </svg>
  ),
  
  MUL: () => (
    <svg width="24" height="16" viewBox="0 0 24 16" className="isa101-block-icon">
      <rect x="2" y="2" width="16" height="12" fill="none" stroke="#000" strokeWidth="1"/>
      <text x="10" y="10" fontSize="8" textAnchor="middle" fill="#000">×</text>
    </svg>
  ),
  
  DIV: () => (
    <svg width="24" height="16" viewBox="0 0 24 16" className="isa101-block-icon">
      <rect x="2" y="2" width="16" height="12" fill="none" stroke="#000" strokeWidth="1"/>
      <text x="10" y="10" fontSize="8" textAnchor="middle" fill="#000">÷</text>
    </svg>
  ),

  // Timer Blocks
  TIMER: () => (
    <svg width="24" height="16" viewBox="0 0 24 16" className="isa101-block-icon">
      <rect x="2" y="2" width="16" height="12" fill="none" stroke="#000" strokeWidth="1"/>
      <circle cx="10" cy="8" r="4" fill="none" stroke="#000" strokeWidth="1"/>
      <path d="M10 6 L10 8 L12 8" stroke="#000" strokeWidth="1"/>
      <text x="10" y="13" fontSize="5" textAnchor="middle" fill="#000">TMR</text>
    </svg>
  ),
  
  DELAY: () => (
    <svg width="24" height="16" viewBox="0 0 24 16" className="isa101-block-icon">
      <rect x="2" y="2" width="16" height="12" fill="none" stroke="#000" strokeWidth="1"/>
      <text x="10" y="6" fontSize="6" textAnchor="middle" fill="#000">DELAY</text>
      <text x="10" y="12" fontSize="5" textAnchor="middle" fill="#000">T#10s</text>
    </svg>
  ),

  // Control Blocks
  PID: () => (
    <svg width="24" height="16" viewBox="0 0 24 16" className="isa101-block-icon">
      <rect x="2" y="2" width="16" height="12" fill="none" stroke="#000" strokeWidth="1"/>
      <text x="10" y="7" fontSize="7" textAnchor="middle" fill="#000">PID</text>
      <text x="10" y="13" fontSize="4" textAnchor="middle" fill="#000">CTRL</text>
    </svg>
  ),
  
  CONTROLLER: () => (
    <svg width="24" height="16" viewBox="0 0 24 16" className="isa101-block-icon">
      <rect x="2" y="2" width="16" height="12" fill="none" stroke="#000" strokeWidth="1"/>
      <text x="10" y="10" fontSize="6" textAnchor="middle" fill="#000">CTRL</text>
    </svg>
  ),

  // I/O Blocks
  INPUT: () => (
    <svg width="24" height="16" viewBox="0 0 24 16" className="isa101-block-icon">
      <path d="M2 8 L8 2 L8 5 L18 5 L18 11 L8 11 L8 14 Z" fill="none" stroke="#000" strokeWidth="1"/>
      <text x="13" y="9" fontSize="5" textAnchor="middle" fill="#000">IN</text>
    </svg>
  ),
  
  OUTPUT: () => (
    <svg width="24" height="16" viewBox="0 0 24 16" className="isa101-block-icon">
      <path d="M18 8 L12 2 L12 5 L2 5 L2 11 L12 11 L12 14 Z" fill="none" stroke="#000" strokeWidth="1"/>
      <text x="7" y="9" fontSize="5" textAnchor="middle" fill="#000">OUT</text>
    </svg>
  ),

  // Communication Blocks
  MODBUS: () => (
    <svg width="24" height="16" viewBox="0 0 24 16" className="isa101-block-icon">
      <rect x="2" y="2" width="16" height="12" fill="none" stroke="#000" strokeWidth="1"/>
      <text x="10" y="8" fontSize="5" textAnchor="middle" fill="#000">MODBUS</text>
      <text x="10" y="12" fontSize="4" textAnchor="middle" fill="#000">TCP/RTU</text>
    </svg>
  ),
  
  S7: () => (
    <svg width="24" height="16" viewBox="0 0 24 16" className="isa101-block-icon">
      <rect x="2" y="2" width="16" height="12" fill="none" stroke="#000" strokeWidth="1"/>
      <text x="10" y="9" fontSize="6" textAnchor="middle" fill="#000">S7</text>
      <text x="10" y="13" fontSize="4" textAnchor="middle" fill="#000">SIEMENS</text>
    </svg>
  ),
  
  MQTT: () => (
    <svg width="24" height="16" viewBox="0 0 24 16" className="isa101-block-icon">
      <rect x="2" y="2" width="16" height="12" fill="none" stroke="#000" strokeWidth="1"/>
      <text x="10" y="9" fontSize="6" textAnchor="middle" fill="#000">MQTT</text>
      <circle cx="6" cy="6" r="1" fill="#000"/>
      <circle cx="10" cy="6" r="1" fill="#000"/>
      <circle cx="14" cy="6" r="1" fill="#000"/>
    </svg>
  ),

  // Signal Blocks
  SIGNAL: () => (
    <svg width="24" height="16" viewBox="0 0 24 16" className="isa101-block-icon">
      <circle cx="10" cy="8" r="6" fill="none" stroke="#000" strokeWidth="1"/>
      <text x="10" y="10" fontSize="6" textAnchor="middle" fill="#000">SIG</text>
    </svg>
  ),
  
  ANALOG: () => (
    <svg width="24" height="16" viewBox="0 0 24 16" className="isa101-block-icon">
      <rect x="2" y="2" width="16" height="12" fill="none" stroke="#000" strokeWidth="1"/>
      <path d="M4 10 L6 6 L8 10 L10 6 L12 10 L14 6 L16 10" fill="none" stroke="#000" strokeWidth="1"/>
    </svg>
  ),
  
  DIGITAL: () => (
    <svg width="24" height="16" viewBox="0 0 24 16" className="isa101-block-icon">
      <rect x="2" y="2" width="16" height="12" fill="none" stroke="#000" strokeWidth="1"/>
      <path d="M4 10 L4 6 L6 6 L6 10 L8 10 L8 6 L10 6 L10 10 L12 10 L12 6 L14 6 L14 10 L16 10" 
            fill="none" stroke="#000" strokeWidth="1"/>
    </svg>
  )
}

// Updated component categories with ISA-101 graphics
const componentCategories = [
  {
    id: 'logic',
    name: 'Logic',
    icon: <span>⚡</span>,
    color: 'blue',
    components: [
      {
        id: 'and-gate',
        type: 'and',
        label: 'AND Gate',
        icon: <ISA101BlockGraphics.AND />,
        description: 'Logical AND operation',
        category: 'logic',
        tags: ['logic', 'and', 'boolean'],
        complexity: 'basic'
      },
      {
        id: 'or-gate',
        type: 'or',
        label: 'OR Gate',
        icon: <ISA101BlockGraphics.OR />,
        description: 'Logical OR operation',
        category: 'logic',
        tags: ['logic', 'or', 'boolean'],
        complexity: 'basic'
      },
      {
        id: 'not-gate',
        type: 'not',
        label: 'NOT Gate',
        icon: <ISA101BlockGraphics.NOT />,
        description: 'Logical NOT operation',
        category: 'logic',
        tags: ['logic', 'not', 'invert'],
        complexity: 'basic'
      },
      {
        id: 'xor-gate',
        type: 'xor',
        label: 'XOR Gate',
        icon: <ISA101BlockGraphics.XOR />,
        description: 'Exclusive OR operation',
        category: 'logic',
        tags: ['logic', 'xor', 'exclusive'],
        complexity: 'basic'
      }
    ]
  },
  {
    id: 'compare',
    name: 'Compare',
    icon: <span>⚖️</span>,
    color: 'green',
    components: [
      {
        id: 'greater-than',
        type: 'greater_than',
        label: 'Greater Than',
        icon: <ISA101BlockGraphics.GT />,
        description: 'A > B comparison',
        category: 'compare',
        tags: ['comparison', 'greater', 'math'],
        complexity: 'basic'
      },
      {
        id: 'less-than',
        type: 'less_than',
        label: 'Less Than',
        icon: <ISA101BlockGraphics.LT />,
        description: 'A < B comparison',
        category: 'compare',
        tags: ['comparison', 'less', 'math'],
        complexity: 'basic'
      },
      {
        id: 'equal',
        type: 'equal',
        label: 'Equal',
        icon: <ISA101BlockGraphics.EQ />,
        description: 'A = B comparison',
        category: 'compare',
        tags: ['comparison', 'equal', 'math'],
        complexity: 'basic'
      }
    ]
  },
  {
    id: 'math',
    name: 'Math',
    icon: <span>🔢</span>,
    color: 'purple',
    components: [
      {
        id: 'add',
        type: 'add',
        label: 'Add',
        icon: <ISA101BlockGraphics.ADD />,
        description: 'Addition: A + B',
        category: 'math',
        tags: ['math', 'add', 'arithmetic'],
        complexity: 'basic'
      },
      {
        id: 'subtract',
        type: 'subtract',
        label: 'Subtract',
        icon: <ISA101BlockGraphics.SUB />,
        description: 'Subtraction: A - B',
        category: 'math',
        tags: ['math', 'subtract', 'arithmetic'],
        complexity: 'basic'
      },
      {
        id: 'multiply',
        type: 'multiply',
        label: 'Multiply',
        icon: <ISA101BlockGraphics.MUL />,
        description: 'Multiplication: A × B',
        category: 'math',
        tags: ['math', 'multiply', 'arithmetic'],
        complexity: 'basic'
      },
      {
        id: 'divide',
        type: 'divide',
        label: 'Divide',
        icon: <ISA101BlockGraphics.DIV />,
        description: 'Division: A ÷ B',
        category: 'math',
        tags: ['math', 'divide', 'arithmetic'],
        complexity: 'basic'
      }
    ]
  },
  {
    id: 'timer',
    name: 'Timers',
    icon: <span>⏱️</span>,
    color: 'orange',
    components: [
      {
        id: 'timer',
        type: 'timer',
        label: 'Timer',
        icon: <ISA101BlockGraphics.TIMER />,
        description: 'Time delay on/off',
        category: 'timer',
        tags: ['timer', 'delay', 'time'],
        complexity: 'intermediate'
      },
      {
        id: 'delay',
        type: 'delay',
        label: 'Delay',
        icon: <ISA101BlockGraphics.DELAY />,
        description: 'Signal delay block',
        category: 'timer',
        tags: ['delay', 'time', 'signal'],
        complexity: 'intermediate'
      }
    ]
  },
  {
    id: 'control',
    name: 'Control',
    icon: <span>🎛️</span>,
    color: 'red',
    components: [
      {
        id: 'pid',
        type: 'pid',
        label: 'PID Controller',
        icon: <ISA101BlockGraphics.PID />,
        description: 'Proportional-Integral-Derivative control',
        category: 'control',
        tags: ['pid', 'control', 'loop'],
        complexity: 'advanced'
      },
      {
        id: 'controller',
        type: 'controller',
        label: 'Controller',
        icon: <ISA101BlockGraphics.CONTROLLER />,
        description: 'Generic controller block',
        category: 'control',
        tags: ['control', 'generic', 'automation'],
        complexity: 'intermediate'
      }
    ]
  },
  {
    id: 'io',
    name: 'I/O',
    icon: <span>🔌</span>,
    color: 'teal',
    components: [
      {
        id: 'input',
        type: 'input',
        label: 'Input',
        icon: <ISA101BlockGraphics.INPUT />,
        description: 'Digital/Analog input',
        category: 'io',
        tags: ['input', 'io', 'signal'],
        complexity: 'basic'
      },
      {
        id: 'output',
        type: 'output',
        label: 'Output',
        icon: <ISA101BlockGraphics.OUTPUT />,
        description: 'Digital/Analog output',
        category: 'io',
        tags: ['output', 'io', 'signal'],
        complexity: 'basic'
      },
      {
        id: 'analog-input',
        type: 'analog_input',
        label: 'Analog Input',
        icon: <ISA101BlockGraphics.ANALOG />,
        description: '4-20mA, 0-10V input',
        category: 'io',
        tags: ['analog', 'input', '4-20ma'],
        complexity: 'basic'
      },
      {
        id: 'digital-input',
        type: 'digital_input',
        label: 'Digital Input',
        icon: <ISA101BlockGraphics.DIGITAL />,
        description: 'Discrete input signal',
        category: 'io',
        tags: ['digital', 'input', 'discrete'],
        complexity: 'basic'
      }
    ]
  },
  {
    id: 'communication',
    name: 'Communication',
    icon: <span>📡</span>,
    color: 'indigo',
    components: [
      {
        id: 'modbus',
        type: 'modbus',
        label: 'Modbus',
        icon: <ISA101BlockGraphics.MODBUS />,
        description: 'Modbus TCP/RTU communication',
        category: 'communication',
        tags: ['modbus', 'protocol', 'tcp', 'rtu'],
        complexity: 'advanced'
      },
      {
        id: 's7',
        type: 's7',
        label: 'Siemens S7',
        icon: <ISA101BlockGraphics.S7 />,
        description: 'Siemens S7 communication',
        category: 'communication',
        tags: ['s7', 'siemens', 'plc'],
        complexity: 'advanced'
      },
      {
        id: 'mqtt',
        type: 'mqtt',
        label: 'MQTT',
        icon: <ISA101BlockGraphics.MQTT />,
        description: 'MQTT publish/subscribe',
        category: 'communication',
        tags: ['mqtt', 'publish', 'subscribe'],
        complexity: 'intermediate'
      }
    ]
  },
  {
    id: 'signals',
    name: 'Signals',
    icon: <span>📊</span>,
    color: 'yellow',
    components: [
      {
        id: 'signal',
        type: 'signal',
        label: 'Signal',
        icon: <ISA101BlockGraphics.SIGNAL />,
        description: 'Generic signal block',
        category: 'signals',
        tags: ['signal', 'generic', 'data'],
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
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null)
  const [collapsedCategories, setCollapsedCategories] = useState<Set<string>>(new Set())
  const [favoriteComponents, setFavoriteComponents] = useState<Set<string>>(new Set())

  const handleDragStart = (e: React.DragEvent, componentType: string, componentId: string) => {
    e.dataTransfer.setData('application/reactflow', componentType)
    e.dataTransfer.effectAllowed = 'move'
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
        
        const matchesCategory = !selectedCategory || category.id === selectedCategory
        
        return matchesSearch && matchesCategory
      })
    })).filter(category => category.components.length > 0)
  }, [searchTerm, selectedCategory])

  return (
    <div className={`isa101-sidebar w-64 h-full ${className}`}>
      {/* Header */}
      <div className="isa101-panel-header">
        <span className="text-sm font-medium">Block Library</span>
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
              <span className="text-xs font-medium text-black">{category.name}</span>
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
                    onDragStart={(e) => handleDragStart(e, component.type, component.id)}
                    className="isa101-block-item group relative"
                  >
                    <div className="flex items-center gap-2 p-2 hover:bg-[#C8C8C8] cursor-move transition-colors">
                      <div className="flex-shrink-0">
                        {component.icon}
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
                      <div className="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity">
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
