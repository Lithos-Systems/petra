import { memo } from 'react'
import { Handle, Position, NodeProps } from '@xyflow/react'
import { FaPhone, FaSms, FaCheckCircle } from 'react-icons/fa'
import type { TwilioNodeData } from '@/types/nodes'

function TwilioNode({ data, selected }: NodeProps) {
  const twilioData = data as TwilioNodeData
  const Icon = twilioData.actionType === 'call' ? FaPhone : FaSms

  return (
    <div
      className={`
        px-4 py-3 shadow-md rounded-md border-2 bg-purple-50
        ${selected ? 'border-purple-500' : 'border-purple-200'}
        ${!twilioData.configured ? 'opacity-75' : ''}
      `}
    >
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          <Icon className="w-4 h-4 text-purple-600" />
          <div className="text-sm font-medium">{twilioData.label}</div>
        </div>
        {twilioData.configured && <FaCheckCircle className="w-4 h-4 text-green-500" />}
      </div>

      <div className="text-xs text-gray-600">
        {twilioData.actionType === 'call' ? 'Voice Call' : 'SMS'}
      </div>

      <Handle
        type="target"
        position={Position.Left}
        id="trigger"
        className="w-3 h-3"
        style={{ background: '#9333ea' }}
      />
      <Handle
        type="source"
        position={Position.Right}
        id="success"
        className="w-3 h-3"
        style={{ background: '#10b981' }}
      />
    </div>
  )
}

export default memo(TwilioNode)
