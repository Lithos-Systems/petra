import { memo } from 'react'
import { Handle, Position, NodeProps } from '@xyflow/react'
import { FaPhone, FaSms, FaCheckCircle, FaExclamationTriangle, FaMobile } from 'react-icons/fa'
import type { TwilioNodeData } from '@/types/nodes'

function TwilioNode({ data, selected }: NodeProps) {
  const twilioData = data as TwilioNodeData
  const Icon = twilioData.actionType === 'call' ? FaPhone : FaSms

  // Status indicator
  const getStatusColor = () => {
    if (!twilioData.configured) return '#808080' // Gray - not configured
    if (twilioData.toNumber && twilioData.content) {
      return '#00C800' // Green - configured
    }
    return '#FF8C00' // Orange - partially configured
  }

  const statusColor = getStatusColor()

  // Format phone number display
  const formatPhoneNumber = (phone: string) => {
    if (!phone) return '?'
    if (phone.startsWith('+1') && phone.length === 12) {
      // US format: +1 (xxx) xxx-xxxx
      return `+1 (${phone.slice(2,5)}) ${phone.slice(5,8)}-${phone.slice(8)}`
    }
    // International format: keep as is but truncate if too long
    return phone.length > 15 ? phone.slice(0,12) + '...' : phone
  }

  return (
    <div
      className="relative bg-white min-w-[200px] min-h-[100px] flex flex-col"
      style={{
        border: selected ? '3px solid #000080' : '2px solid #404040',
        backgroundColor: '#FFFFFF'
      }}
    >
      {/* Header Section */}
      <div 
        className="flex items-center justify-between px-3 py-2 border-b border-gray-400"
        style={{ backgroundColor: '#E0E0E0' }}
      >
        <div className="flex items-center gap-2">
          <Icon className="w-4 h-4" style={{ color: '#404040' }} />
          <span className="text-sm font-medium text-black">TWILIO</span>
        </div>
        <div className="flex items-center gap-1">
          {twilioData.configured ? (
            <FaCheckCircle className="w-3 h-3" style={{ color: '#00C800' }} />
          ) : (
            <FaExclamationTriangle className="w-3 h-3" style={{ color: '#FF8C00' }} />
          )}
          <FaMobile className="w-3 h-3" style={{ color: statusColor }} />
        </div>
      </div>

      {/* Content Section */}
      <div className="flex-1 px-3 py-2">
        {/* Node Label */}
        <div className="text-sm font-medium text-black mb-2">
          {twilioData.label}
        </div>

        {/* Configuration Display */}
        <div className="space-y-1 text-xs text-gray-700">
          <div className="flex justify-between">
            <span>Action:</span>
            <span className="font-medium" style={{ 
              color: twilioData.actionType === 'call' ? '#000080' : '#008000'
            }}>
              {twilioData.actionType === 'call' ? 'VOICE CALL' : 'SMS TEXT'}
            </span>
          </div>
          
          <div className="flex justify-between">
            <span>To:</span>
            <span className="font-mono text-xs">
              {formatPhoneNumber(twilioData.toNumber)}
            </span>
          </div>

          {twilioData.content && (
            <div className="mt-2">
              <div className="text-xs text-gray-600 mb-1">Message:</div>
              <div 
                className="text-xs font-mono p-2 border border-gray-300 bg-gray-50 max-h-12 overflow-hidden"
                style={{ fontSize: '10px' }}
              >
                {twilioData.content.length > 60 
                  ? twilioData.content.slice(0, 60) + '...' 
                  : twilioData.content}
              </div>
            </div>
          )}

          {/* Configuration indicators */}
          <div className="flex gap-2 mt-2">
            {twilioData.toNumber && (
              <div className="text-xs" style={{ color: '#008000' }}>
                ✓ Phone
              </div>
            )}
            {twilioData.content && (
              <div className="text-xs" style={{ color: '#008000' }}>
                ✓ Message
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Input Handles */}
      <Handle
        type="target"
        position={Position.Left}
        id="trigger"
        style={{ 
          background: '#404040',
          border: '1px solid #FFFFFF',
          width: '10px',
          height: '10px',
          borderRadius: '0',
          top: '40%'
        }}
        className="handle-input"
      />

      <Handle
        type="target"
        position={Position.Left}
        id="message"
        style={{ 
          background: '#404040',
          border: '1px solid #FFFFFF',
          width: '10px',
          height: '10px',
          borderRadius: '0',
          top: '60%'
        }}
        className="handle-input"
      />

      {/* Output Handle */}
      <Handle
        type="source"
        position={Position.Right}
        id="success"
        style={{ 
          background: '#404040',
          border: '1px solid #FFFFFF',
          width: '10px',
          height: '10px',
          borderRadius: '0',
          top: '50%'
        }}
        className="handle-output"
      />

      {/* Status Indicator */}
      <div 
        className="absolute top-2 right-2 w-3 h-3 border border-black"
        style={{ backgroundColor: statusColor }}
        title={twilioData.configured ? 'Configured' : 'Not Configured'}
      />
    </div>
  )
}

export default memo(TwilioNode)
