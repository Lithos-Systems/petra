import { memo } from 'react'
import { Handle, Position, NodeProps } from '@xyflow/react'
import type { BlockNodeData } from '@/types/nodes'

function BlockNode({ data, selected }: NodeProps) {
  const blockData = data as BlockNodeData

  // ISA-101 Block Graphics (same as sidebar but larger for canvas)
  const renderBlockGraphic = (blockType: string) => {
    switch (blockType) {
      case 'and':
        return (
          <svg width="60" height="40" viewBox="0 0 60 40" className="block-graphic">
            <path d="M5 5 L25 5 A15 15 0 0 1 25 35 L5 35 Z" fill="#FFFFFF" stroke="#000000" strokeWidth="2"/>
            <text x="15" y="23" fontSize="10" textAnchor="middle" fill="#000000" fontWeight="bold">AND</text>
          </svg>
        )
      case 'or':
        return (
          <svg width="60" height="40" viewBox="0 0 60 40" className="block-graphic">
            <path d="M5 5 Q20 5 25 20 Q20 35 5 35 Q15 20 5 5" fill="#FFFFFF" stroke="#000000" strokeWidth="2"/>
            <text x="15" y="23" fontSize="10" textAnchor="middle" fill="#000000" fontWeight="bold">OR</text>
          </svg>
        )
      case 'not':
        return (
          <svg width="60" height="40" viewBox="0 0 60 40" className="block-graphic">
            <path d="M5 5 L5 35 L30 20 Z" fill="#FFFFFF" stroke="#000000" strokeWidth="2"/>
            <circle cx="35" cy="20" r="5" fill="#FFFFFF" stroke="#000000" strokeWidth="2"/>
            <text x="15" y="23" fontSize="8" textAnchor="middle" fill="#000000" fontWeight="bold">NOT</text>
          </svg>
        )
      case 'xor':
        return (
          <svg width="60" height="40" viewBox="0 0 60 40" className="block-graphic">
            <path d="M5 5 Q20 5 25 20 Q20 35 5 35" fill="#FFFFFF" stroke="#000000" strokeWidth="2"/>
            <path d="M2 5 Q17 5 22 20 Q17 35 2 35" fill="none" stroke="#000000" strokeWidth="2"/>
            <text x="15" y="23" fontSize="9" textAnchor="middle" fill="#000000" fontWeight="bold">XOR</text>
          </svg>
        )
      case 'greater_than':
        return (
          <svg width="60" height="40" viewBox="0 0 60 40" className="block-graphic">
            <rect x="5" y="5" width="40" height="30" fill="#FFFFFF" stroke="#000000" strokeWidth="2"/>
            <text x="25" y="23" fontSize="12" textAnchor="middle" fill="#000000" fontWeight="bold">{">"}</text>
          </svg>
        )
      case 'less_than':
        return (
          <svg width="60" height="40" viewBox="0 0 60 40" className="block-graphic">
            <rect x="5" y="5" width="40" height="30" fill="#FFFFFF" stroke="#000000" strokeWidth="2"/>
            <text x="25" y="23" fontSize="12" textAnchor="middle" fill="#000000" fontWeight="bold">{"<"}</text>
          </svg>
        )
      case 'equal':
        return (
          <svg width="60" height="40" viewBox="0 0 60 40" className="block-graphic">
            <rect x="5" y="5" width="40" height="30" fill="#FFFFFF" stroke="#000000" strokeWidth="2"/>
            <text x="25" y="23" fontSize="12" textAnchor="middle" fill="#000000" fontWeight="bold">=</text>
          </svg>
        )
      case 'add':
        return (
          <svg width="60" height="40" viewBox="0 0 60 40" className="block-graphic">
            <rect x="5" y="5" width="40" height="30" fill="#FFFFFF" stroke="#000000" strokeWidth="2"/>
            <text x="25" y="23" fontSize="14" textAnchor="middle" fill="#000000" fontWeight="bold">+</text>
          </svg>
        )
      case 'subtract':
        return (
          <svg width="60" height="40" viewBox="0 0 60 40" className="block-graphic">
            <rect x="5" y="5" width="40" height="30" fill="#FFFFFF" stroke="#000000" strokeWidth="2"/>
            <text x="25" y="23" fontSize="14" textAnchor="middle" fill="#000000" fontWeight="bold">-</text>
          </svg>
        )
      case 'multiply':
        return (
          <svg width="60" height="40" viewBox="0 0 60 40" className="block-graphic">
            <rect x="5" y="5" width="40" height="30" fill="#FFFFFF" stroke="#000000" strokeWidth="2"/>
            <text x="25" y="23" fontSize="14" textAnchor="middle" fill="#000000" fontWeight="bold">×</text>
          </svg>
        )
      case 'divide':
        return (
          <svg width="60" height="40" viewBox="0 0 60 40" className="block-graphic">
            <rect x="5" y="5" width="40" height="30" fill="#FFFFFF" stroke="#000000" strokeWidth="2"/>
            <text x="25" y="23" fontSize="14" textAnchor="middle" fill="#000000" fontWeight="bold">÷</text>
          </svg>
        )
      case 'timer':
        return (
          <svg width="60" height="40" viewBox="0 0 60 40" className="block-graphic">
            <rect x="5" y="5" width="40" height="30" fill="#FFFFFF" stroke="#000000" strokeWidth="2"/>
            <circle cx="25" cy="20" r="10" fill="none" stroke="#000000" strokeWidth="2"/>
            <path d="M25 15 L25 20 L30 20" stroke="#000000" strokeWidth="2"/>
            <text x="25" y="33" fontSize="6" textAnchor="middle" fill="#000000" fontWeight="bold">TIMER</text>
          </svg>
        )
      case 'delay':
        return (
          <svg width="60" height="40" viewBox="0 0 60 40" className="block-graphic">
            <rect x="5" y="5" width="40" height="30" fill="#FFFFFF" stroke="#000000" strokeWidth="2"/>
            <text x="25" y="18" fontSize="8" textAnchor="middle" fill="#000000" fontWeight="bold">DELAY</text>
            <text x="25" y="28" fontSize="6" textAnchor="middle" fill="#000000">T#10s</text>
          </svg>
        )
      case 'pid':
        return (
          <svg width="60" height="40" viewBox="0 0 60 40" className="block-graphic">
            <rect x="5" y="5" width="40" height="30" fill="#FFFFFF" stroke="#000000" strokeWidth="2"/>
            <text x="25" y="18" fontSize="10" textAnchor="middle" fill="#000000" fontWeight="bold">PID</text>
            <text x="25" y="28" fontSize="6" textAnchor="middle" fill="#000000">CTRL</text>
          </svg>
        )
      case 'controller':
        return (
          <svg width="60" height="40" viewBox="0 0 60 40" className="block-graphic">
            <rect x="5" y="5" width="40" height="30" fill="#FFFFFF" stroke="#000000" strokeWidth="2"/>
            <text x="25" y="23" fontSize="8" textAnchor="middle" fill="#000000" fontWeight="bold">CTRL</text>
          </svg>
        )
      case 'input':
        return (
          <svg width="60" height="40" viewBox="0 0 60 40" className="block-graphic">
            <path d="M5 20 L20 5 L20 12 L45 12 L45 28 L20 28 L20 35 Z" fill="#FFFFFF" stroke="#000000" strokeWidth="2"/>
            <text x="32" y="23" fontSize="8" textAnchor="middle" fill="#000000" fontWeight="bold">IN</text>
          </svg>
        )
      case 'output':
        return (
          <svg width="60" height="40" viewBox="0 0 60 40" className="block-graphic">
            <path d="M45 20 L30 5 L30 12 L5 12 L5 28 L30 28 L30 35 Z" fill="#FFFFFF" stroke="#000000" strokeWidth="2"/>
            <text x="18" y="23" fontSize="8" textAnchor="middle" fill="#000000" fontWeight="bold">OUT</text>
          </svg>
        )
      case 'analog_input':
        return (
          <svg width="60" height="40" viewBox="0 0 60 40" className="block-graphic">
            <rect x="5" y="5" width="40" height="30" fill="#FFFFFF" stroke="#000000" strokeWidth="2"/>
            <path d="M10 25 L15 15 L20 25 L25 15 L30 25 L35 15 L40 25" fill="none" stroke="#000000" strokeWidth="2"/>
            <text x="25" y="33" fontSize="6" textAnchor="middle" fill="#000000">ANALOG</text>
          </svg>
        )
      case 'digital_input':
        return (
          <svg width="60" height="40" viewBox="0 0 60 40" className="block-graphic">
            <rect x="5" y="5" width="40" height="30" fill="#FFFFFF" stroke="#000000" strokeWidth="2"/>
            <path d="M10 25 L10 15 L15 15 L15 25 L20 25 L20 15 L25 15 L25 25 L30 25 L30 15 L35 15 L35 25 L40 25" 
                  fill="none" stroke="#000000" strokeWidth="2"/>
            <text x="25" y="33" fontSize="6" textAnchor="middle" fill="#000000">DIGITAL</text>
          </svg>
        )
      case 'modbus':
        return (
          <svg width="60" height="40" viewBox="0 0 60 40" className="block-graphic">
            <rect x="5" y="5" width="40" height="30" fill="#FFFFFF" stroke="#000000" strokeWidth="2"/>
            <text x="25" y="18" fontSize="7" textAnchor="middle" fill="#000000" fontWeight="bold">MODBUS</text>
            <text x="25" y="28" fontSize="5" textAnchor="middle" fill="#000000">TCP/RTU</text>
          </svg>
        )
      case 's7':
        return (
          <svg width="60" height="40" viewBox="0 0 60 40" className="block-graphic">
            <rect x="5" y="5" width="40" height="30" fill="#FFFFFF" stroke="#000000" strokeWidth="2"/>
            <text x="25" y="20" fontSize="10" textAnchor="middle" fill="#000000" fontWeight="bold">S7</text>
            <text x="25" y="30" fontSize="5" textAnchor="middle" fill="#000000">SIEMENS</text>
          </svg>
        )
      case 'mqtt':
        return (
          <svg width="60" height="40" viewBox="0 0 60 40" className="block-graphic">
            <rect x="5" y="5" width="40" height="30" fill="#FFFFFF" stroke="#000000" strokeWidth="2"/>
            <text x="25" y="20" fontSize="8" textAnchor="middle" fill="#000000" fontWeight="bold">MQTT</text>
            <circle cx="15" cy="12" r="2" fill="#000000"/>
            <circle cx="25" cy="12" r="2" fill="#000000"/>
            <circle cx="35" cy="12" r="2" fill="#000000"/>
          </svg>
        )
      case 'signal':
        return (
          <svg width="60" height="40" viewBox="0 0 60 40" className="block-graphic">
            <circle cx="25" cy="20" r="15" fill="#FFFFFF" stroke="#000000" strokeWidth="2"/>
            <text x="25" y="23" fontSize="8" textAnchor="middle" fill="#000000" fontWeight="bold">SIG</text>
          </svg>
        )
      default:
        return (
          <svg width="60" height="40" viewBox="0 0 60 40" className="block-graphic">
            <rect x="5" y="5" width="40" height="30" fill="#FFFFFF" stroke="#000000" strokeWidth="2"/>
            <text x="25" y="23" fontSize="8" textAnchor="middle" fill="#000000" fontWeight="bold">{blockType?.toUpperCase() || 'BLOCK'}</text>
          </svg>
        )
    }
  }

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

export default memo(BlockNode)
