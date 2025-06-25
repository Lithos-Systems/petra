import { memo } from 'react'
import { Handle, Position, NodeProps, Node } from '@xyflow/react'
import { getBlockIcon } from '@/utils/blockIcons'
import { getTypeColor } from '@/utils/colors'
import type { BlockNodeData } from '@/types/nodes'

function BlockNode({ data, selected }: NodeProps<Node<BlockNodeData>>) {
  const blockData = data as BlockNodeData
  const Icon = getBlockIcon(blockData.blockType)

  return (
    <div
      className={`
        px-4 py-3 shadow-md rounded-md border-2 bg-white min-w-[180px]
        ${selected ? 'border-petra-500' : 'border-gray-200'}
      `}
    >
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          <Icon className="w-4 h-4 text-petra-600" />
          <div className="text-sm font-medium">{blockData.label}</div>
        </div>
        <div className="text-xs text-gray-500 bg-gray-100 px-2 py-1 rounded">
          {blockData.blockType}
        </div>
      </div>

      {blockData.inputs.map((input, idx) => (
        <Handle
          key={input.name}
          type="target"
          position={Position.Left}
          id={input.name}
          style={{
            top: 30 + idx * 20,
            background: getTypeColor(input.type),
          }}
          className="w-3 h-3"
        />
      ))}

      {blockData.outputs.map((output, idx) => (
        <Handle
          key={output.name}
          type="source"
          position={Position.Right}
          id={output.name}
          style={{
            top: 30 + idx * 20,
            background: getTypeColor(output.type),
          }}
          className="w-3 h-3"
        />
      ))}
    </div>
  )
}

export default memo(BlockNode)
