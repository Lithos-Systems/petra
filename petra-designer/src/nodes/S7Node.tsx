import { memo } from 'react'
import { Handle, Position, NodeProps } from '@xyflow/react'
import { FaIndustry, FaCheckCircle } from 'react-icons/fa'
import type { S7NodeData } from '@/types/nodes'

function S7Node({ data, selected }: NodeProps<S7NodeData>) {
  return (
    <div
      className={`
        px-4 py-3 shadow-md rounded-md border-2 bg-red-50
        ${selected ? 'border-red-500' : 'border-red-200'}
        ${!data.configured ? 'opacity-75' : ''}
      `}
    >
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          <FaIndustry className="w-4 h-4 text-red-600" />
          <div className="text-sm font-medium">{data.label}</div>
        </div>
        {data.configured && <FaCheckCircle className="w-4 h-4 text-green-500" />}
      </div>

      <div className="text-xs text-gray-600">
        {data.area}
        {data.dbNumber > 0 ? data.dbNumber : ''}:{data.address}
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
