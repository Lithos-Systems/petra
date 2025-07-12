// petra-designer/src/nodes/BlockNode.tsx
import React from 'react'
import { Handle, Position, type NodeProps } from '@xyflow/react'
import type { BlockNodeData } from '@/types/nodes'

// PETRA backend block configurations
const PETRA_BLOCKS = {
  // Logic blocks - with variable inputs
  AND: { 
    symbol: '&', 
    minInputs: 2, 
    maxInputs: 8, 
    outputs: ['out'],
    category: 'logic',
    description: 'AND gate - all inputs must be true'
  },
  OR: { 
    symbol: '≥1', 
    minInputs: 2, 
    maxInputs: 8, 
    outputs: ['out'],
    category: 'logic',
    description: 'OR gate - at least one input must be true'
  },
  NOT: { 
    symbol: '1', 
    inputs: ['in'], 
    outputs: ['out'],
    category: 'logic',
    description: 'NOT gate - inverts input'
  },
  XOR: { 
    symbol: '=1', 
    inputs: ['a', 'b'], 
    outputs: ['out'],
    category: 'logic',
    description: 'XOR gate - odd number of true inputs'
  },
  
  // Comparison blocks
  GT: { symbol: '>', inputs: ['a', 'b'], outputs: ['out'], category: 'compare' },
  LT: { symbol: '<', inputs: ['a', 'b'], outputs: ['out'], category: 'compare' },
  GTE: { symbol: '≥', inputs: ['a', 'b'], outputs: ['out'], category: 'compare' },
  LTE: { symbol: '≤', inputs: ['a', 'b'], outputs: ['out'], category: 'compare' },
  EQ: { symbol: '=', inputs: ['a', 'b'], outputs: ['out'], category: 'compare' },
  NEQ: { symbol: '≠', inputs: ['a', 'b'], outputs: ['out'], category: 'compare' },
  
  // Math blocks
  ADD: { symbol: '+', inputs: ['a', 'b'], outputs: ['out'], category: 'math' },
  SUB: { symbol: '-', inputs: ['a', 'b'], outputs: ['out'], category: 'math' },
  MUL: { symbol: '×', inputs: ['a', 'b'], outputs: ['out'], category: 'math' },
  DIV: { symbol: '÷', inputs: ['a', 'b'], outputs: ['out'], category: 'math' },
  
  // Timer blocks
  ON_DELAY: { 
    symbol: 'TON', 
    inputs: ['in', 'pt'], 
    outputs: ['out', 'et'],
    category: 'timer',
    description: 'Timer On Delay'
  },
  OFF_DELAY: { 
    symbol: 'TOF', 
    inputs: ['in', 'pt'], 
    outputs: ['out', 'et'],
    category: 'timer',
    description: 'Timer Off Delay'
  },
  PULSE: { 
    symbol: 'TP', 
    inputs: ['in', 'pt'], 
    outputs: ['out', 'et'],
    category: 'timer',
    description: 'Pulse Timer'
  },
  
  // Data blocks
  SCALE: { 
    symbol: 'SCL', 
    inputs: ['in'], 
    outputs: ['out'],
    category: 'data',
    params: ['in_min', 'in_max', 'out_min', 'out_max']
  },
  LIMIT: { 
    symbol: 'LIM', 
    inputs: ['in'], 
    outputs: ['out'],
    category: 'data',
    params: ['min', 'max']
  },
  SELECT: { 
    symbol: 'SEL', 
    inputs: ['sel', 'a', 'b'], 
    outputs: ['out'],
    category: 'data'
  },
  MUX: { 
    symbol: 'MUX', 
    inputs: ['sel', 'i0', 'i1', 'i2', 'i3'], 
    outputs: ['out'],
    category: 'data'
  },
  DEMUX: { 
    symbol: 'DMUX', 
    inputs: ['sel', 'in'], 
    outputs: ['o0', 'o1', 'o2', 'o3'],
    category: 'data'
  },
  
  // Control blocks
  PID: { 
    symbol: 'PID', 
    inputs: ['pv', 'sp', 'man', 'reset'], 
    outputs: ['out', 'err'],
    category: 'control',
    params: ['kp', 'ki', 'kd', 'out_min', 'out_max']
  },
  RAMP: { 
    symbol: 'RMP', 
    inputs: ['in', 'run'], 
    outputs: ['out'],
    category: 'control',
    params: ['rate_up', 'rate_down']
  },
  LEADLAG: { 
    symbol: 'LL', 
    inputs: ['in'], 
    outputs: ['out'],
    category: 'control',
    params: ['lead_time', 'lag_time']
  },
  
  // Additional blocks from PETRA backend
  AVG: { 
    symbol: 'AVG', 
    inputs: ['in'], 
    outputs: ['out', 'count'],
    category: 'statistics',
    params: ['window_size']
  },
  MIN_MAX: { 
    symbol: 'MM', 
    inputs: ['in', 'reset'], 
    outputs: ['min', 'max'],
    category: 'statistics'
  },
  STDDEV: { 
    symbol: 'σ', 
    inputs: ['in'], 
    outputs: ['out', 'mean'],
    category: 'statistics',
    params: ['window_size']
  }
}

