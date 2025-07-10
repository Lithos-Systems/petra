import React, { useState, useMemo } from 'react'
import { FaSearch, FaChevronRight, FaChevronDown } from 'react-icons/fa'

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

  // Memory Blocks
  MEMORY: () => (
    <svg width="24" height="16" viewBox="0 0 24 16" className="isa101-block-icon">
      <rect x="2" y="2" width="16" height="12" fill="none" stroke="#000" strokeWidth="1"/>
      <rect x="4" y="4" width="12" height="2" fill="#000"/>
      <rect x="4" y="7" width="12" height="2" fill="#000"/>
      <rect x="4" y="10" width="12" height="2" fill="#000"/>
    </svg>
  ),
  
  REGISTER: () => (
    <svg width="24" height="16" viewBox="0 0 24 16" className="isa101-block-icon">
      <rect x="2" y="2" width="16" height="12" fill="none" stroke="#000" strokeWidth="1"/>
      <text x="10" y="10" fontSize="6" textAnchor="middle" fill="#000">REG</text>
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

// Block categories following ISA-101 organization
const blockCategories = [
  {
    id: 'logic',
    name: 'Logic',
    collapsed: false,
    blocks: [
      { id: 'AND', type: 'and', label: 'AND Gate', description: 'Logical AND operation' },
      { id: 'OR', type: 'or', label: 'OR Gate', description: 'Logical OR operation' },
      { id: 'NOT', type: 'not', label: 'NOT Gate', description: 'Logical NOT operation' },
      { id: 'XOR', type: 'xor', label: 'XOR Gate', description: 'Exclusive OR operation' },
    ]
  },
  {
    id: 'compare',
    name: 'Compare',
    collapsed: false,
    blocks: [
      { id: 'GT', type: 'greater_than', label: 'Greater Than', description: 'A > B comparison' },
      { id: 'LT', type: 'less_than', label: 'Less Than', description: 'A < B comparison' },
      { id: 'EQ', type: 'equal', label: 'Equal', description: 'A = B comparison' },
    ]
  },
  {
    id: 'math',
    name: 'Math',
    collapsed: false,
    blocks: [
      { id: 'ADD', type: 'add', label: 'Add', description: 'Addition: A + B' },
      { id: 'SUB', type: 'subtract', label: 'Subtract', description: 'Subtraction: A - B' },
      { id: 'MUL', type: 'multiply', label: 'Multiply', description: 'Multiplication: A × B' },
      { id: 'DIV', type: 'divide', label: 'Divide', description: 'Division: A ÷ B' },
    ]
  },
  {
    id: 'timer',
    name: 'Timers',
    collapsed: false,
    blocks: [
      { id: 'TIMER', type: 'timer', label: 'Timer', description: 'Time delay on/off' },
      { id: 'DELAY', type: 'delay', label: 'Delay', description: 'Signal delay block' },
    ]
  },
  {
    id: 'control',
    name: 'Control',
    collapsed: false,
    blocks: [
      { id: 'PID', type: 'pid', label: 'PID Controller', description: 'Proportional-Integral-Derivative control' },
      { id: 'CONTROLLER', type: 'controller', label: 'Controller', description: 'Generic controller block' },
    ]
  },
  {
    id: 'io',
    name: 'I/O',
    collapsed: false,
    blocks: [
      { id: 'INPUT', type: 'input', label: 'Input', description: 'Digital/Analog input' },
      { id: 'OUTPUT', type: 'output', label: 'Output', description: 'Digital/Analog output' },
      { id: 'ANALOG', type: 'analog_input', label: 'Analog Input', description: '4-20mA, 0-10V input' },
      { id: 'DIGITAL', type: 'digital_input', label: 'Digital Input', description: 'Discrete input signal' },
    ]
  },
  {
    id: 'memory',
    name: 'Memory',
    collapsed: false,
    blocks: [
      { id: 'MEMORY', type: 'memory', label: 'Memory', description: 'Data storage block' },
      { id: 'REGISTER', type: 'register', label: 'Register', description: 'Value register' },
    ]
  },
  {
    id: 'communication',
    name: 'Communication',
    collapsed: false,
    blocks: [
      { id: 'MODBUS', type: 'modbus', label: 'Modbus', description: 'Modbus TCP/RTU communication' },
      { id: 'S7', type: 's7', label: 'Siemens S7', description: 'Siemens S7 communication' },
      { id: 'MQTT', type: 'mqtt', label: 'MQTT', description: 'MQTT publish/subscribe' },
    ]
  }
]

interface ISA101SidebarProps {
  className?: string
}

export default function ISA101Sidebar({ className = '' }: ISA101SidebarProps) {
  const [searchTerm, setSearchTerm] = useState('')
  const [collapsedCategories, setCollapsedCategories] = useState<Set<string>>(new Set())

  const handleDragStart = (e: React.DragEvent, blockType: string) => {
    e.dataTransfer.setData('application/reactflow', blockType)
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

  const filteredCategories = useMemo(() => {
    return blockCategories.map(category => ({
      ...category,
      blocks: category.blocks.filter(block =>
        searchTerm === '' || 
        block.label.toLowerCase().includes(searchTerm.toLowerCase()) ||
        block.description.toLowerCase().includes(searchTerm.toLowerCase())
      )
    })).filter(category => category.blocks.length > 0)
  }, [searchTerm])

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
                <span className="text-xs text-[#606060]">{category.blocks.length}</span>
                {collapsedCategories.has(category.id) ? (
                  <FaChevronRight className="w-2 h-2 text-[#606060]" />
                ) : (
                  <FaChevronDown className="w-2 h-2 text-[#606060]" />
                )}
              </div>
            </button>

            {/* Category Blocks */}
            {!collapsedCategories.has(category.id) && (
              <div className="px-1 pb-2">
                {category.blocks.map(block => {
                  const BlockIcon = ISA101BlockGraphics[block.id as keyof typeof ISA101BlockGraphics]
                  return (
                    <div
                      key={block.id}
                      draggable
                      onDragStart={(e) => handleDragStart(e, block.type)}
                      className="isa101-block-item group"
                    >
                      <div className="flex items-center gap-2 p-2 hover:bg-[#C8C8C8] cursor-move transition-colors">
                        <div className="flex-shrink-0">
                          {BlockIcon ? <BlockIcon /> : (
                            <div className="w-6 h-4 bg-[#E0E0E0] border border-[#606060] flex items-center justify-center">
                              <span className="text-xs font-mono">{block.id.slice(0, 2)}</span>
                            </div>
                          )}
                        </div>
                        <div className="flex-1 min-w-0">
                          <div className="text-xs font-medium text-black">{block.label}</div>
                          <div className="text-xs text-[#606060] truncate">{block.description}</div>
                        </div>
                      </div>
                    </div>
                  )
                })}
              </div>
            )}
          </div>
        ))}
      </div>

      {/* Footer */}
      <div className="isa101-panel-header border-t border-[#606060]">
        <div className="text-xs text-[#606060]">
          Drag blocks to canvas
        </div>
      </div>
    </div>
  )
}
