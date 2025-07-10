import { useFlowStore } from '@/store/flowStore'
import { BLOCK_TYPES } from '@/utils/blockIcons'
import type { Node } from '@xyflow/react'
import type {
  SignalNodeData,
  BlockNodeData,
  TwilioNodeData,
  MqttNodeData,
  S7NodeData,
  ModbusNodeData,
} from '@/types/nodes'
import {
  isSignalNode,
  isBlockNode,
  isTwilioNode,
  isMqttNode,
  isS7Node,
  isModbusNode,
} from '@/types/nodes'
import { validateNodeConfiguration } from '@/utils/validation'
import toast from 'react-hot-toast'

const getStringValue = (value: any): string => {
  if (value === null || value === undefined) return ''
  if (typeof value === 'string') return value
  if (typeof value === 'number') return String(value)
  if (typeof value === 'boolean') return String(value)
  if (typeof value === 'object') return JSON.stringify(value)
  return ''
}

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

// Comprehensive block parameter configurations
const BLOCK_PARAMETERS: Record<string, Array<{
  name: string
  label: string
  type: 'number' | 'boolean' | 'select' | 'text'
  options?: Array<{ value: string; label: string }>
  min?: number
  max?: number
  step?: number
  default?: any
  hint?: string
}>> = {
  // Timer blocks
  ON_DELAY: [
    { name: 'duration', label: 'Duration (ms)', type: 'number', min: 0, default: 1000, hint: 'Time delay in milliseconds' }
  ],
  OFF_DELAY: [
    { name: 'duration', label: 'Duration (ms)', type: 'number', min: 0, default: 1000, hint: 'Time delay in milliseconds' }
  ],
  PULSE: [
    { name: 'duration', label: 'Pulse Width (ms)', type: 'number', min: 0, default: 500, hint: 'Pulse duration in milliseconds' }
  ],
  
  // Math blocks
  ADD: [],
  SUB: [],
  MUL: [],
  DIV: [
    { name: 'zero_handling', label: 'Division by Zero', type: 'select', options: [
      { value: 'error', label: 'Error' },
      { value: 'zero', label: 'Return 0' },
      { value: 'inf', label: 'Return Infinity' }
    ], default: 'zero' }
  ],
  
  // Control blocks
  PID: [
    { name: 'kp', label: 'Proportional Gain (Kp)', type: 'number', step: 0.01, default: 1.0, hint: 'Proportional control factor' },
    { name: 'ki', label: 'Integral Gain (Ki)', type: 'number', step: 0.01, default: 0.1, hint: 'Integral control factor' },
    { name: 'kd', label: 'Derivative Gain (Kd)', type: 'number', step: 0.01, default: 0.0, hint: 'Derivative control factor' },
    { name: 'setpoint', label: 'Setpoint', type: 'number', step: 0.1, default: 0.0 },
    { name: 'min_output', label: 'Min Output', type: 'number', default: -100 },
    { name: 'max_output', label: 'Max Output', type: 'number', default: 100 },
    { name: 'integral_limit', label: 'Integral Limit', type: 'number', default: 100, hint: 'Anti-windup limit' }
  ],
  
  // Scaling
  SCALE: [
    { name: 'in_min', label: 'Input Min', type: 'number', default: 0 },
    { name: 'in_max', label: 'Input Max', type: 'number', default: 100 },
    { name: 'out_min', label: 'Output Min', type: 'number', default: 0 },
    { name: 'out_max', label: 'Output Max', type: 'number', default: 1 },
    { name: 'clamp', label: 'Clamp Output', type: 'boolean', default: true, hint: 'Limit output to min/max range' }
  ],
  
  // Limit
  LIMIT: [
    { name: 'min', label: 'Minimum', type: 'number', default: 0 },
    { name: 'max', label: 'Maximum', type: 'number', default: 100 }
  ],
  
  // Selection
  SELECT: [
    { name: 'default_input', label: 'Default Input', type: 'select', options: [
      { value: 'a', label: 'Input A' },
      { value: 'b', label: 'Input B' }
    ], default: 'a' }
  ],
  
  MUX: [
    { name: 'inputs', label: 'Number of Inputs', type: 'number', min: 2, max: 16, default: 4 }
  ],
  
  DEMUX: [
    { name: 'outputs', label: 'Number of Outputs', type: 'number', min: 2, max: 16, default: 4 }
  ],
  
  // Comparison blocks
  GT: [
    { name: 'threshold', label: 'Threshold', type: 'number', step: 0.1, default: 0 }
  ],
  LT: [
    { name: 'threshold', label: 'Threshold', type: 'number', step: 0.1, default: 0 }
  ],
  GTE: [
    { name: 'threshold', label: 'Threshold', type: 'number', step: 0.1, default: 0 }
  ],
  LTE: [
    { name: 'threshold', label: 'Threshold', type: 'number', step: 0.1, default: 0 }
  ],
  EQ: [
    { name: 'threshold', label: 'Value', type: 'number', step: 0.1, default: 0 },
    { name: 'tolerance', label: 'Tolerance', type: 'number', step: 0.001, default: 0, hint: 'Comparison tolerance' }
  ],
  NEQ: [
    { name: 'threshold', label: 'Value', type: 'number', step: 0.1, default: 0 },
    { name: 'tolerance', label: 'Tolerance', type: 'number', step: 0.001, default: 0, hint: 'Comparison tolerance' }
  ],
  
  // Advanced blocks
  ALARM: [
    { name: 'high_high', label: 'High High Limit', type: 'number', step: 0.1, hint: 'Critical high alarm' },
    { name: 'high', label: 'High Limit', type: 'number', step: 0.1 },
    { name: 'low', label: 'Low Limit', type: 'number', step: 0.1 },
    { name: 'low_low', label: 'Low Low Limit', type: 'number', step: 0.1, hint: 'Critical low alarm' },
    { name: 'deadband', label: 'Deadband', type: 'number', step: 0.1, default: 0.5, hint: 'Hysteresis to prevent chattering' },
    { name: 'priority', label: 'Alarm Priority', type: 'select', options: [
      { value: '1', label: '1 - Critical' },
      { value: '2', label: '2 - High' },
      { value: '3', label: '3 - Medium' },
      { value: '4', label: '4 - Low' }
    ], default: '3' }
  ],
  
  EDGE: [
    { name: 'edge_type', label: 'Edge Type', type: 'select', options: [
      { value: 'rising', label: 'Rising Edge' },
      { value: 'falling', label: 'Falling Edge' },
      { value: 'both', label: 'Both Edges' }
    ], default: 'rising' }
  ],
  
  COUNTER: [
    { name: 'preset', label: 'Preset Value', type: 'number', default: 100 },
    { name: 'count_up', label: 'Count Direction', type: 'select', options: [
      { value: 'up', label: 'Count Up' },
      { value: 'down', label: 'Count Down' }
    ], default: 'up' },
    { name: 'reset_on_preset', label: 'Reset on Preset', type: 'boolean', default: true }
  ],
  
  AVERAGE: [
    { name: 'window_size', label: 'Window Size', type: 'number', min: 2, max: 1000, default: 10, hint: 'Number of samples to average' },
    { name: 'type', label: 'Average Type', type: 'select', options: [
      { value: 'simple', label: 'Simple Moving' },
      { value: 'exponential', label: 'Exponential' }
    ], default: 'simple' }
  ],
  
  RATELIMIT: [
    { name: 'rising_rate', label: 'Rising Rate Limit', type: 'number', step: 0.1, default: 10, hint: 'Max increase per second' },
    { name: 'falling_rate', label: 'Falling Rate Limit', type: 'number', step: 0.1, default: 10, hint: 'Max decrease per second' }
  ],
  
  DEADBAND: [
    { name: 'band', label: 'Deadband', type: 'number', step: 0.1, default: 1.0, hint: 'Change threshold' }
  ]
}