export default function BlockNode({ data, selected }: NodeProps<BlockNodeData>) {
  const blockType = data.blockType
  const config = PETRA_BLOCKS[blockType as keyof typeof PETRA_BLOCKS]
  
  if (!config) return null
  
  // Generate inputs based on configuration
  const getInputs = () => {
    // For AND/OR gates with variable inputs
    if ((blockType === 'AND' || blockType === 'OR') && 'minInputs' in config) {
      const inputCount = data.inputCount || config.minInputs
      const inputs = []
      for (let i = 0; i < inputCount; i++) {
        inputs.push(String.fromCharCode(97 + i)) // a, b, c, d...
      }
      return inputs
    }
    // For fixed input blocks
    return config.inputs || []
  }
  
  const inputs = getInputs()
  const outputs = config.outputs || []
  
  // Calculate handle spacing
  const inputSpacing = 100 / (inputs.length + 1)
  const outputSpacing = 100 / (outputs.length + 1)
  
  return (
    <div
      className={`
        relative bg-white border-2 rounded-md min-w-[120px] min-h-[80px] 
        flex flex-col items-center justify-center p-3
        ${selected ? 'border-blue-600 shadow-lg' : 'border-gray-700'}
        ${data.status === 'running' ? 'bg-green-50' : 'bg-gray-50'}
      `}
    >
      {/* Input handles - larger and more visible */}
      {inputs.map((input, index) => (
        <Handle
          key={`input-${input}`}
          id={input}
          type="target"
          position={Position.Left}
          style={{
            top: `${inputSpacing * (index + 1)}%`,
            left: '-10px',
            width: '20px',
            height: '20px',
            background: '#404040',
            border: '2px solid #000',
            borderRadius: '50%',
            cursor: 'crosshair',
          }}
          className="hover:bg-blue-500 hover:scale-125 transition-all duration-200"
          title={`Input ${input}`}
        />
      ))}
      
      {/* Block content */}
      <div className="text-center">
        <div className="text-2xl font-bold text-gray-800">{config.symbol}</div>
        <div className="text-xs text-gray-600 mt-1">{data.label}</div>
        {config.description && (
          <div className="text-xs text-gray-500 mt-1 italic">{config.description}</div>
        )}
      </div>
      
      {/* Output handles - larger and more visible */}
      {outputs.map((output, index) => (
        <Handle
          key={`output-${output}`}
          id={output}
          type="source"
          position={Position.Right}
          style={{
            top: `${outputSpacing * (index + 1)}%`,
            right: '-10px',
            width: '20px',
            height: '20px',
            background: '#404040',
            border: '2px solid #000',
            borderRadius: '50%',
            cursor: 'crosshair',
          }}
          className="hover:bg-green-500 hover:scale-125 transition-all duration-200"
          title={`Output ${output}`}
        />
      ))}
      
      {/* Status indicator */}
      {data.status && (
        <div className={`
          absolute top-1 right-1 w-2 h-2 rounded-full
          ${data.status === 'running' ? 'bg-green-500' : 'bg-gray-400'}
        `} />
      )}
    </div>
  )
}

// Export block configurations for use in other components
export { PETRA_BLOCKS }
