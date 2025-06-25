import { memo } from 'react'
import { Handle, Position, NodeProps } from '@xyflow/react'

import { getBlockIcon } from '@/utils/blockIcons'
import { getTypeColor } from '@/utils/colors'
import type { BlockNodeData } from '@/types/nodes'

/**
 * Render a logic / timer / math block.
 * Inputs appear on the left, outputs on the right.
 */
function BlockNode({ data, selected }: NodeProps<BlockNodeData>) {
  const Icon = getBlockIcon(data.blockType)

  return (
    <div
      className={`
        px-4 py-3 shadow-md rounded-md border-2 bg-white min-w-[180px]
        ${selected ? 'border-petra-500' : 'border-gray-200'}
      `}
    >
      {/* header */}
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          <Icon className="w-4 h-4 text-petra-600" />
          <div className="text-sm font-medium">{data.label}</div>
        </div>
        <div className="text-xs text-gray-500 bg-gray-100 px-2 py-1 rounded">
          {data.blockType}
        </div>
      </div>

      {/* input handles */}
      {data.inputs.map(
        (i: { name: string; type: string }, idx: number) => (
          <Handle
            key={i.name}
            type="target"
            position={Position.Left}
            id={i.name}
            style={{
              top: 30 + idx * 20,
              background: getTypeColor(i.type),
            }}
            className="w-3 h-3"
          />
        ),
      )}

      {/* output handles */}
      {data.outputs.map(
        (o: { name: string; type: string }, idx: number) => (
          <Handle
            key={o.name}
            type="source"
            position={Position.Right}
            id={o.name}
            style={{
              top: 30 + idx * 20,
              background: getTypeColor(o.type),
            }}
            className="w-3 h-3"
          />
        ),
      )}
    </div>
  )
}

export default memo(BlockNode)
