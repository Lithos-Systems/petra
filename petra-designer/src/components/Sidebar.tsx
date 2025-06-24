import { DragEvent } from 'react'
import {
  FaCircle,
  FaCubes,
  FaPhone,
  FaServer,
  FaIndustry,
} from 'react-icons/fa'

const nodeTypes = [
  {
    type: 'signal',
    label: 'Signal',
    icon: FaCircle,
    color: 'text-blue-500',
    description: 'Input/Output signal',
  },
  {
    type: 'block',
    label: 'Logic Block',
    icon: FaCubes,
    color: 'text-green-500',
    description: 'Processing logic',
  },
  {
    type: 'twilio',
    label: 'Twilio',
    icon: FaPhone,
    color: 'text-purple-500',
    description: 'SMS/Call alerts',
  },
  {
    type: 'mqtt',
    label: 'MQTT',
    icon: FaServer,
    color: 'text-orange-500',
    description: 'MQTT configuration',
  },
  {
    type: 's7',
    label: 'S7 PLC',
    icon: FaIndustry,
    color: 'text-red-500',
    description: 'Siemens S7 mapping',
  },
]

export default function Sidebar() {
  const onDragStart = (event: DragEvent, nodeType: string) => {
    event.dataTransfer.setData('application/reactflow', nodeType)
    event.dataTransfer.effectAllowed = 'move'
  }

  return (
    <div className="w-64 bg-white border-r border-gray-200 p-4">
      <h2 className="text-lg font-semibold mb-4">Components</h2>
      
      <div className="space-y-2">
        {nodeTypes.map((node) => (
          <div
            key={node.type}
            className="p-3 border border-gray-200 rounded-lg cursor-move hover:border-petra-500 hover:bg-petra-50 transition-colors"
            draggable
            onDragStart={(e) => onDragStart(e, node.type)}
          >
            <div className="flex items-center gap-3">
              <node.icon className={`w-5 h-5 ${node.color}`} />
              <div>
                <div className="font-medium text-sm">{node.label}</div>
                <div className="text-xs text-gray-500">{node.description}</div>
              </div>
            </div>
          </div>
        ))}
      </div>

      <div className="mt-6 p-3 bg-petra-50 rounded-lg">
        <h3 className="text-sm font-medium text-petra-700 mb-2">Quick Tips</h3>
        <ul className="text-xs text-petra-600 space-y-1">
          <li>• Drag components to canvas</li>
          <li>• Connect nodes by dragging handles</li>
          <li>• Click nodes to edit properties</li>
          <li>• Export to YAML when ready</li>
        </ul>
      </div>
    </div>
  )
}
