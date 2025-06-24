import { memo, FC } from 'react'
import { Handle, Position, NodeProps } from '@xyflow/react'
import { getBlockIcon } from '@/utils/blockIcons'
import { getTypeColor } from '@/utils/colors'

const BlockNode: FC<NodeProps> = ({ data, selected }) => {
  const Icon = getBlockIcon(data.blockType)

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
          <div className="text-sm font-medium">{data.label}</div>
        </div>
        <div className="text-xs text-gray-500 bg-gray-100 px-2 py-1 rounded">
          {data.blockType}
        </div>
      </div>

      {/* Input handles */}
      {data.inputs?.map((input: any, index: number) => (
        <Handle
          key={`input-${index}`}
          type="target"
          position={Position.Left}
          id={input.name}
          style={{
            top: `${30 + index * 20}px`,
            background: getTypeColor(input.type),
          }}
          className="w-3 h-3"
        />
      ))}

      {/* Output handles */}
      {data.outputs?.map((output: any, index: number) => (
        <Handle
          key={`output-${index}`}
          type="source"
          position={Position.Right}
          id={output.name}
          style={{
            top: `${30 + index * 20}px`,
            background: getTypeColor(output.type),
          }}
          className="w-3 h-3"
        />
      ))}
    </div>
  )
}

export default memo(BlockNode)
