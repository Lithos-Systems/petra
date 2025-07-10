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

  if (!selectedNode) {
    return (
      <div className="isa101-sidebar w-80 h-full">
        <div className="isa101-panel-header">
          <span className="text-sm font-medium">Properties</span>
        </div>
        <div className="p-4 text-center text-[#606060]">
          <p className="text-sm">Select a block to edit properties</p>
        </div>
      </div>
    )
  }
  
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

  const renderInput = (label: string, field: string, type: string = 'text', placeholder?: string, props?: any) => (
    <div className="mb-3">
      <label className="block text-xs font-medium text-black mb-1">
        {label}
      </label>
      <input
        type={type}
        value={node.data[field] || ''}
        onChange={createChangeHandler(field)}
        placeholder={placeholder}
        className="isa101-input w-full text-xs"
        {...props}
      />
    </div>
  )

  const renderSelect = (label: string, field: string, options: Array<{ value: string; label: string }>) => (
    <div className="mb-3">
      <label className="block text-xs font-medium text-black mb-1">
        {label}
      </label>
      <select
        value={node.data[field] || ''}
        onChange={createChangeHandler(field)}
        className="isa101-input w-full text-xs"
      >
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </div>
  )

  const renderTextArea = (label: string, field: string, placeholder?: string, rows: number = 3) => (
    <div className="mb-3">
      <label className="block text-xs font-medium text-black mb-1">
        {label}
      </label>
      <textarea
        value={node.data[field] || ''}
        onChange={createChangeHandler(field)}
        placeholder={placeholder}
        rows={rows}
        className="isa101-input w-full text-xs resize-none"
      />
    </div>
  )

  const renderCheckbox = (label: string, field: string) => (
    <div className="mb-3 flex items-center">
      <input
        type="checkbox"
        checked={Boolean(node.data[field])}
        onChange={createChangeHandler(field)}
        className="mr-2"
        style={{ accentColor: '#404040' }}
      />
      <label className="text-xs font-medium text-black">
        {label}
      </label>
    </div>
  )

  return (
    <div className="isa101-sidebar w-80 h-full">
      <div className="isa101-panel-header">
        <span className="text-sm font-medium">Properties</span>
      </div>

      <div className="flex-1 overflow-y-auto p-4">
        {/* Common Properties */}
        <div className="mb-4">
          <h4 className="text-sm font-medium text-black mb-2 border-b border-[#606060] pb-1">
            General
          </h4>
          {renderInput('Name', 'label', 'text', 'Block name')}
        </div>

        {/* Signal node */}
        {isSignalNode(node) && (
          <div className="space-y-4">
            <div>
              <h4 className="text-sm font-medium text-black mb-2 border-b border-[#606060] pb-1">
                Signal Configuration
              </h4>
              {renderSelect('Signal Type', 'signalType', [
                { value: 'bool', label: 'Boolean' },
                { value: 'int', label: 'Integer' },
                { value: 'float', label: 'Float' }
              ])}

              {renderSelect('Mode', 'mode', [
                { value: 'read', label: 'Read Only' },
                { value: 'write', label: 'Write Only' }
              ])}

              <div className="mb-3">
                <label className="block text-xs font-medium text-black mb-1">
                  Initial Value
                </label>
                {(node.data as SignalNodeData).signalType === 'bool' ? (
                  <label className="flex items-center">
                    <input
                      type="checkbox"
                      checked={Boolean((node.data as SignalNodeData).initial)}
                      onChange={(e) => updateNodeData(node.id, { initial: e.target.checked })}
                      className="mr-2"
                      style={{ accentColor: '#404040' }}
                    />
                    <span className="text-xs">{Boolean((node.data as SignalNodeData).initial) ? 'True' : 'False'}</span>
                  </label>
                ) : (
                  <input
                    type="number"
                    value={Number((node.data as SignalNodeData).initial) || 0}
                    onChange={createNumberHandler('initial')}
                    step={(node.data as SignalNodeData).signalType === 'float' ? '0.1' : '1'}
                    className="isa101-input w-full text-xs"
                  />
                )}
              </div>
            </div>
          </div>
        )}

        {/* Block node */}
        {isBlockNode(node) && (
          <div className="space-y-4">
            <div>
              <h4 className="text-sm font-medium text-black mb-2 border-b border-[#606060] pb-1">
                Block Configuration
              </h4>
              <div className="mb-3">
                <label className="block text-xs font-medium text-black mb-1">
                  Block Type
                </label>
                <select
                  value={(node.data as BlockNodeData).blockType || 'AND'}
                  onChange={createChangeHandler('blockType')}
                  className="isa101-input w-full text-xs"
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
            </div>
          </div>
        )}

        {/* MQTT node */}
        {isMqttNode(node) && (
          <div className="space-y-4">
            <div>
              <h4 className="text-sm font-medium text-black mb-2 border-b border-[#606060] pb-1">
                MQTT Configuration
              </h4>
              {renderInput('Broker Host', 'brokerHost', 'text', 'localhost')}
              {renderInput('Broker Port', 'brokerPort', 'number', '1883', { min: 1, max: 65535 })}
              {renderInput('Client ID', 'clientId', 'text', 'petra_client')}
              {renderInput('Topic Prefix', 'topicPrefix', 'text', 'petra')}
              {renderInput('Signal Name', 'signalName', 'text', 'sensor_data')}
              
              {renderSelect('Signal Type', 'signalType', [
                { value: 'bool', label: 'Boolean' },
                { value: 'int', label: 'Integer' },
                { value: 'float', label: 'Float' }
              ])}

              {renderSelect('Mode', 'mode', [
                { value: 'read', label: 'Read (Subscribe)' },
                { value: 'write', label: 'Write (Publish)' },
                { value: 'read_write', label: 'Read/Write' }
              ])}

              {renderInput('Username', 'username', 'text', 'Optional')}
              {renderInput('Password', 'password', 'password', 'Optional')}
              
              {renderCheckbox('Publish on Change', 'publishOnChange')}
            </div>

            <button
              onClick={validateConfiguration}
              className="isa101-button w-full text-xs py-2"
              style={{ 
                backgroundColor: '#00C800', 
                color: 'white',
                borderColor: '#008000'
              }}
            >
              Validate MQTT Configuration
            </button>
          </div>
        )}

        {/* S7 node */}
        {isS7Node(node) && (
          <div className="space-y-4">
            <div>
              <h4 className="text-sm font-medium text-black mb-2 border-b border-[#606060] pb-1">
                Siemens S7 Configuration
              </h4>
              
              <div className="mb-4">
                <h5 className="text-xs font-medium text-[#404040] mb-2">Connection Settings</h5>
                {renderInput('IP Address', 'ip', 'text', '192.168.1.100')}
                <div className="grid grid-cols-2 gap-2">
                  {renderInput('Rack', 'rack', 'number', '0', { min: 0, max: 7 })}
                  {renderInput('Slot', 'slot', 'number', '1', { min: 0, max: 31 })}
                </div>
              </div>

              <div className="mb-4">
                <h5 className="text-xs font-medium text-[#404040] mb-2">Data Mapping</h5>
                {renderInput('Signal Name', 'signal', 'text', 'motor_running')}
                
                {renderSelect('Memory Area', 'area', [
                  { value: 'DB', label: 'DB – Data Block' },
                  { value: 'I', label: 'I – Inputs' },
                  { value: 'Q', label: 'Q – Outputs' },
                  { value: 'M', label: 'M – Markers' }
                ])}

                {(node.data as S7NodeData).area === 'DB' && (
                  renderInput('DB Number', 'dbNumber', 'number', '1', { min: 1, max: 65535 })
                )}

                {renderInput('Address', 'address', 'number', '0', { min: 0, max: 65535 })}

                {renderSelect('Data Type', 'dataType', [
                  { value: 'bool', label: 'Bool' },
                  { value: 'byte', label: 'Byte' },
                  { value: 'word', label: 'Word' },
                  { value: 'int', label: 'Int' },
                  { value: 'dint', label: 'DInt' },
                  { value: 'real', label: 'Real' }
                ])}

                {(node.data as S7NodeData).dataType === 'bool' && (
                  renderInput('Bit', 'bit', 'number', '0', { min: 0, max: 7 })
                )}

                {renderSelect('Direction', 'direction', [
                  { value: 'read', label: 'Read from PLC' },
                  { value: 'write', label: 'Write to PLC' },
                  { value: 'read_write', label: 'Read/Write' }
                ])}
              </div>
            </div>

            <button
              onClick={validateConfiguration}
              className="isa101-button w-full text-xs py-2"
              style={{ 
                backgroundColor: '#00C800', 
                color: 'white',
                borderColor: '#008000'
              }}
            >
              Validate S7 Configuration
            </button>
          </div>
        )}

        {/* Twilio node */}
        {isTwilioNode(node) && (
          <div className="space-y-4">
            <div>
              <h4 className="text-sm font-medium text-black mb-2 border-b border-[#606060] pb-1">
                Twilio Configuration
              </h4>
              
              {renderSelect('Action Type', 'actionType', [
                { value: 'sms', label: 'SMS Message' },
                { value: 'call', label: 'Voice Call' }
              ])}

              {renderInput('To Phone Number', 'toNumber', 'tel', '+1234567890')}
              
              {renderTextArea('Message Content', 'content', 
                (node.data as TwilioNodeData).actionType === 'call' 
                  ? 'TwiML for voice call' 
                  : 'SMS message text', 4)}

              <div className="text-xs text-[#606060] p-2 border border-[#606060] bg-[#F0F0F0]">
                <strong>Note:</strong> Phone numbers must be in E.164 format (e.g., +1234567890).
                Account SID and Auth Token are configured in environment variables.
              </div>
            </div>

            <button
              onClick={validateConfiguration}
              className="isa101-button w-full text-xs py-2"
              style={{ 
                backgroundColor: '#00C800', 
                color: 'white',
                borderColor: '#008000'
              }}
            >
              Validate Twilio Configuration
            </button>
          </div>
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

  const createParamSelectHandler = (key: string) => {
    return (e: React.ChangeEvent<HTMLSelectElement>) => {
      updateNodeData(nodeId, {
        params: { ...params, [key]: e.target.value }
      })
    }
  }

  switch (blockType) {
    case 'timer':
    case 'delay':
      return (
        <div className="space-y-3">
          <h5 className="text-xs font-medium text-[#404040] mb-2">Timer Parameters</h5>
          <div>
            <label className="block text-xs font-medium text-black mb-1">Preset Time</label>
            <input
              type="number"
              value={getParamNumber('preset', 1.0)}
              onChange={createParamHandler('preset', true)}
              step="0.1"
              min="0"
              className="isa101-input w-full text-xs"
            />
          </div>
          <div>
            <label className="block text-xs font-medium text-black mb-1">Time Units</label>
            <select
              value={params.units || 'seconds'}
              onChange={createParamSelectHandler('units')}
              className="isa101-input w-full text-xs"
            >
              <option value="seconds">Seconds</option>
              <option value="minutes">Minutes</option>
              <option value="hours">Hours</option>
            </select>
          </div>
        </div>
      )

    case 'pid':
      return (
        <div className="space-y-3">
          <h5 className="text-xs font-medium text-[#404040] mb-2">PID Parameters</h5>
          <div>
            <label className="block text-xs font-medium text-black mb-1">Proportional Gain (Kp)</label>
            <input
              type="number"
              value={getParamNumber('kp', 1.0)}
              onChange={createParamHandler('kp', true)}
              step="0.01"
              className="isa101-input w-full text-xs"
            />
          </div>
          <div>
            <label className="block text-xs font-medium text-black mb-1">Integral Gain (Ki)</label>
            <input
              type="number"
              value={getParamNumber('ki', 0.1)}
              onChange={createParamHandler('ki', true)}
              step="0.01"
              className="isa101-input w-full text-xs"
            />
          </div>
          <div>
            <label className="block text-xs font-medium text-black mb-1">Derivative Gain (Kd)</label>
            <input
              type="number"
              value={getParamNumber('kd', 0.01)}
              onChange={createParamHandler('kd', true)}
              step="0.001"
              className="isa101-input w-full text-xs"
            />
          </div>
          <div className="grid grid-cols-2 gap-2">
            <div>
              <label className="block text-xs font-medium text-black mb-1">Output Min</label>
              <input
                type="number"
                value={getParamNumber('output_min', 0.0)}
                onChange={createParamHandler('output_min', true)}
                className="isa101-input w-full text-xs"
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-black mb-1">Output Max</label>
              <input
                type="number"
                value={getParamNumber('output_max', 100.0)}
                onChange={createParamHandler('output_max', true)}
                className="isa101-input w-full text-xs"
              />
            </div>
          </div>
        </div>
      )

    case 'greater_than':
    case 'less_than':
    case 'equal':
      return (
        <div className="space-y-3">
          <h5 className="text-xs font-medium text-[#404040] mb-2">Comparison Parameters</h5>
          <div>
            <label className="block text-xs font-medium text-black mb-1">Threshold</label>
            <input
              type="number"
              value={getParamNumber('threshold', 0.0)}
              onChange={createParamHandler('threshold', true)}
              step="0.1"
              className="isa101-input w-full text-xs"
            />
          </div>
          <div>
            <label className="block text-xs font-medium text-black mb-1">Deadband</label>
            <input
              type="number"
              value={getParamNumber('deadband', 0.1)}
              onChange={createParamHandler('deadband', true)}
              step="0.01"
              min="0"
              className="isa101-input w-full text-xs"
            />
          </div>
        </div>
      )

    default:
      return null
  }
}
