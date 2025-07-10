import { memo } from 'react'
import { Handle, Position, NodeProps } from '@xyflow/react'
import { FaServer, FaCheckCircle, FaWifi, FaExclamationTriangle } from 'react-icons/fa'
import type { MqttNodeData } from '@/types/nodes'

function MqttNode({ data, selected }: NodeProps) {
  const mqttData = data as MqttNodeData

  // Show different handles based on mode
  const showInput = mqttData.mode === 'write' || mqttData.mode === 'read_write'
  const showOutput = mqttData.mode === 'read' || mqttData.mode === 'read_write'

  // Status indicator
  const getStatusColor = () => {
    if (!mqttData.configured) return '#808080' // Gray - not configured
    if (mqttData.brokerHost && mqttData.brokerPort && mqttData.clientId) {
      return '#00C800' // Green - configured
    }
    return '#FF8C00' // Orange - partially configured
  }

  const statusColor = getStatusColor()

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
          <FaServer className="w-4 h-4" style={{ color: '#404040' }} />
          <span className="text-sm font-medium text-black">MQTT</span>
        </div>
        <div className="flex items-center gap-1">
          {mqttData.configured ? (
            <FaCheckCircle className="w-3 h-3" style={{ color: '#00C800' }} />
          ) : (
            <FaExclamationTriangle className="w-3 h-3" style={{ color: '#FF8C00' }} />
          )}
          <FaWifi className="w-3 h-3" style={{ color: statusColor }} />
        </div>
      </div>

      {/* Content Section */}
      <div className="flex-1 px-3 py-2">
        {/* Node Label */}
        <div className="text-sm font-medium text-black mb-2">
          {mqttData.label}
        </div>

        {/* Configuration Display */}
        <div className="space-y-1 text-xs text-gray-700">
          <div className="flex justify-between">
            <span>Broker:</span>
            <span className="font-mono">
              {mqttData.brokerHost || 'localhost'}:{mqttData.brokerPort || 1883}
            </span>
          </div>
          
          <div className="flex justify-between">
            <span>Client ID:</span>
            <span className="font-mono truncate max-w-[100px]">
              {mqttData.clientId || 'petra_client'}
            </span>
          </div>

          <div className="flex justify-between">
            <span>Topic:</span>
            <span className="font-mono truncate max-w-[100px]">
              {mqttData.topicPrefix || 'petra'}/{mqttData.signalName || '?'}
            </span>
          </div>

          <div className="flex justify-between">
            <span>Mode:</span>
            <span className="font-medium" style={{ 
              color: mqttData.mode === 'read_write' ? '#000080' : 
                     mqttData.mode === 'write' ? '#800080' : '#008000'
            }}>
              {mqttData.mode?.toUpperCase() || 'READ_WRITE'}
            </span>
          </div>

          {mqttData.username && (
            <div className="flex justify-between">
              <span>User:</span>
              <span className="font-mono truncate max-w-[100px]">{mqttData.username}</span>
            </div>
          )}

          {mqttData.publishOnChange && (
            <div className="text-xs" style={{ color: '#008000' }}>
              ✓ Publish on Change
            </div>
          )}
        </div>
      </div>

      {/* Input Handle for writing to MQTT */}
      {showInput && (
        <Handle
          type="target"
          position={Position.Left}
          id="value"
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

      {/* Output Handle for reading from MQTT */}
      {showOutput && (
        <Handle
          type="source"
          position={Position.Right}
          id="value"
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
        title={mqttData.configured ? 'Configured' : 'Not Configured'}
      />
    </div>
  )
}

export default memo(MqttNode)
