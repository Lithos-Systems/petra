// petra-designer/src/nodes/MqttNode.tsx
import React from 'react'
import { Handle, Position, type NodeProps, type Node } from '@xyflow/react'
import type { MqttNodeData } from '@/types/nodes'
import { FaWifi, FaExclamationTriangle } from 'react-icons/fa'

export default function MqttNode({ data, selected }: NodeProps<Node<MqttNodeData>>) {
  const isConfigured = data.configured && data.brokerHost && data.brokerPort
  
  return (
    <div
      className={`
        relative bg-white border-2 rounded-md min-w-[160px] min-h-[120px] p-4
        ${selected ? 'border-blue-600 shadow-lg' : 'border-gray-700'}
        ${!isConfigured ? 'border-orange-500' : ''}
      `}
    >
      {/* Status indicator */}
      <div className="absolute top-2 right-2">
        {isConfigured ? (
          <FaWifi className="w-4 h-4 text-green-500" title="Configured" />
        ) : (
          <FaExclamationTriangle className="w-4 h-4 text-orange-500" title="Not configured" />
        )}
      </div>
      
      {/* Input handle for triggering/data */}
      <Handle
        id="trigger"
        type="target"
        position={Position.Left}
        style={{
          top: '30%',
          left: '-10px',
          width: '20px',
          height: '20px',
          background: '#404040',
          border: '2px solid #000',
          borderRadius: '50%',
          cursor: 'crosshair',
        }}
        className="hover:bg-blue-500 hover:scale-125 transition-all duration-200"
        title="Trigger/Data Input"
      />
      
      {/* Value input for publish */}
      <Handle
        id="value"
        type="target"
        position={Position.Left}
        style={{
          top: '70%',
          left: '-10px',
          width: '20px',
          height: '20px',
          background: '#404040',
          border: '2px solid #000',
          borderRadius: '50%',
          cursor: 'crosshair',
        }}
        className="hover:bg-blue-500 hover:scale-125 transition-all duration-200"
        title="Value to Publish"
      />
      
      {/* Node content */}
      <div className="space-y-2">
        <div className="text-center">
          <div className="text-lg font-bold text-gray-800">MQTT</div>
          <div className="text-xs text-gray-600">{data.label}</div>
        </div>
        
        {/* Configuration display */}
        <div className="text-xs space-y-1">
          <div className="flex justify-between">
            <span className="text-gray-500">Broker:</span>
            <span className="text-gray-700 font-mono">
              {data.brokerHost || 'not set'}:{data.brokerPort || '1883'}
            </span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-500">Topic:</span>
            <span className="text-gray-700 font-mono truncate max-w-[100px]" title={data.topicPrefix}>
              {data.topicPrefix || 'not set'}
            </span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-500">Mode:</span>
            <span className="text-gray-700">
              {data.mode === 'read' && '📥 Sub'}
              {data.mode === 'write' && '📤 Pub'}
              {data.mode === 'read_write' && '🔄 Both'}
            </span>
          </div>
        </div>
        
        {/* Subscription/Publication info */}
        {data.subscriptions && data.subscriptions.length > 0 && (
          <div className="text-xs">
            <div className="text-gray-500">Subscriptions:</div>
            {data.subscriptions.slice(0, 2).map((sub: { topic: string }, i: number) => (
              <div key={i} className="text-gray-600 font-mono truncate" title={sub.topic}>
                {sub.topic}
              </div>
            ))}
            {data.subscriptions.length > 2 && (
              <div className="text-gray-400">+{data.subscriptions.length - 2} more</div>
            )}
          </div>
        )}
      </div>
      
      {/* Output handle for received data */}
      <Handle
        id="data"
        type="source"
        position={Position.Right}
        style={{
          top: '30%',
          right: '-10px',
          width: '20px',
          height: '20px',
          background: '#404040',
          border: '2px solid #000',
          borderRadius: '50%',
          cursor: 'crosshair',
        }}
        className="hover:bg-green-500 hover:scale-125 transition-all duration-200"
        title="Received Data Output"
      />
      
      {/* Status output */}
      <Handle
        id="status"
        type="source"
        position={Position.Right}
        style={{
          top: '70%',
          right: '-10px',
          width: '20px',
          height: '20px',
          background: '#404040',
          border: '2px solid #000',
          borderRadius: '50%',
          cursor: 'crosshair',
        }}
        className="hover:bg-green-500 hover:scale-125 transition-all duration-200"
        title="Connection Status"
      />
    </div>
  )
}
