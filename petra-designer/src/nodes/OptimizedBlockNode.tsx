import { memo } from 'react'
import { Handle, Position, type NodeProps } from '@xyflow/react'
import type { BlockNodeData } from '@/types/nodes'
import { renderBlockGraphic } from '@/utils/blockIcons'

function BlockNodeComponent({ data, selected }: NodeProps) {
  const blockData = data as BlockNodeData


  return (
    <div
      className={`
        relative bg-white min-w-[120px] min-h-[80px] flex flex-col items-center justify-center
        ${selected ? 'border-2 border-blue-800' : 'border-2 border-gray-700'}
      `}
      style={{
        borderColor: selected ? '#000080' : '#404040',
        backgroundColor: '#FFFFFF'
      }}
    >
      {/* Block Graphic */}
      <div className="mb-1">
        {renderBlockGraphic(blockData.blockType)}
      </div>

      {/* Block Label */}
      <div className="text-xs font-medium text-black text-center px-1">
        {blockData.label}
      </div>

      {/* Input Handles */}
      {blockData.inputs?.map((input, idx) => (
        <Handle
          key={input.name}
          type="target"
          position={Position.Left}
          id={input.name}
          style={{
            top: 20 + idx * 15,
            background: '#000000',
            border: '1px solid #FFFFFF',
            width: '8px',
            height: '8px',
            borderRadius: '0'
          }}
          className="handle-input"
        />
      ))}

      {/* Output Handles */}
      {blockData.outputs?.map((output, idx) => (
        <Handle
          key={output.name}
          type="source"
          position={Position.Right}
          id={output.name}
          style={{
            top: 20 + idx * 15,
            background: '#000000',
            border: '1px solid #FFFFFF',
            width: '8px',
            height: '8px',
            borderRadius: '0'
          }}
          className="handle-output"
        />
      ))}

      {/* Status indicator */}
      {blockData.params && (
        <div className="absolute top-1 right-1 w-2 h-2 bg-green-500 border border-black"></div>
      )}
    </div>
  )
}

export const OptimizedBlockNode = memo(BlockNodeComponent)
