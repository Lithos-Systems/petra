import { memo } from 'react'
import { Handle, Position, NodeProps } from '@xyflow/react'
import { FaEnvelope, FaCheckCircle } from 'react-icons/fa'
import type { EmailNodeData } from '@/types/nodes'

function EmailNode({ data, selected }: NodeProps) {
  const emailData = data as EmailNodeData

  return (
    <div
      className={`
        px-4 py-3 shadow-md rounded-md border-2 bg-blue-50 min-w-[180px]
        ${selected ? 'border-blue-500' : 'border-blue-200'}
        ${!emailData.configured ? 'opacity-75' : ''}
      `}
    >
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          <FaEnvelope className="w-4 h-4 text-blue-600" />
          <div className="text-sm font-medium">{emailData.label}</div>
        </div>
        {emailData.configured && <FaCheckCircle className="w-4 h-4 text-green-500" />}
      </div>

      <div className="text-xs text-gray-600 space-y-1">
        <div>To: {emailData.toEmail || 'Not configured'}</div>
        <div>Provider: {emailData.provider || 'SMTP'}</div>
      </div>

      <Handle
        type="target"
        position={Position.Left}
        id="trigger"
        className="w-3 h-3"
        style={{ background: '#3b82f6' }}
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

export default memo(EmailNode)
