import { memo } from 'react'
import { Handle, Position, NodeProps } from '@xyflow/react'
import { FaServer, FaCheckCircle } from 'react-icons/fa'
import type { MqttNodeData } from '@/types/nodes'

function MqttNode({ data, selected }: NodeProps) {
  const mqttData = data as MqttNodeData

  return (
    <div
      className={`
        px-4 py-3 shadow-md rounded-md border-2 bg-orange-50
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

      <div className="text-xs text-gray-600">
        {mqttData.brokerHost}:{mqttData.brokerPort}
      </div>

      <Handle
        type="target"
        position={Position.Left}
        id="config"
        className="w-3 h-3"
        style={{ background: '#f97316' }}
      />
    </div>
  )
}

export default memo(MqttNode)