function renderBlockParams(
  blockData: BlockNodeData,
  updateNodeData: (nodeId: string, data: any) => void,
  nodeId: string
) {
  const blockType = blockData.blockType
  const parameters = BLOCK_PARAMETERS[blockType] || []
  
  if (parameters.length === 0) return null
  
  const currentParams = blockData.params || {}
  
  const updateParam = (paramName: string, newValue: any) => {
    updateNodeData(nodeId, {
      params: {
        ...currentParams,
        [paramName]: newValue
      }
    })
  }
  
  return (
    <div className="mt-4">
      <h5 className="text-xs font-medium text-[#404040] mb-2">Parameters</h5>
      {parameters.map(param => {
        const value = currentParams[param.name] ?? param.default
        
        switch (param.type) {
          case 'number':
            return (
              <div key={param.name} className="mb-3">
                <label className="block text-xs font-medium text-black mb-1">
                  {param.label}
                </label>
                <input
                  type="number"
                  value={value || 0}
                  onChange={(e) => updateParam(param.name, parseFloat(e.target.value) || 0)}
                  min={param.min}
                  max={param.max}
                  step={param.step || 1}
                  className="isa101-input w-full text-xs"
                />
                {param.hint && (
                  <p className="text-[10px] text-gray-600 mt-1">{param.hint}</p>
                )}
              </div>
            )
            
          case 'boolean':
            return (
              <div key={param.name} className="mb-3">
                <label className="flex items-center">
                  <input
                    type="checkbox"
                    checked={Boolean(value)}
                    onChange={(e) => updateParam(param.name, e.target.checked)}
                    className="mr-2"
                    style={{ accentColor: '#404040' }}
                  />
                  <span className="text-xs font-medium text-black">{param.label}</span>
                </label>
                {param.hint && (
                  <p className="text-[10px] text-gray-600 mt-1 ml-6">{param.hint}</p>
                )}
              </div>
            )
            
          case 'select':
            return (
              <div key={param.name} className="mb-3">
                <label className="block text-xs font-medium text-black mb-1">
                  {param.label}
                </label>
                <select
                  value={value || param.default}
                  onChange={(e) => updateParam(param.name, e.target.value)}
                  className="isa101-input w-full text-xs"
                >
                  {param.options?.map(opt => (
                    <option key={opt.value} value={opt.value}>
                      {opt.label}
                    </option>
                  ))}
                </select>
                {param.hint && (
                  <p className="text-[10px] text-gray-600 mt-1">{param.hint}</p>
                )}
              </div>
            )
            
          case 'text':
            return (
              <div key={param.name} className="mb-3">
                <label className="block text-xs font-medium text-black mb-1">
                  {param.label}
                </label>
                <input
                  type="text"
                  value={getStringValue(value)}
                  onChange={(e) => updateParam(param.name, e.target.value)}
                  className="isa101-input w-full text-xs"
                />
                {param.hint && (
                  <p className="text-[10px] text-gray-600 mt-1">{param.hint}</p>
                )}
              </div>
            )
        }
      })}
    </div>
  )
}

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
    <div className="mb-2">
      <label className="block text-xs font-medium text-black mb-1">
        {label}
      </label>
      <input
        type={type}
        value={getStringValue((node.data as any)[field])}
        onChange={createChangeHandler(field)}
        className="isa101-input w-full text-xs"
        placeholder={placeholder}
        {...(props || {})}
      />
    </div>
  )

  const renderSelect = (label: string, field: string, options: Array<{ value: string; label: string }>) => (
    <div className="mb-2">
      <label className="block text-xs font-medium text-black mb-1">
        {label}
      </label>
      <select
        value={getStringValue((node.data as any)[field]) || options[0]?.value}
        onChange={createChangeHandler(field)}
        className="isa101-input w-full text-xs"
      >
        {options.map(opt => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>
    </div>
  )

  const renderCheckbox = (label: string, field: string) => (
    <div className="mb-2">
      <label className="flex items-center">
        <input
          type="checkbox"
          checked={Boolean((node.data as any)[field])}
          onChange={createChangeHandler(field)}
          className="mr-2"
          style={{ accentColor: '#404040' }}
        />
        <span className="text-xs font-medium text-black">{label}</span>
      </label>
    </div>
  )

  return (
    <div className="isa101-sidebar w-80 h-full overflow-y-auto">
      <div className="isa101-panel-header">
        <span className="text-sm font-medium">
          {node.type === 'signal' && 'Signal Properties'}
          {node.type === 'block' && 'Block Properties'}
          {node.type === 'mqtt' && 'MQTT Properties'}
          {node.type === 's7' && 'S7 Properties'}
          {node.type === 'modbus' && 'Modbus Properties'}
          {node.type === 'twilio' && 'Twilio Properties'}
        </span>
      </div>
      
      <div className="p-4">
        {/* Common properties */}
        <div className="mb-4">
          <label className="block text-xs font-medium text-black mb-1">
            Name
          </label>
          <input
            type="text"
            value={getStringValue(node.data.label)}
            onChange={createChangeHandler('label')}
            className="isa101-input w-full text-xs"
            placeholder="Enter name"
          />
        </div>

        {/* Signal node */}
        {isSignalNode(node) && (
          <div className="space-y-4">
            <div>
              <h4 className="text-sm font-medium text-black mb-2 border-b border-[#606060] pb-1">
                Signal Configuration
              </h4>
              <div className="mb-3">
                <label className="block text-xs font-medium text-black mb-1">
                  Signal Type
                </label>
                <select
                  value={(node.data as SignalNodeData).signalType || 'float'}
                  onChange={createChangeHandler('signalType')}
                  className="isa101-input w-full text-xs"
                >
                  <option value="bool">Boolean</option>
                  <option value="int">Integer</option>
                  <option value="float">Float</option>
                </select>
              </div>
              
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
                  { value: 'M', label: 'M – Memory' }
                ])}
                
                {(node.data as S7NodeData).area === 'DB' && 
                  renderInput('DB Number', 'dbNumber', 'number', '1', { min: 1 })}
                
                {renderInput('Address', 'address', 'number', '0', { min: 0 })}
                
                {renderSelect('Data Type', 'dataType', [
                  { value: 'bool', label: 'Bool' },
                  { value: 'byte', label: 'Byte' },
                  { value: 'int', label: 'Int' },
                  { value: 'real', label: 'Real' }
                ])}
                
                {(node.data as S7NodeData).dataType === 'bool' && 
                  renderInput('Bit', 'bit', 'number', '0', { min: 0, max: 7 })}
                
                {renderSelect('Direction', 'direction', [
                  { value: 'read', label: 'Read' },
                  { value: 'write', label: 'Write' }
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

        {/* Modbus node */}
        {isModbusNode(node) && (
          <div className="space-y-4">
            <div>
              <h4 className="text-sm font-medium text-black mb-2 border-b border-[#606060] pb-1">
                Modbus Configuration
              </h4>
              
              {renderSelect('Protocol', 'protocol', [
                { value: 'tcp', label: 'Modbus TCP' },
                { value: 'rtu', label: 'Modbus RTU' }
              ])}
              
              {renderInput('Host/Port', 'host', 'text', '192.168.1.100')}
              {renderInput('Port', 'port', 'number', '502', { min: 1, max: 65535 })}
              {renderInput('Unit ID', 'unitId', 'number', '1', { min: 1, max: 255 })}
              
              <div className="mb-4">
                <h5 className="text-xs font-medium text-[#404040] mb-2">Register Mapping</h5>
                {renderInput('Signal Name', 'signal', 'text', 'sensor_value')}
                
                {renderSelect('Register Type', 'registerType', [
                  { value: 'coil', label: 'Coil (0x)' },
                  { value: 'discrete', label: 'Discrete Input (1x)' },
                  { value: 'input', label: 'Input Register (3x)' },
                  { value: 'holding', label: 'Holding Register (4x)' }
                ])}
                
                {renderInput('Register Address', 'address', 'number', '0', { min: 0, max: 65535 })}
                
                {renderSelect('Data Type', 'dataType', [
                  { value: 'bool', label: 'Boolean' },
                  { value: 'int16', label: 'Int16' },
                  { value: 'uint16', label: 'UInt16' },
                  { value: 'int32', label: 'Int32' },
                  { value: 'uint32', label: 'UInt32' },
                  { value: 'float', label: 'Float32' }
                ])}
                
                {renderSelect('Direction', 'direction', [
                  { value: 'read', label: 'Read' },
                  { value: 'write', label: 'Write' }
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
              Validate Modbus Configuration
            </button>
          </div>
        )}

        {/* Twilio node */}
        {isTwilioNode(node) && (
          <div className="space-y-4">
            <div>
              <h4 className="text-sm font-medium text-black mb-2 border-b border-[#606060] pb-1">
                Twilio Alarm Configuration
              </h4>
              
              <div className="mb-4">
                <h5 className="text-xs font-medium text-[#404040] mb-2">Account Settings</h5>
                {renderInput('Account SID', 'accountSid', 'text', 'Your Account SID')}
                {renderInput('Auth Token', 'authToken', 'password', 'Your Auth Token')}
                {renderInput('From Number', 'fromNumber', 'text', '+1234567890')}
              </div>

              <div className="mb-4">
                <h5 className="text-xs font-medium text-[#404040] mb-2">Alert Configuration</h5>
                {renderInput('To Number', 'toNumber', 'text', '+1234567890')}
                
                {renderSelect('Action Type', 'actionType', [
                  { value: 'sms', label: 'SMS' },
                  { value: 'call', label: 'Voice Call' },
                  { value: 'both', label: 'SMS + Call' }
                ])}
                
                <div className="mb-2">
                  <label className="block text-xs font-medium text-black mb-1">
                    Message Content
                  </label>
                  <textarea
                    value={getStringValue((node.data as TwilioNodeData).content)}
                    onChange={createChangeHandler('content')}
                    className="isa101-input w-full text-xs"
                    rows={3}
                    placeholder="Alert: {{signal_name}} value is {{value}}"
                  />
                </div>
                
                {renderInput('Cooldown (seconds)', 'cooldown', 'number', '300', { min: 0 })}
                {renderCheckbox('Require Acknowledgment', 'requireAck')}
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
