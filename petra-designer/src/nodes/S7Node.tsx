import { memo } from 'react'
import { Handle, Position, NodeProps } from '@xyflow/react'
import { FaIndustry, FaCheckCircle, FaLink, FaExclamationTriangle } from 'react-icons/fa'
import type { S7NodeData } from '@/types/nodes'

function S7Node({ data, selected }: NodeProps) {
  const s7Data = data as S7NodeData

  // Status indicator
  const getStatusColor = () => {
    if (!s7Data.configured) return '#808080' // Gray - not configured
    if (s7Data.ip && s7Data.signal && s7Data.area && s7Data.address !== undefined) {
      return '#00C800' // Green - configured
    }
    return '#FF8C00' // Orange - partially configured
  }

  const statusColor = getStatusColor()

  // Format address display
  const formatAddress = () => {
    let addr = `${s7Data.area}`
    if (s7Data.area === 'DB' && s7Data.dbNumber) {
      addr += s7Data.dbNumber
    }
    addr += `.${s7Data.address || 0}`
    if (s7Data.dataType === 'bool' && s7Data.bit !== undefined) {
      addr += `.${s7Data.bit}`
    }
    return addr
  }

  // Show handles based on direction
  const showInput = s7Data.direction === 'write' || s7Data.direction === 'read_write'
  const showOutput = s7Data.direction === 'read' || s7Data.direction === 'read_write'

  return (
    <div
      className="relative bg-white min-w-[200px] min-h-[100px] flex flex-col"
      style={{
        border: selected ? '3px solid #000080' : '2px solid #404040',
        backgroundColor: '#FFFFFF'
      }}
    >
      {/* Header Section */}
      <div 
        className="flex items-center justify-between px-3 py-2 border-b border-gray-400"
        style={{ backgroundColor: '#E0E0E0' }}
      >
        <div className="flex items-center gap-2">
          <FaIndustry className="w-4 h-4" style={{ color: '#404040' }} />
          <span className="text-sm font-medium text-black">SIEMENS S7</span>
        </div>
        <div className="flex items-center gap-1">
          {s7Data.configured ? (
            <FaCheckCircle className="w-3 h-3" style={{ color: '#00C800' }} />
          ) : (
            <FaExclamationTriangle className="w-3 h-3" style={{ color: '#FF8C00' }} />
          )}
          <FaLink className="w-3 h-3" style={{ color: statusColor }} />
        </div>
      </div>

      {/* Content Section */}
      <div className="flex-1 px-3 py-2">
        {/* Node Label */}
        <div className="text-sm font-medium text-black mb-2">
          {s7Data.label}
        </div>

        {/* Configuration Display */}
        <div className="space-y-1 text-xs text-gray-700">
          <div className="flex justify-between">
            <span>PLC:</span>
            <span className="font-mono">
              {s7Data.ip || '192.168.1.100'}
            </span>
          </div>
          
          <div className="flex justify-between">
            <span>Rack/Slot:</span>
            <span className="font-mono">
              {s7Data.rack || 0}/{s7Data.slot || 1}
            </span>
          </div>

          <div className="flex justify-between">
            <span>Address:</span>
            <span className="font-mono font-medium">
              {formatAddress()}
            </span>
          </div>

          <div className="flex justify-between">
            <span>Type:</span>
            <span className="font-medium" style={{ color: '#000080' }}>
              {s7Data.dataType?.toUpperCase() || 'REAL'}
            </span>
          </div>

          <div className="flex justify-between">
            <span>Direction:</span>
            <span className="font-medium" style={{ 
              color: s7Data.direction === 'read_write' ? '#000080' : 
                     s7Data.direction === 'write' ? '#800080' : '#008000'
            }}>
              {s7Data.direction?.toUpperCase() || 'READ'}
            </span>
          </div>

          {s7Data.signal && (
            <div className="flex justify-between">
              <span>Signal:</span>
              <span className="font-mono truncate max-w-[100px]">{s7Data.signal}</span>
            </div>
          )}
        </div>
      </div>

      {/* Input Handle for writing to S7 */}
      {showInput && (
        <Handle
          type="target"
          position={Position.Left}
          id="signal"
          style={{ 
            background: '#404040',
            border: '1px solid #FFFFFF',
            width: '10px',
            height: '10px',
            borderRadius: '0',
            top: '50%'
          }}
          className="handle-input"
        />
      )}

      {/* Output Handle for reading from S7 */}
      {showOutput && (
        <Handle
          type="source"
          position={Position.Right}
          id="signal"
          style={{ 
            background: '#404040',
            border: '1px solid #FFFFFF',
            width: '10px',
            height: '10px',
            borderRadius: '0',
            top: '50%'
          }}
          className="handle-output"
        />
      )}

      {/* Status Indicator */}
      <div 
        className="absolute top-2 right-2 w-3 h-3 border border-black"
        style={{ backgroundColor: statusColor }}
        title={s7Data.configured ? 'Configured' : 'Not Configured'}
      />
    </div>
  )
}

export default memo(S7Node)
