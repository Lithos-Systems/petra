import { memo } from 'react'
import { Handle, Position, NodeProps } from '@xyflow/react'
import { FaBell, FaExclamationTriangle, FaCheckCircle } from 'react-icons/fa'
import type { AlarmNodeData } from '@/types/nodes'
import { getTypeColor } from '@/utils/colors'

function AlarmNode({ data, selected }: NodeProps) {
  const alarmData = data as AlarmNodeData

  return (
    <div
      className={`
        px-4 py-3 shadow-md rounded-md border-2 bg-yellow-50 min-w-[200px]
        ${selected ? 'border-yellow-500' : 'border-yellow-200'}
        ${!alarmData.configured ? 'opacity-75' : ''}
      `}
    >
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          <FaBell className="w-4 h-4 text-yellow-600" />
          <div className="text-sm font-medium">{alarmData.label}</div>
        </div>
        {alarmData.configured && <FaCheckCircle className="w-4 h-4 text-green-500" />}
      </div>

      <div className="text-xs text-gray-600 space-y-1">
        <div className="flex items-center gap-2">
          <FaExclamationTriangle className="w-3 h-3 text-yellow-600" />
          <span>Severity: {alarmData.severity}</span>
        </div>
        <div>Delay: {alarmData.delaySeconds}s | Repeat: {alarmData.repeatInterval}s</div>
        <div>Condition: {alarmData.condition}</div>
      </div>

      {/* Input for signal to monitor */}
      <Handle
        type="target"
        position={Position.Left}
        id="signal"
        className="w-3 h-3"
        style={{ 
          background: getTypeColor('float'),
          top: '30%'
        }}
      />

      {/* Input for setpoint */}
      <Handle
        type="target"
        position={Position.Left}
        id="setpoint"
        className="w-3 h-3"
        style={{ 
          background: getTypeColor('float'),
          top: '50%'
        }}
      />

      {/* Input for enable/disable */}
      <Handle
        type="target"
        position={Position.Left}
        id="enable"
        className="w-3 h-3"
        style={{ 
          background: getTypeColor('bool'),
          top: '70%'
        }}
      />

      {/* Output to trigger contacts */}
      <Handle
        type="source"
        position={Position.Right}
        id="alarm"
        className="w-3 h-3"
        style={{ 
          background: '#eab308',
          top: '40%'
        }}
      />

      {/* Acknowledged status output */}
      <Handle
        type="source"
        position={Position.Right}
        id="acknowledged"
        className="w-3 h-3"
        style={{ 
          background: getTypeColor('bool'),
          top: '60%'
        }}
      />
    </div>
  )
}

export default memo(AlarmNode)
