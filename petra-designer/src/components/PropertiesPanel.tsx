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
import { validateNodeConfiguration } from '@/utils/validation'
import toast from 'react-hot-toast'

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

  // Extract handlers to reduce repetition
  const createChangeHandler = (field: string) => {
    return (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>) => {
      const value = e.target.type === 'checkbox' ? (e.target as HTMLInputElement).checked : e.target.value
      updateNodeData(node.id, { [field]: value })
    }
  }

  const createNumberHandler = (field: string) => {
    return (e: React.ChangeEvent<HTMLInputElement>) => {
      const value = parseFloat(e.target.value) || 0
      updateNodeData(node.id, { [field]: value })
    }
  }

  const validateConfiguration = () => {
    const validation = validateNodeConfiguration(node)
    if (validation.valid) {
      updateNodeData(node.id, { configured: true })
      toast.success('Configuration validated successfully')
    } else {
      updateNodeData(node.id, { configured: false })
      toast.error(validation.error || 'Invalid configuration')
    }
  }

  // Helper to ensure label is a string
  const getLabel = (data: any): string => {
    return (data.label as string) || ''
  }

  // Common input component factory
  const renderInput = (label: string, field: string, type: string = 'text', placeholder?: string, props?: any) => (
    <div>
      <label className="block text-sm font-medium text-gray-700 mb-1">{label}</label>
      <input
        type={type}
        value={node.data[field] || ''}
        onChange={createChangeHandler(field)}
        placeholder={placeholder}
        className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
        {...props}
      />
    </div>
  )

  const renderSelect = (label: string, field: string, options: Array<{value: string, label: string}>) => (
    <div>
      <label className="block text-sm font-medium text-gray-700 mb-1">{label}</label>
      <select
        value={node.data[field] || options[0].value}
        onChange={createChangeHandler(field)}
        className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
      >
        {options.map(opt => (
          <option key={opt.value} value={opt.value}>{opt.label}</option>
        ))}
      </select>
    </div>
  )

  return (
    <div className="w-80 bg-white border-l border-gray-200 p-4 overflow-y-auto">
      <h3 className="text-lg font-semibold mb-4">Properties</h3>

      <div className="space-y-4">
        {renderInput('Label', 'label')}

        {/* Signal node */}
        {isSignalNode(node) && (
          <>
            {renderSelect('Signal Type', 'signalType', [
              { value: 'bool', label: 'Boolean' },
              { value: 'int', label: 'Integer' },
              { value: 'float', label: 'Float' }
            ])}

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Initial Value
              </label>
              {(node.data as SignalNodeData).signalType === 'bool' ? (
                <label className="flex items-center">
                  <input
                    type="checkbox"
                    checked={Boolean((node.data as SignalNodeData).initial)}
                    onChange={createChangeHandler('initial')}
                    className="mr-2"
                  />
                  <span>{(node.data as SignalNodeData).initial ? 'True' : 'False'}</span>
                </label>
              ) : (
                <input
                  type="number"
                  value={Number((node.data as SignalNodeData).initial) || 0}
                  onChange={createNumberHandler('initial')}
                  step={(node.data as SignalNodeData).signalType === 'float' ? '0.1' : '1'}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
                />
              )}
            </div>
          </>
        )}

        {/* Block node */}
        {isBlockNode(node) && (
          <>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Block Type
              </label>
              <select
                value={(node.data as BlockNodeData).blockType || 'AND'}
                onChange={createChangeHandler('blockType')}
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

        {/* Twilio node */}
        {isTwilioNode(node) && (
          <>
            {renderSelect('Action Type', 'actionType', [
              { value: 'sms', label: 'SMS' },
              { value: 'call', label: 'Voice Call' }
            ])}
            {renderInput('To Number', 'toNumber', 'tel', '+1234567890')}
            
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Content
              </label>
              <textarea
                value={(node.data as TwilioNodeData).content || ''}
                onChange={createChangeHandler('content')}
                rows={4}
                placeholder={
                  (node.data as TwilioNodeData).actionType === 'call'
                    ? 'Enter message or TwiML'
                    : 'Enter SMS message'
                }
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
              />
            </div>

            <button
              onClick={validateConfiguration}
              className="w-full px-3 py-2 bg-petra-500 text-white rounded hover:bg-petra-600 transition-colors"
            >
              Validate Configuration
            </button>
          </>
        )}

        {/* Enhanced MQTT node with username/password */}
        {isMqttNode(node) && (
          <>
            {renderInput('Broker Host', 'brokerHost', 'text', 'mqtt.lithos.systems')}
            {renderInput('Broker Port', 'brokerPort', 'number')}
            {renderInput('Client ID', 'clientId', 'text', 'petra-01')}
            {renderInput('Topic Prefix', 'topicPrefix', 'text', 'petra/plc')}
            
            <div className="border-t pt-4 mt-4">
              <h4 className="text-sm font-medium text-gray-700 mb-2">Authentication (Optional)</h4>
              {renderInput('Username', 'username', 'text', 'Leave empty for anonymous')}
              {renderInput('Password', 'password', 'password', 'Leave empty for anonymous')}
            </div>

            {renderSelect('Mode', 'mode', [
              { value: 'read_write', label: 'Read/Write' },
              { value: 'read', label: 'Read Only' },
              { value: 'write', label: 'Write Only' }
            ])}

            <label className="flex items-center mt-2">
              <input
                type="checkbox"
                checked={Boolean((node.data as MqttNodeData).publishOnChange)}
                onChange={createChangeHandler('publishOnChange')}
                className="mr-2"
              />
              <span className="text-sm">Publish on change</span>
            </label>

            <button
              onClick={validateConfiguration}
              className="w-full px-3 py-2 bg-petra-500 text-white rounded hover:bg-petra-600 transition-colors"
            >
              Validate Configuration
            </button>
          </>
        )}

        {/* Enhanced S7 node with IP configuration */}
        {isS7Node(node) && (
          <>
            <div className="border-b pb-4 mb-4">
              <h4 className="text-sm font-medium text-gray-700 mb-2">Connection Settings</h4>
              {renderInput('IP Address', 'ip', 'text', '192.168.1.100')}
              <div className="grid grid-cols-2 gap-2 mt-2">
                {renderInput('Rack', 'rack', 'number')}
                {renderInput('Slot', 'slot', 'number')}
              </div>
            </div>

            {renderInput('Signal Name', 'signal', 'text', 'motor_running')}
            
            {renderSelect('Area', 'area', [
              { value: 'DB', label: 'DB – Data Block' },
              { value: 'I', label: 'I – Inputs' },
              { value: 'Q', label: 'Q – Outputs' },
              { value: 'M', label: 'M – Markers' }
            ])}

            {(node.data as S7NodeData).area === 'DB' && (
              renderInput('DB Number', 'dbNumber', 'number')
            )}

            {renderInput('Address', 'address', 'number')}

            {renderSelect('Data Type', 'dataType', [
              { value: 'bool', label: 'Bool' },
              { value: 'byte', label: 'Byte' },
              { value: 'word', label: 'Word' },
              { value: 'int', label: 'Int' },
              { value: 'dint', label: 'DInt' },
              { value: 'real', label: 'Real' }
            ])}

            {(node.data as S7NodeData).dataType === 'bool' && (
              renderInput('Bit', 'bit', 'number')
            )}

            {renderSelect('Direction', 'direction', [
              { value: 'read', label: 'Read' },
              { value: 'write', label: 'Write' },
              { value: 'read_write', label: 'Read/Write' }
            ])}

            <button
              onClick={validateConfiguration}
              className="w-full px-3 py-2 bg-petra-500 text-white rounded hover:bg-petra-600 transition-colors"
            >
              Validate Configuration
            </button>
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

  const createParamHandler = (key: string, isFloat: boolean = false) => {
    return (e: React.ChangeEvent<HTMLInputElement>) => {
      const value = isFloat ? parseFloat(e.target.value) || 0 : parseInt(e.target.value) || 0
      updateNodeData(nodeId, { 
        params: { ...params, [key]: value } 
      })
    }
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
            onChange={createParamHandler('preset_ms')}
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
            onChange={createParamHandler('increment')}
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
              onChange={createParamHandler('frequency', true)}
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
              onChange={createParamHandler('amplitude', true)}
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
