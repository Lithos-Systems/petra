import { memo } from 'react'
import { Handle, Position, NodeProps } from '@xyflow/react'
import { FaIndustry, FaCheckCircle, FaPlug, FaExclamationTriangle } from 'react-icons/fa'
import type { ModbusNodeData } from '@/types/nodes'

function ModbusNode({ data, selected }: NodeProps) {
  const modbusData = data as ModbusNodeData

  // Status indicator
  const getStatusColor = () => {
    if (!modbusData.configured) return '#808080' // Gray - not configured
    if (modbusData.host && modbusData.port && modbusData.unitId !== undefined && modbusData.address !== undefined) {
      return '#00C800' // Green - configured
    }
    return '#FF8C00' // Orange - partially configured
  }

  const statusColor = getStatusColor()

  // Show handles based on direction
  const showInput = modbusData.direction === 'write' || modbusData.direction === 'read_write'
  const showOutput = modbusData.direction === 'read' || modbusData.direction === 'read_write'

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
          <span className="text-sm font-medium text-black">MODBUS</span>
        </div>
        <div className="flex items-center gap-1">
          {modbusData.configured ? (
            <FaCheckCircle className="w-3 h-3" style={{ color: '#00C800' }} />
          ) : (
            <FaExclamationTriangle className="w-3 h-3" style={{ color: '#FF8C00' }} />
          )}
          <FaPlug className="w-3 h-3" style={{ color: statusColor }} />
        </div>
      </div>

      {/* Content Section */}
      <div className="flex-1 px-3 py-2">
        {/* Node Label */}
        <div className="text-sm font-medium text-black mb-2">
          {modbusData.label}
        </div>

        {/* Configuration Display */}
        <div className="space-y-1 text-xs text-gray-700">
          <div className="flex justify-between">
            <span>Host:</span>
            <span className="font-mono">
              {modbusData.host || 'localhost'}:{modbusData.port || 502}
            </span>
          </div>
          
          <div className="flex justify-between">
            <span>Unit ID:</span>
            <span className="font-mono">
              {modbusData.unitId || 1}
            </span>
          </div>

          <div className="flex justify-between">
            <span>Register:</span>
            <span className="font-mono font-medium">
              {modbusData.dataType?.toUpperCase() || 'HOLDING'}.{modbusData.address || 0}
            </span>
          </div>

          <div className="flex justify-between">
            <span>Direction:</span>
            <span className="font-medium" style={{ 
              color: modbusData.direction === 'read_write' ? '#000080' : 
                     modbusData.direction === 'write' ? '#800080' : '#008000'
            }}>
              {modbusData.direction?.toUpperCase() || 'READ'}
            </span>
          </div>

          {modbusData.signal && (
            <div className="flex justify-between">
              <span>Signal:</span>
              <span className="font-mono truncate max-w-[100px]">{modbusData.signal}</span>
            </div>
          )}
        </div>
      </div>

      {/* Input Handle for writing to Modbus */}
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

      {/* Output Handle for reading from Modbus */}
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
        title={modbusData.configured ? 'Configured' : 'Not Configured'}
      />
    </div>
  )
}

export default memo(ModbusNode)
