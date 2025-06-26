import { memo } from 'react'
import { Handle, Position, NodeProps } from '@xyflow/react'
import { FaUser, FaEnvelope, FaPhone, FaSms, FaCheckCircle } from 'react-icons/fa'
import type { ContactNodeData } from '@/types/nodes'

function ContactNode({ data, selected }: NodeProps) {
  const contactData = data as ContactNodeData

  const getMethodIcon = () => {
    switch (contactData.preferredMethod) {
      case 'email': return <FaEnvelope className="w-4 h-4 text-blue-600" />
      case 'sms': return <FaSms className="w-4 h-4 text-green-600" />
      case 'call': return <FaPhone className="w-4 h-4 text-purple-600" />
      default: return <FaUser className="w-4 h-4 text-gray-600" />
    }
  }

  return (
    <div
      className={`
        px-4 py-3 shadow-md rounded-md border-2 bg-indigo-50 min-w-[180px]
        ${selected ? 'border-indigo-500' : 'border-indigo-200'}
        ${!contactData.configured ? 'opacity-75' : ''}
      `}
    >
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          <FaUser className="w-4 h-4 text-indigo-600" />
          <div className="text-sm font-medium">{contactData.label}</div>
        </div>
        {contactData.configured && <FaCheckCircle className="w-4 h-4 text-green-500" />}
      </div>

      <div className="text-xs text-gray-600 space-y-1">
        <div className="flex items-center gap-2">
          {getMethodIcon()}
          <span>{contactData.name || 'Not configured'}</span>
        </div>
        <div className="text-xs">
          Priority: {contactData.priority} | Delay: {contactData.escalationDelay}s
        </div>
      </div>

      <Handle
        type="source"
        position={Position.Right}
        id="contact"
        className="w-3 h-3"
        style={{ background: '#6366f1' }}
      />
    </div>
  )
}

export default memo(ContactNode)
