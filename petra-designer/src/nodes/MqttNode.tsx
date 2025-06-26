import { memo } from 'react'
import { Handle, Position, NodeProps } from '@xyflow/react'
import { FaServer, FaCheckCircle, FaArrowDown, FaArrowUp } from 'react-icons/fa'
import type { MqttNodeData } from '@/types/nodes'
import { getTypeColor } from '@/utils/colors'

function MqttNode({ data, selected }: NodeProps) {
  const mqttData = data as MqttNodeData

  // Show different handles based on mode
  const showInput = mqttData.mode === 'write' || mqttData.mode === 'read_write'
  const showOutput = mqttData.mode === 'read' || mqttData.mode === 'read_write'

  return (
    <div
      className={`
        px-4 py-3 shadow-md rounded-md border-2 bg-orange-50 min-w-[180px]
        ${selected ? 'border-orange-500' : 'border-orange-200'}
        ${!mqttData.configured ? 'opacity-75' : ''}
      `}
    >
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          <FaServer className="w-4 h-4 text-orange-600" />
          <div className="text-sm font-medium">{mqttData.label}</div>
        </div>
        {mqttData.configured && <FaCheckCircle className="w-4 h-4 text-green-500" />}
      </div>

      <div className="text-xs text-gray-600 space-y-1">
        <div>{mqttData.brokerHost}:{mqttData.brokerPort}</div>
        <div className="flex items-center gap-2">
          <span className="font-medium">Topic:</span> 
          <span>{mqttData.topicPrefix}/{mqttData.signalName || '?'}</span>
        </div>
        <div className="flex items-center gap-2">
          {showInput && <FaArrowUp className="w-3 h-3 text-orange-600" />}
          {showOutput && <FaArrowDown className="w-3 h-3 text-orange-600" />}
          <span className="text-xs">{mqttData.mode}</span>
        </div>
      </div>

      {/* Input handle for writing to MQTT */}
      {showInput && (
        <Handle
          type="target"
          position={Position.Left}
          id="value"
          className="w-3 h-3"
          style={{ 
            background: getTypeColor(mqttData.signalType || 'float'),
            top: '50%'
          }}
        />
      )}

      {/* Output handle for reading from MQTT */}
      {showOutput && (
        <Handle
          type="source"
          position={Position.Right}
          id="value"
          className="w-3 h-3"
          style={{ 
            background: getTypeColor(mqttData.signalType || 'float'),
            top: '50%'
          }}
        />
      )}
    </div>
  )
}

export default memo(MqttNode)
