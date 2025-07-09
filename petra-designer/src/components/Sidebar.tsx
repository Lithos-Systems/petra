// File: petra-designer/src/components/EnhancedSidebar.tsx
// Enhanced PETRA Designer Sidebar with modern UI and search functionality
import { useState, useMemo } from 'react'
import {
  FaSearch,
  FaTimes,
  FaChevronDown,
  FaChevronRight,
  FaStar,
  FaHistory,
  FaFilter,
  FaIndustry,
  FaTachometerAlt,
  FaMousePointer,
  FaWater,
  FaSquare,
  FaDatabase,
  FaNetworkWired,
  FaShieldAlt,
  FaBell,
  FaChartLine,
  FaCog,
  FaPlug,
  FaCode,
  FaExclamationTriangle,
  FaInfoCircle,
  FaQuestionCircle
} from 'react-icons/fa'

interface ComponentItem {
  id: string
  type: string
  label: string
  icon: React.ReactNode
  description?: string
  category: string
  tags?: string[]
  complexity?: 'basic' | 'intermediate' | 'advanced'
  isNew?: boolean
  isBeta?: boolean
}

interface ComponentCategory {
  id: string
  name: string
  icon: React.ReactNode
  color: string
  components: ComponentItem[]
}

const componentCategories: ComponentCategory[] = [
  {
    id: 'equipment',
    name: 'Process Equipment',
    icon: <FaIndustry className="w-4 h-4" />,
    color: 'blue',
    components: [
      {
        id: 'tank',
        type: 'tank',
        label: 'Tank',
        icon: (
          <div className="w-8 h-10 relative">
            <div className="absolute inset-0 border-2 border-gray-600 rounded-sm bg-gradient-to-b from-gray-100 to-gray-300">
              <div className="absolute bottom-0 left-0 right-0 h-1/2 bg-blue-500 opacity-60 rounded-b-sm" />
            </div>
          </div>
        ),
        description: 'Storage tank with level monitoring',
        category: 'equipment',
        tags: ['storage', 'vessel', 'liquid'],
        complexity: 'basic'
      },
      {
        id: 'pump',
        type: 'pump',
        label: 'Pump',
        icon: (
          <div className="w-8 h-8 relative">
            <div className="absolute inset-0 bg-green-600 rounded-full flex items-center justify-center">
              <div className="w-4 h-4 bg-white rounded-full" />
            </div>
          </div>
        ),
        description: 'Centrifugal pump with VFD control',
        category: 'equipment',
        tags: ['pump', 'motor', 'flow'],
        complexity: 'intermediate'
      },
      {
        id: 'valve',
        type: 'valve',
        label: 'Control Valve',
        icon: (
          <div className="w-8 h-8 relative flex items-center justify-center">
            <div className="w-6 h-3 bg-gray-600 rounded-sm" />
            <div className="absolute w-0 h-0 border-l-8 border-r-8 border-b-8 border-l-transparent border-r-transparent border-b-gray-600" />
          </div>
        ),
        description: 'Modulating control valve',
        category: 'equipment',
        tags: ['valve', 'control', 'flow'],
        complexity: 'intermediate'
      },
      {
        id: 'heat-exchanger',
        type: 'heat-exchanger',
        label: 'Heat Exchanger',
        icon: (
          <div className="w-10 h-6 relative flex items-center justify-center">
            <div className="w-full h-4 border-2 border-gray-600 bg-gradient-to-r from-red-400 to-blue-400">
              <div className="absolute inset-y-0 left-1/2 w-px bg-gray-600" />
            </div>
          </div>
        ),
        description: 'Plate & frame heat exchanger',
        category: 'equipment',
        tags: ['heat', 'temperature', 'exchanger'],
        complexity: 'advanced',
        isNew: true
      },
      {
        id: 'mixer',
        type: 'mixer',
        label: 'Mixer/Agitator',
        icon: (
          <div className="w-8 h-8 relative">
            <div className="absolute inset-0 border-2 border-gray-600 rounded-full">
              <div className="absolute inset-2 border-t-2 border-gray-600 rounded-full animate-spin" />
            </div>
          </div>
        ),
        description: 'Mixing vessel with agitator',
        category: 'equipment',
        tags: ['mixer', 'agitator', 'blend'],
        complexity: 'intermediate',
        isBeta: true
      }
    ]
  },
  {
    id: 'instrumentation',
    name: 'Instrumentation',
    icon: <FaTachometerAlt className="w-4 h-4" />,
    color: 'purple',
    components: [
      {
        id: 'gauge',
        type: 'gauge',
        label: 'Gauge',
        icon: <FaTachometerAlt className="w-6 h-6 text-blue-500" />,
        description: 'Analog gauge display',
        category: 'instrumentation',
        tags: ['gauge', 'analog', 'display'],
        complexity: 'basic'
      },
      {
        id: 'trend',
        type: 'trend',
        label: 'Trend Chart',
        icon: <FaChartLine className="w-6 h-6 text-purple-500" />,
        description: 'Real-time trend display',
        category: 'instrumentation',
        tags: ['trend', 'chart', 'graph'],
        complexity: 'intermediate'
      },
      {
        id: 'digital-display',
        type: 'digital-display',
        label: 'Digital Display',
        icon: (
          <div className="w-8 h-6 bg-black rounded flex items-center justify-center">
            <span className="text-green-400 font-mono text-xs">88.8</span>
          </div>
        ),
        description: 'LED/LCD numeric display',
        category: 'instrumentation',
        tags: ['digital', 'display', 'numeric'],
        complexity: 'basic',
        isNew: true
      }
    ]
  },
  {
    id: 'controls',
    name: 'Controls',
    icon: <FaMousePointer className="w-4 h-4" />,
    color: 'green',
    components: [
      {
        id: 'button',
        type: 'button',
        label: 'Push Button',
        icon: (
          <div className="w-8 h-6 bg-blue-500 rounded shadow-md flex items-center justify-center">
            <div className="w-2 h-2 bg-white rounded-full" />
          </div>
        ),
        description: 'Interactive push button',
        category: 'controls',
        tags: ['button', 'control', 'input'],
        complexity: 'basic'
      },
      {
        id: 'switch',
        type: 'switch',
        label: 'Toggle Switch',
        icon: (
          <div className="w-8 h-4 bg-gray-300 rounded-full relative">
            <div className="absolute left-0 top-0 w-4 h-4 bg-green-500 rounded-full shadow" />
          </div>
        ),
        description: 'ON/OFF toggle switch',
        category: 'controls',
        tags: ['switch', 'toggle', 'boolean'],
        complexity: 'basic'
      },
      {
        id: 'slider',
        type: 'slider',
        label: 'Slider',
        icon: (
          <div className="w-8 h-2 bg-gray-300 rounded-full relative">
            <div className="absolute left-1/2 top-1/2 transform -translate-x-1/2 -translate-y-1/2 w-3 h-3 bg-blue-500 rounded-full shadow" />
          </div>
        ),
        description: 'Analog value slider',
        category: 'controls',
        tags: ['slider', 'analog', 'input'],
        complexity: 'basic'
      }
    ]
  },
  {
    id: 'logic',
    name: 'Logic Blocks',
    icon: <FaCode className="w-4 h-4" />,
    color: 'orange',
    components: [
      {
        id: 'and-gate',
        type: 'AND',
        label: 'AND Gate',
        icon: (
          <div className="w-8 h-6 border-2 border-gray-600 rounded-r-full flex items-center justify-center bg-white">
            <span className="text-xs font-bold">&</span>
          </div>
        ),
        description: 'Logical AND operation',
        category: 'logic',
        tags: ['logic', 'and', 'boolean'],
        complexity: 'basic'
      },
      {
        id: 'pid',
        type: 'PID',
        label: 'PID Controller',
        icon: (
          <div className="w-8 h-6 bg-orange-500 rounded flex items-center justify-center">
            <span className="text-white text-xs font-bold">PID</span>
          </div>
        ),
        description: 'PID control block',
        category: 'logic',
        tags: ['pid', 'control', 'loop'],
        complexity: 'advanced'
      }
    ]
  },
  {
    id: 'connectivity',
    name: 'Connectivity',
    icon: <FaNetworkWired className="w-4 h-4" />,
    color: 'teal',
    components: [
      {
        id: 'mqtt',
        type: 'mqtt',
        label: 'MQTT Client',
        icon: <FaPlug className="w-6 h-6 text-teal-600" />,
        description: 'MQTT broker connection',
        category: 'connectivity',
        tags: ['mqtt', 'protocol', 'network'],
        complexity: 'intermediate'
      },
      {
        id: 's7',
        type: 's7',
        label: 'S7 Driver',
        icon: (
          <div className="w-8 h-6 bg-gray-700 rounded flex items-center justify-center">
            <span className="text-white text-xs font-bold">S7</span>
          </div>
        ),
        description: 'Siemens S7 protocol',
        category: 'connectivity',
        tags: ['s7', 'siemens', 'plc'],
        complexity: 'advanced'
      }
    ]
  }
]

