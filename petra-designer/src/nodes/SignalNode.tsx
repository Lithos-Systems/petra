import { memo } from 'react'
import { Handle, Position, NodeProps } from '@xyflow/react'
import { FaCircle } from 'react-icons/fa'
import { getTypeColor } from '@/utils/colors'
import type { SignalNodeData } from '@/types/nodes'

function SignalNode({ data, selected }: NodeProps) {
  const signalData = data as SignalNodeData
  const color = getTypeColor(signalData.signalType)

  return (
    <div
      className={`
        px-4 py-2 shadow-md rounded-md border-2 bg-white
        ${selected ? 'border-petra-500' : 'border-gray-200'}
      `}
    >
      <div className="flex items-center gap-2">
        <FaCircle className="w-3 h-3" style={{ color }} />
        <div className="text-sm font-medium">{signalData.label}</div>
      </div>

      <div className="text-xs text-gray-500 mt-1">
        {signalData.signalType}: {String(signalData.initial)}
      </div>

      <Handle
        type="source"
        position={Position.Right}
        className="w-3 h-3"
        style={{ background: color }}
      />
    </div>
  )
}

export default memo(SignalNode)
