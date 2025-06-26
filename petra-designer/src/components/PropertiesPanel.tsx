// src/components/PropertiesPanel.tsx
import { useFlowStore } from '@/store/flowStore'
import { BLOCK_TYPES } from '@/utils/blockIcons'
import type { Node } from '@xyflow/react'
import type {
  SignalNodeData,
  BlockNodeData,
  TwilioNodeData,
  MqttNodeData,
  S7NodeData,
} from '@/types/nodes'
import {
  isSignalNode,
  isBlockNode,
  isTwilioNode,
  isMqttNode,
  isS7Node,
} from '@/types/nodes'

type CategoryMap = Record<string, Array<{
  value: string
  label: string
  category: string
}>>

const blockTypesByCategory: CategoryMap = BLOCK_TYPES.reduce((acc, t) => {
  if (!acc[t.category]) acc[t.category] = []
  acc[t.category].push(t)
  return acc
}, {} as CategoryMap)

export default function PropertiesPanel() {
  const { selectedNode, updateNodeData } = useFlowStore()

  if (!selectedNode) return null
  
  const node = selectedNode as Node

  // CRITICAL FIX: Extract the value immediately to avoid synthetic event issues
  function handleChange(field: string) {
    return (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>) => {
      const value = e.target.type === 'checkbox' ? (e.target as HTMLInputElement).checked : e.target.value
      updateNodeData(node.id, { [field]: value })
    }
  }

  function handleNumberChange(field: string) {
    return (e: React.ChangeEvent<HTMLInputElement>) => {
      const value = parseFloat(e.target.value) || 0
      updateNodeData(node.id, { [field]: value })
    }
  }

  // Helper to ensure label is a string
  const getLabel = (data: any): string => {
    return (data.label as string) || ''
  }

  const labelInput = (
    <div>
      <label className="block text-sm font-medium text-gray-700 mb-1">Label</label>
      <input
        type="text"
        value={getLabel(node.data)}
        onChange={handleChange('label')}
        className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
      />
    </div>
  )

  return (
    <div className="w-80 bg-white border-l border-gray-200 p-4 overflow-y-auto">
      <h3 className="text-lg font-semibold mb-4">Properties</h3>

      <div className="space-y-4">
        {labelInput}

        {/* Signal node */}
        {isSignalNode(node) && (
          <>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Signal Type
              </label>
              <select
                value={(node.data as SignalNodeData).signalType || 'float'}
                onChange={handleChange('signalType')}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
              >
                <option value="bool">Boolean</option>
                <option value="int">Integer</option>
                <option value="float">Float</option>
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Initial Value
              </label>
              {(node.data as SignalNodeData).signalType === 'bool' ? (
                <label className="flex items-center">
                  <input
                    type="checkbox"
                    checked={Boolean((node.data as SignalNodeData).initial)}
                    onChange={handleChange('initial')}
                    className="mr-2"
                  />
                  <span>{(node.data as SignalNodeData).initial ? 'True' : 'False'}</span>
                </label>
              ) : (
                <input
                  type="number"
                  value={Number((node.data as SignalNodeData).initial) || 0}
                  onChange={handleNumberChange('initial')}
                  step={(node.data as SignalNodeData).signalType === 'float' ? '0.1' : '1'}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
                />
              )}
            </div>
          </>
        )}

        {/* Block node with better params handling */}
        {isBlockNode(node) && (
          <>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Block Type
              </label>
              <select
                value={(node.data as BlockNodeData).blockType || 'AND'}
                onChange={handleChange('blockType')}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
              >
                {Object.entries(blockTypesByCategory).map(([cat, types]) => (
                  <optgroup key={cat} label={cat}>
                    {types.map((t) => (
                      <option key={t.value} value={t.value}>
                        {t.label}
                      </option>
                    ))}
                  </optgroup>
                ))}
              </select>
            </div>

            {/* Block-specific parameters */}
            {renderBlockParams(node.data as BlockNodeData, updateNodeData, node.id)}
          </>
        )}

        {/* Twilio node - Fixed validation */}
        {isTwilioNode(node) && (
          <>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Action Type
              </label>
              <select
                value={(node.data as TwilioNodeData).actionType || 'sms'}
                onChange={handleChange('actionType')}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
              >
                <option value="sms">SMS</option>
                <option value="call">Voice Call</option>
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                To Number
              </label>
              <input
                type="tel"
                value={(node.data as TwilioNodeData).toNumber || ''}
                onChange={handleChange('toNumber')}
                placeholder="+1234567890"
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Content
              </label>
              <textarea
                value={(node.data as TwilioNodeData).content || ''}
                onChange={handleChange('content')}
                rows={4}
                placeholder={
                  (node.data as TwilioNodeData).actionType === 'call'
                    ? 'Enter message or TwiML'
                    : 'Enter SMS message'
                }
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
              />
            </div>

            <div className="mt-2">
              <button
                onClick={() => {
                  const isValid = !!(node.data as TwilioNodeData).toNumber && 
                                 !!(node.data as TwilioNodeData).content
                  updateNodeData(node.id, { configured: isValid })
                }}
                className="px-3 py-1 bg-petra-500 text-white rounded hover:bg-petra-600"
              >
                Validate Configuration
              </button>
            </div>
          </>
        )}

        {/* MQTT node */}
        {isMqttNode(node) && (
          <>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Broker Host
              </label>
              <input
                type="text"
                value={(node.data as MqttNodeData).brokerHost || ''}
                onChange={handleChange('brokerHost')}
                placeholder="mqtt.lithos.systems"
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Broker Port
              </label>
              <input
                type="number"
                value={(node.data as MqttNodeData).brokerPort || 1883}
                onChange={handleNumberChange('brokerPort')}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Client ID
              </label>
              <input
                type="text"
                value={(node.data as MqttNodeData).clientId || ''}
                onChange={handleChange('clientId')}
                placeholder="petra-01"
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Topic Prefix
              </label>
              <input
                type="text"
                value={(node.data as MqttNodeData).topicPrefix || ''}
                onChange={handleChange('topicPrefix')}
                placeholder="petra/plc"
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
              />
            </div>

            <div className="mt-2">
              <button
                onClick={() => {
                  const isValid = !!(node.data as MqttNodeData).brokerHost && 
                                 !!(node.data as MqttNodeData).clientId
                  updateNodeData(node.id, { configured: isValid })
                }}
                className="px-3 py-1 bg-petra-500 text-white rounded hover:bg-petra-600"
              >
                Validate Configuration
              </button>
            </div>
          </>
        )}

        {/* S7 node */}
        {isS7Node(node) && (
          <>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Signal Name
              </label>
              <input
                type="text"
                value={(node.data as S7NodeData).signal || ''}
                onChange={handleChange('signal')}
                placeholder="motor_running"
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Area
              </label>
              <select
                value={(node.data as S7NodeData).area || 'DB'}
                onChange={handleChange('area')}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
              >
                <option value="DB">DB – Data Block</option>
                <option value="I">I – Inputs</option>
                <option value="Q">Q – Outputs</option>
                <option value="M">M – Markers</option>
              </select>
            </div>

            {(node.data as S7NodeData).area === 'DB' && (
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  DB Number
                </label>
                <input
                  type="number"
                  value={(node.data as S7NodeData).dbNumber || 0}
                  onChange={handleNumberChange('dbNumber')}
                  min="0"
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
                />
              </div>
            )}

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Address
              </label>
              <input
                type="number"
                value={(node.data as S7NodeData).address || 0}
                onChange={handleNumberChange('address')}
                min="0"
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Data Type
              </label>
              <select
                value={(node.data as S7NodeData).dataType || 'bool'}
                onChange={handleChange('dataType')}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
              >
                <option value="bool">Bool</option>
                <option value="byte">Byte</option>
                <option value="word">Word</option>
                <option value="int">Int</option>
                <option value="dint">DInt</option>
                <option value="real">Real</option>
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Direction
              </label>
              <select
                value={(node.data as S7NodeData).direction || 'read'}
                onChange={handleChange('direction')}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
              >
                <option value="read">Read</option>
                <option value="write">Write</option>
                <option value="read_write">Read/Write</option>
              </select>
            </div>

            <div className="mt-2">
              <button
                onClick={() => {
                  const isValid = !!(node.data as S7NodeData).signal
                  updateNodeData(node.id, { configured: isValid })
                }}
                className="px-3 py-1 bg-petra-500 text-white rounded hover:bg-petra-600"
              >
                Validate Configuration
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  )
}

// Helper function to render block-specific parameters
function renderBlockParams(data: BlockNodeData, updateNodeData: any, nodeId: string) {
  const blockType = data.blockType
  const params = data.params || {}

  // Helper to get number value from params
  const getParamNumber = (key: string, defaultValue: number): number => {
    const value = params[key]
    if (typeof value === 'number') return value
    if (typeof value === 'string') return parseFloat(value) || defaultValue
    return defaultValue
  }

  switch (blockType) {
    case 'TON':
    case 'TOF':
      return (
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Timer Preset (ms)
          </label>
          <input
            type="number"
            value={getParamNumber('preset_ms', 1000)}
            onChange={(e) => {
              const value = parseInt(e.target.value) || 1000
              updateNodeData(nodeId, { 
                params: { ...params, preset_ms: value } 
              })
            }}
            min="0"
            step="100"
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
          />
        </div>
      )

    case 'COUNTER':
      return (
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Increment Value
          </label>
          <input
            type="number"
            value={getParamNumber('increment', 1)}
            onChange={(e) => {
              const value = parseInt(e.target.value) || 1
              updateNodeData(nodeId, { 
                params: { ...params, increment: value } 
              })
            }}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
          />
        </div>
      )

    case 'DATA_GENERATOR':
      return (
        <>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Frequency (Hz)
            </label>
            <input
              type="number"
              value={getParamNumber('frequency', 1.0)}
              onChange={(e) => {
                const value = parseFloat(e.target.value) || 1.0
                updateNodeData(nodeId, { 
                  params: { ...params, frequency: value } 
                })
              }}
              min="0"
              step="0.1"
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Amplitude
            </label>
            <input
              type="number"
              value={getParamNumber('amplitude', 10.0)}
              onChange={(e) => {
                const value = parseFloat(e.target.value) || 10.0
                updateNodeData(nodeId, { 
                  params: { ...params, amplitude: value } 
                })
              }}
              min="0"
              step="0.1"
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
            />
          </div>
        </>
      )

    default:
      return null
  }
}
