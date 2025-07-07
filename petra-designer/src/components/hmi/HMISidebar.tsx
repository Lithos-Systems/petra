import { useState } from 'react'
import { 
  FaSquare, 
  FaCircle, 
  FaPlay, 
  FaCog, 
  FaTachometerAlt,
  FaChartLine,
  FaFont,
  FaMousePointer,
  FaGripLines,
  FaBolt,
  FaExclamationTriangle,
  FaSlidersH,
  FaImage,
  FaWater,
  FaIndustry
} from 'react-icons/fa'

interface ComponentCategory {
  name: string
  icon: React.ReactNode
  components: ComponentItem[]
}

interface ComponentItem {
  type: string
  label: string
  icon: React.ReactNode
  description?: string
}

const componentCategories: ComponentCategory[] = [
  {
    name: 'Process Equipment',
    icon: <FaIndustry className="w-4 h-4" />,
    components: [
      { 
        type: 'tank', 
        label: 'Tank', 
        icon: <div className="w-6 h-8 border-2 border-gray-600 rounded-sm" />,
        description: 'Storage tank with level indication'
      },
      { 
        type: 'pump', 
        label: 'Pump', 
        icon: <FaCircle className="w-6 h-6 text-green-600" />,
        description: 'Centrifugal pump with status'
      },
      { 
        type: 'valve', 
        label: 'Valve', 
        icon: <FaPlay className="w-6 h-6 text-gray-600 rotate-90" />,
        description: 'Control valve with position'
      },
      { 
        type: 'motor', 
        label: 'Motor', 
        icon: <FaCog className="w-6 h-6 text-blue-600" />,
        description: 'Electric motor with VFD'
      },
      { 
        type: 'mixer', 
        label: 'Mixer', 
        icon: <div className="w-6 h-6 border-2 border-gray-600 rounded-full relative">
          <div className="absolute inset-0 flex items-center justify-center text-xs">M</div>
        </div>,
        description: 'Mixing vessel with agitator'
      },
      { 
        type: 'heat-exchanger', 
        label: 'Heat Exchanger', 
        icon: <div className="w-8 h-6 border-2 border-gray-600 relative">
          <div className="absolute inset-0 flex items-center justify-center">
            <div className="w-1 h-full bg-gray-600 mx-px"></div>
            <div className="w-1 h-full bg-gray-600 mx-px"></div>
          </div>
        </div>,
        description: 'Plate heat exchanger'
      },
      { 
        type: 'conveyor', 
        label: 'Conveyor', 
        icon: <FaGripLines className="w-6 h-6 text-gray-500" />,
        description: 'Belt conveyor system'
      },
    ]
  },
  {
    name: 'Instrumentation',
    icon: <FaTachometerAlt className="w-4 h-4" />,
    components: [
      { 
        type: 'gauge', 
        label: 'Gauge', 
        icon: <FaTachometerAlt className="w-6 h-6 text-blue-500" />,
        description: 'Analog gauge display'
      },
      { 
        type: 'trend', 
        label: 'Trend', 
        icon: <FaChartLine className="w-6 h-6 text-purple-500" />,
        description: 'Real-time trend chart'
      },
      { 
        type: 'indicator', 
        label: 'Indicator', 
        icon: <FaCircle className="w-6 h-6 text-yellow-500" />,
        description: 'Status indicator light'
      },
    ]
  },
  {
    name: 'Controls',
    icon: <FaMousePointer className="w-4 h-4" />,
    components: [
      { 
        type: 'button', 
        label: 'Button', 
        icon: <div className="w-8 h-5 bg-blue-500 rounded" />,
        description: 'Push button control'
      },
      { 
        type: 'slider', 
        label: 'Slider', 
        icon: <FaSlidersH className="w-6 h-6 text-gray-600" />,
        description: 'Analog value control'
      },
      { 
        type: 'text', 
        label: 'Text', 
        icon: <FaFont className="w-6 h-6 text-gray-700" />,
        description: 'Dynamic text display'
      },
    ]
  },
  {
    name: 'Piping',
    icon: <FaWater className="w-4 h-4" />,
    components: [
      { 
        type: 'pipe', 
        label: 'Pipe', 
        icon: <FaGripLines className="w-6 h-6 text-gray-500" />,
        description: 'Process piping'
      },
    ]
  },
  {
    name: 'Shapes & Graphics',
    icon: <FaSquare className="w-4 h-4" />,
    components: [
      { 
        type: 'shape', 
        label: 'Shape', 
        icon: <FaSquare className="w-6 h-6 text-gray-400" />,
        description: 'Basic geometric shape'
      },
      { 
        type: 'image', 
        label: 'Image', 
        icon: <FaImage className="w-6 h-6 text-gray-400" />,
        description: 'Static or dynamic image'
      },
    ]
  }
]

