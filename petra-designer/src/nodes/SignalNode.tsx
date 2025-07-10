import { memo } from 'react'
import { Handle, Position, NodeProps } from '@xyflow/react'
import type { SignalNodeData } from '@/types/nodes'

function SignalNode({ data, selected }: NodeProps) {
  const signalData = data as SignalNodeData

  // Get color based on signal type
  const getSignalColor = (type: string) => {
    switch (type) {
      case 'bool': return '#FF0000'
      case 'int': return '#0000FF'
      case 'float': return '#008000'
      default: return '#808080'
    }
  }

  const signalColor = getSignalColor(signalData.signalType)

  return (
    <div
      className={`
        relative bg-white min-w-[100px] min-h-[60px] flex flex-col items-center justify-center
        ${selected ? 'border-2 border-blue-800' : 'border-2 border-gray-700'}
      `}
      style={{
        borderColor: selected ? '#000080' : '#404040',
        backgroundColor: '#FFFFFF'
      }}
    >
      {/* Signal Icon */}
      <div className="mb-1">
        <svg width="40" height="30" viewBox="0 0 40 30" className="signal-graphic">
          <circle cx="20" cy="15" r="12" fill="#FFFFFF" stroke="#000000" strokeWidth="2"/>
          <circle cx="20" cy="15" r="8" fill={signalColor} stroke="#000000" strokeWidth="1"/>
          <text x="20" y="18" fontSize="6" textAnchor="middle" fill="#FFFFFF" fontWeight="bold">
            {signalData.signalType.toUpperCase()}
          </text>
        </svg>
      </div>

      {/* Signal Label */}
      <div className="text-xs font-medium text-black text-center px-1">
        {signalData.label}
      </div>

      {/* Signal Value */}
      <div className="text-xs text-gray-600 text-center">
        {String(signalData.initial)}
      </div>

      {/* Input Handle (for write mode or read/write) */}
      {(signalData.mode === 'write' || signalData.mode === 'read') && (
        <Handle
          type="target"
          position={Position.Left}
          style={{
            background: signalColor,
            border: '1px solid #000000',
            width: '8px',
            height: '8px',
            borderRadius: '0'
          }}
          className="handle-input"
        />
      )}

      {/* Output Handle */}
      <Handle
        type="source"
        position={Position.Right}
        style={{
          background: signalColor,
          border: '1px solid #000000',
          width: '8px',
          height: '8px',
          borderRadius: '0'
        }}
        className="handle-output"
      />
    </div>
  )
}

export default memo(SignalNode)