export default function EnhancedSidebar() {
  const [searchTerm, setSearchTerm] = useState('')
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null)
  const [collapsedCategories, setCollapsedCategories] = useState<Set<string>>(new Set())
  const [favoriteComponents, setFavoriteComponents] = useState<Set<string>>(new Set())
  const [recentComponents, setRecentComponents] = useState<string[]>([])
  const [showFavorites, setShowFavorites] = useState(false)
  const [showRecent, setShowRecent] = useState(false)
  const [complexityFilter, setComplexityFilter] = useState<string | null>(null)

  // Filter components based on search and filters
  const filteredCategories = useMemo(() => {
    return componentCategories.map(category => ({
      ...category,
      components: category.components.filter(component => {
        const matchesSearch = searchTerm === '' || 
          component.label.toLowerCase().includes(searchTerm.toLowerCase()) ||
          component.description?.toLowerCase().includes(searchTerm.toLowerCase()) ||
          component.tags?.some(tag => tag.toLowerCase().includes(searchTerm.toLowerCase()))
        
        const matchesCategory = !selectedCategory || category.id === selectedCategory
        
        const matchesComplexity = !complexityFilter || component.complexity === complexityFilter
        
        const matchesFavorites = !showFavorites || favoriteComponents.has(component.id)
        
        const matchesRecent = !showRecent || recentComponents.includes(component.id)
        
        return matchesSearch && matchesCategory && matchesComplexity && matchesFavorites && matchesRecent
      })
    })).filter(category => category.components.length > 0)
  }, [searchTerm, selectedCategory, complexityFilter, showFavorites, showRecent, favoriteComponents, recentComponents])

  const handleDragStart = (e: React.DragEvent, componentType: string, componentId: string) => {
    e.dataTransfer.setData('application/reactflow', componentType)
    e.dataTransfer.effectAllowed = 'move'
    
    // Add to recent components
    setRecentComponents(prev => {
      const updated = [componentId, ...prev.filter(id => id !== componentId)].slice(0, 5)
      return updated
    })
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

  return (
    <div className="w-80 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 flex flex-col h-full">
      {/* Header */}
      <div className="p-4 border-b border-gray-200 dark:border-gray-700">
        <h2 className="text-lg font-semibold text-gray-800 dark:text-gray-200 mb-3">Components</h2>
        
        {/* Search Bar */}
        <div className="relative">
          <FaSearch className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-4 h-4" />
          <input
            type="text"
            placeholder="Search components..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="w-full pl-10 pr-10 py-2 bg-gray-50 dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-petra-primary-500 text-sm"
          />
          {searchTerm && (
            <button
              onClick={() => setSearchTerm('')}
              className="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-gray-600"
            >
              <FaTimes className="w-4 h-4" />
            </button>
          )}
        </div>
        
        {/* Filter Buttons */}
        <div className="flex flex-wrap gap-2 mt-3">
          <button
            onClick={() => setShowFavorites(!showFavorites)}
            className={`px-3 py-1 rounded-full text-xs font-medium transition-colors ${
              showFavorites 
                ? 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300' 
                : 'bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400'
            }`}
          >
            <FaStar className="inline w-3 h-3 mr-1" />
            Favorites
          </button>
          <button
            onClick={() => setShowRecent(!showRecent)}
            className={`px-3 py-1 rounded-full text-xs font-medium transition-colors ${
              showRecent 
                ? 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300' 
                : 'bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400'
            }`}
          >
            <FaHistory className="inline w-3 h-3 mr-1" />
            Recent
          </button>
          <select
            value={complexityFilter || ''}
            onChange={(e) => setComplexityFilter(e.target.value || null)}
            className="px-3 py-1 rounded-full text-xs font-medium bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400 border-none focus:outline-none focus:ring-2 focus:ring-petra-primary-500"
          >
            <option value="">All Levels</option>
            <option value="basic">Basic</option>
            <option value="intermediate">Intermediate</option>
            <option value="advanced">Advanced</option>
          </select>
        </div>
      </div>
      
      {/* Component Categories */}
      <div className="flex-1 overflow-y-auto petra-scrollbar">
        {filteredCategories.map(category => (
          <div key={category.id} className="border-b border-gray-200 dark:border-gray-700 last:border-b-0">
            {/* Category Header */}
            <button
              onClick={() => toggleCategory(category.id)}
              className="w-full px-4 py-3 flex items-center justify-between hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
            >
              <div className="flex items-center gap-2">
                <div className={`p-1.5 rounded bg-${category.color}-100 dark:bg-${category.color}-900`}>
                  {category.icon}
                </div>
                <span className="font-medium text-gray-700 dark:text-gray-300">{category.name}</span>
                <span className="text-xs text-gray-500 dark:text-gray-400">({category.components.length})</span>
              </div>
              {collapsedCategories.has(category.id) ? (
                <FaChevronRight className="w-3 h-3 text-gray-400" />
              ) : (
                <FaChevronDown className="w-3 h-3 text-gray-400" />
              )}
            </button>
            
            {/* Category Components */}
            {!collapsedCategories.has(category.id) && (
              <div className="px-2 pb-2">
                {category.components.map(component => (
                  <div
                    key={component.id}
                    draggable
                    onDragStart={(e) => handleDragStart(e, component.type, component.id)}
                    className="petra-component-item group relative"
                  >
                    <div className="flex items-center gap-3">
                      <div className="flex-shrink-0">{component.icon}</div>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2">
                          <span className="font-medium text-sm text-gray-700 dark:text-gray-300">
                            {component.label}
                          </span>
                          {component.isNew && (
                            <span className="px-1.5 py-0.5 text-xs bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300 rounded">
                              NEW
                            </span>
                          )}
                          {component.isBeta && (
                            <span className="px-1.5 py-0.5 text-xs bg-orange-100 text-orange-700 dark:bg-orange-900 dark:text-orange-300 rounded">
                              BETA
                            </span>
                          )}
                        </div>
                        <p className="text-xs text-gray-500 dark:text-gray-400 truncate">
                          {component.description}
                        </p>
                      </div>
                      <button
                        onClick={(e) => {
                          e.stopPropagation()
                          toggleFavorite(component.id)
                        }}
                        className="opacity-0 group-hover:opacity-100 transition-opacity"
                      >
                        <FaStar
                          className={`w-4 h-4 ${
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
                          <div className="w-1 h-3 bg-green-500 rounded" title="Basic" />
                        )}
                        {component.complexity === 'intermediate' && (
                          <div className="flex gap-0.5">
                            <div className="w-1 h-3 bg-yellow-500 rounded" title="Intermediate" />
                            <div className="w-1 h-3 bg-yellow-500 rounded" />
                          </div>
                        )}
                        {component.complexity === 'advanced' && (
                          <div className="flex gap-0.5">
                            <div className="w-1 h-3 bg-red-500 rounded" title="Advanced" />
                            <div className="w-1 h-3 bg-red-500 rounded" />
                            <div className="w-1 h-3 bg-red-500 rounded" />
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
          <div className="p-8 text-center text-gray-500 dark:text-gray-400">
            <FaSearch className="w-12 h-12 mx-auto mb-3 text-gray-300 dark:text-gray-600" />
            <p className="text-sm">No components found</p>
            <p className="text-xs mt-1">Try adjusting your search or filters</p>
          </div>
        )}
      </div>
      
      {/* Footer Help */}
      <div className="p-4 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900">
        <div className="flex items-center justify-between text-xs text-gray-500 dark:text-gray-400">
          <div className="flex items-center gap-1">
            <FaInfoCircle className="w-3 h-3" />
            <span>Drag components to canvas</span>
          </div>
          <button className="hover:text-gray-700 dark:hover:text-gray-300">
            <FaQuestionCircle className="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>
  )
}