export default function HMISidebar() {
  const [expandedCategory, setExpandedCategory] = useState<string>('Process Equipment')
  const [searchTerm, setSearchTerm] = useState('')

  const handleDragStart = (e: React.DragEvent, type: string) => {
    e.dataTransfer.setData('hmi-component', type)
    e.dataTransfer.effectAllowed = 'copy'
  }

  const filteredCategories = componentCategories.map(category => ({
    ...category,
    components: category.components.filter(
      comp => comp.label.toLowerCase().includes(searchTerm.toLowerCase()) ||
              comp.description?.toLowerCase().includes(searchTerm.toLowerCase())
    )
  })).filter(category => category.components.length > 0)

  return (
    <div className="w-64 bg-white border-r border-gray-200 flex flex-col">
      <div className="p-4 border-b border-gray-200">
        <h3 className="text-lg font-semibold text-gray-800 mb-3">HMI Components</h3>
        
        {/* Search */}
        <input
          type="text"
          placeholder="Search components..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-petra-500"
        />
      </div>

      <div className="flex-1 overflow-y-auto">
        {filteredCategories.map((category) => (
          <div key={category.name} className="border-b border-gray-100">
            <button
              onClick={() => setExpandedCategory(
                expandedCategory === category.name ? '' : category.name
              )}
              className="w-full px-4 py-3 flex items-center justify-between hover:bg-gray-50 transition-colors"
            >
              <div className="flex items-center gap-2">
                {category.icon}
                <span className="text-sm font-medium text-gray-700">
                  {category.name}
                </span>
              </div>
              <span className="text-xs text-gray-500">
                {category.components.length}
              </span>
            </button>

            {expandedCategory === category.name && (
              <div className="px-2 pb-2">
                {category.components.map((component) => (
                  <div
                    key={component.type}
                    draggable
                    onDragStart={(e) => handleDragStart(e, component.type)}
                    className="flex items-center gap-3 p-3 mb-1 rounded-lg hover:bg-gray-100 cursor-move transition-colors group"
                  >
                    <div className="flex-shrink-0">
                      {component.icon}
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="text-sm font-medium text-gray-700">
                        {component.label}
                      </div>
                      {component.description && (
                        <div className="text-xs text-gray-500 truncate">
                          {component.description}
                        </div>
                      )}
                    </div>
                    <div className="opacity-0 group-hover:opacity-100 transition-opacity">
                      <FaMousePointer className="w-3 h-3 text-gray-400" />
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        ))}
      </div>

      {/* Templates section */}
      <div className="p-4 border-t border-gray-200">
        <h4 className="text-sm font-medium text-gray-700 mb-2">Templates</h4>
        <div className="space-y-2">
          <button className="w-full text-left px-3 py-2 text-sm bg-gray-50 hover:bg-gray-100 rounded transition-colors">
            Motor Control Panel
          </button>
          <button className="w-full text-left px-3 py-2 text-sm bg-gray-50 hover:bg-gray-100 rounded transition-colors">
            Tank Level Monitor
          </button>
          <button className="w-full text-left px-3 py-2 text-sm bg-gray-50 hover:bg-gray-100 rounded transition-colors">
            Pump Station
          </button>
        </div>
      </div>

      {/* Help text */}
      <div className="p-4 bg-gray-50 text-xs text-gray-600">
        <p>Drag components onto the canvas to create your HMI display.</p>
      </div>
    </div>
  )
}
