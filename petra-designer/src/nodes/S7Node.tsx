import { memo } from 'react'
import { Handle, Position, NodeProps } from '@xyflow/react'
import { FaIndustry, FaCheckCircle } from 'react-icons/fa'
import type { S7NodeData } from '@/types/nodes'

function S7Node({ data, selected }: NodeProps) {
  const s7Data = data as S7NodeData

  return (
    <div
      className={`
        px-4 py-3 shadow-md rounded-md border-2 bg-red-50
        ${selected ? 'border-red-500' : 'border-red-200'}
        ${!s7Data.configured ? 'opacity-75' : ''}
      `}
    >
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          <FaIndustry className="w-4 h-4 text-red-600" />
          <div className="text-sm font-medium">{s7Data.label}</div>
        </div>
        {s7Data.configured && <FaCheckCircle className="w-4 h-4 text-green-500" />}
      </div>

      <div className="text-xs text-gray-600">
        {s7Data.area}
        {s7Data.dbNumber > 0 ? s7Data.dbNumber : ''}:{s7Data.address}
      </div>

      <Handle
        type="source"
        position={Position.Right}
        id="signal"
        className="w-3 h-3"
        style={{ background: '#ef4444' }}
      />
      <Handle
        type="target"
        position={Position.Left}
        id="signal"
        className="w-3 h-3"
        style={{ background: '#ef4444' }}
      />
    </div>
  )
}

export default memo(S7Node)
