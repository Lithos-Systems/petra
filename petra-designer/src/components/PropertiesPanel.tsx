// petra-designer/src/components/PropertiesPanel.tsx
import { useFlowStore } from '@/store/flowStore'
import { PETRA_BLOCKS } from '@/nodes/BlockNode'
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

// Block parameter configurations
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
  AND: [
    {
      name: 'inputCount',
      label: 'Number of Inputs',
      type: 'select',
      options: [
        { value: '2', label: '2 inputs' },
        { value: '3', label: '3 inputs' },
        { value: '4', label: '4 inputs' },
        { value: '5', label: '5 inputs' },
        { value: '6', label: '6 inputs' },
        { value: '7', label: '7 inputs' },
        { value: '8', label: '8 inputs' },
      ],
      default: '2',
      hint: 'Number of inputs for AND gate'
    }
  ],
  OR: [
    {
      name: 'inputCount',
      label: 'Number of Inputs',
      type: 'select',
      options: [
        { value: '2', label: '2 inputs' },
        { value: '3', label: '3 inputs' },
        { value: '4', label: '4 inputs' },
        { value: '5', label: '5 inputs' },
        { value: '6', label: '6 inputs' },
        { value: '7', label: '7 inputs' },
        { value: '8', label: '8 inputs' },
      ],
      default: '2',
      hint: 'Number of inputs for OR gate'
    }
  ],
  ON_DELAY: [
    {
      name: 'preset',
      label: 'Preset Time',
      type: 'number',
      min: 0,
      max: 3600,
      step: 0.1,
      default: 1.0,
      hint: 'Delay time in seconds'
    }
  ],
  OFF_DELAY: [
    {
      name: 'preset',
      label: 'Preset Time',
      type: 'number',
      min: 0,
      max: 3600,
      step: 0.1,
      default: 1.0,
      hint: 'Delay time in seconds'
    }
  ],
  PULSE: [
    {
      name: 'preset',
      label: 'Pulse Duration',
      type: 'number',
      min: 0,
      max: 3600,
      step: 0.1,
      default: 1.0,
      hint: 'Pulse duration in seconds'
    }
  ],
  PID: [
    {
      name: 'kp',
      label: 'Proportional Gain (Kp)',
      type: 'number',
      min: 0,
      max: 100,
      step: 0.1,
      default: 1.0,
    },
    {
      name: 'ki',
      label: 'Integral Gain (Ki)',
      type: 'number',
      min: 0,
      max: 100,
      step: 0.01,
      default: 0.1,
    },
    {
      name: 'kd',
      label: 'Derivative Gain (Kd)',
      type: 'number',
      min: 0,
      max: 100,
      step: 0.01,
      default: 0.01,
    },
    {
      name: 'out_min',
      label: 'Output Min',
      type: 'number',
      min: -1000,
      max: 1000,
      step: 1,
      default: 0,
    },
    {
      name: 'out_max',
      label: 'Output Max',
      type: 'number',
      min: -1000,
      max: 1000,
      step: 1,
      default: 100,
    }
  ],
  SCALE: [
    {
      name: 'in_min',
      label: 'Input Min',
      type: 'number',
      default: 0,
    },
    {
      name: 'in_max',
      label: 'Input Max',
      type: 'number',
      default: 100,
    },
    {
      name: 'out_min',
      label: 'Output Min',
      type: 'number',
      default: 0,
    },
    {
      name: 'out_max',
      label: 'Output Max',
      type: 'number',
      default: 100,
    }
  ],
  LIMIT: [
    {
      name: 'min',
      label: 'Minimum Value',
      type: 'number',
      default: 0,
    },
    {
      name: 'max',
      label: 'Maximum Value',
      type: 'number',
      default: 100,
    }
  ],
  RAMP: [
    {
      name: 'rate_up',
      label: 'Rate Up (units/sec)',
      type: 'number',
      min: 0,
      step: 0.1,
      default: 1.0,
    },
    {
      name: 'rate_down',
      label: 'Rate Down (units/sec)',
      type: 'number',
      min: 0,
      step: 0.1,
      default: 1.0,
    }
  ],
  AVG: [
    {
      name: 'window_size',
      label: 'Window Size',
      type: 'number',
      min: 1,
      max: 1000,
      step: 1,
      default: 10,
      hint: 'Number of samples to average'
    }
  ],
  STDDEV: [
    {
      name: 'window_size',
      label: 'Window Size',
      type: 'number',
      min: 2,
      max: 1000,
      step: 1,
      default: 10,
      hint: 'Number of samples for standard deviation'
    }
  ]
}

// Component to render block-specific parameters
function renderBlockParams(blockData: BlockNodeData, updateNodeData: any, nodeId: string) {
  const params = BLOCK_PARAMETERS[blockData.blockType]
  if (!params) return null

  return (
    <div className="space-y-3 mt-4">
      <h4 className="text-xs font-medium text-black border-b border-[#606060] pb-1">
        Block Parameters
      </h4>
      {params.map(param => (
        <div key={param.name} className="space-y-1">
          <label className="block text-xs font-medium text-black">
            {param.label}
          </label>
          
          {param.type === 'select' ? (
            <select
              value={
                param.name === 'inputCount' 
                  ? String(blockData.inputCount || param.default)
                  : (blockData.params?.[param.name] || param.default)
              }
              onChange={(e) => {
                if (param.name === 'inputCount') {
                  updateNodeData(nodeId, { inputCount: parseInt(e.target.value) })
                } else {
                  updateNodeData(nodeId, {
                    params: {
                      ...blockData.params,
                      [param.name]: e.target.value
                    }
                  })
                }
              }}
              className="isa101-input w-full text-xs"
            >
              {param.options?.map(opt => (
                <option key={opt.value} value={opt.value}>{opt.label}</option>
              ))}
            </select>
          ) : param.type === 'number' ? (
            <input
              type="number"
              value={blockData.params?.[param.name] || param.default}
              onChange={(e) => {
                updateNodeData(nodeId, {
                  params: {
                    ...blockData.params,
                    [param.name]: parseFloat(e.target.value)
                  }
                })
              }}
              min={param.min}
              max={param.max}
              step={param.step}
              className="isa101-input w-full text-xs"
            />
          ) : param.type === 'boolean' ? (
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={blockData.params?.[param.name] || false}
                onChange={(e) => {
                  updateNodeData(nodeId, {
                    params: {
                      ...blockData.params,
                      [param.name]: e.target.checked
                    }
                  })
                }}
                className="w-3 h-3"
              />
              <span className="text-xs">Enable</span>
            </label>
          ) : (
            <input
              type="text"
              value={blockData.params?.[param.name] || param.default || ''}
              onChange={(e) => {
                updateNodeData(nodeId, {
                  params: {
                    ...blockData.params,
                    [param.name]: e.target.value
                  }
                })
              }}
              className="isa101-input w-full text-xs"
            />
          )}
          
          {param.hint && (
            <p className="text-xs text-gray-500">{param.hint}</p>
          )}
        </div>
      ))}
    </div>
  )
}

export default function PropertiesPanel() {
  const { selectedNode, updateNodeData, validateLogic } = useFlowStore()
  
  if (!selectedNode) return null
  
  const node = selectedNode
  
  // Validate node configuration
  const validation = validateNodeConfiguration(node)
  
  return (
    <div className="isa101-properties-panel w-80 h-full">
      <div className="isa101-panel-header">
        <span className="text-sm font-medium">Properties</span>
      </div>
      
      <div className="p-4 overflow-y-auto max-h-full">
        {/* Validation warnings */}
        {!validation.valid && (
          <div className="mb-4 p-2 bg-orange-100 border border-orange-300 rounded text-xs text-orange-800">
            {validation.error}
          </div>
        )}
        
        {/* Common properties */}
        <div className="space-y-3">
          <div>
            <label className="block text-xs font-medium text-black mb-1">
              Label
            </label>
            <input
              type="text"
              value={node.data.label || ''}
              onChange={(e) => updateNodeData(node.id, { label: e.target.value })}
              className="isa101-input w-full text-xs"
            />
          </div>
        </div>
        
        {/* Signal node properties */}
        {isSignalNode(node) && (
          <div className="space-y-3 mt-4">
            <h4 className="text-xs font-medium text-black border-b border-[#606060] pb-1">
              Signal Configuration
            </h4>
            <div>
              <label className="block text-xs font-medium text-black mb-1">
                Signal Name
              </label>
              <input
                type="text"
                value={(node.data as SignalNodeData).signalName || ''}
                onChange={(e) => updateNodeData(node.id, { signalName: e.target.value })}
                className="isa101-input w-full text-xs"
              >
                <option value="bool">Boolean</option>
                <option value="int">Integer</option>
                <option value="float">Float</option>
              </select>
            </div>
            <div>
              <label className="block text-xs font-medium text-black mb-1">
                Initial Value
              </label>
              {(node.data as SignalNodeData).signalType === 'bool' ? (
                <select
                  value={String((node.data as SignalNodeData).initial || false)}
                  onChange={(e) => updateNodeData(node.id, { initial: e.target.value === 'true' })}
                  className="isa101-input w-full text-xs"
                >
                  <option value="false">False</option>
                  <option value="true">True</option>
                </select>
              ) : (
                <input
                  type="number"
                  value={(node.data as SignalNodeData).initial || 0}
                  onChange={(e) => updateNodeData(node.id, { initial: parseFloat(e.target.value) })}
                  step={(node.data as SignalNodeData).signalType === 'float' ? '0.1' : '1'}
                  className="isa101-input w-full text-xs"
                />
              )}
            </div>
            <div>
              <label className="block text-xs font-medium text-black mb-1">
                Mode
              </label>
              <select
                value={(node.data as SignalNodeData).mode || 'write'}
                onChange={(e) => updateNodeData(node.id, { mode: e.target.value })}
                className="isa101-input w-full text-xs"
              >
                <option value="read">Read Only</option>
                <option value="write">Write Only</option>
              </select>
            </div>
          </div>
        )}
        
        {/* Block node properties */}
        {isBlockNode(node) && (
          <div className="space-y-3 mt-4">
            <h4 className="text-xs font-medium text-black border-b border-[#606060] pb-1">
              Block Type: {(node.data as BlockNodeData).blockType}
            </h4>
            {renderBlockParams(node.data as BlockNodeData, updateNodeData, node.id)}
          </div>
        )}
        
        {/* MQTT node properties */}
        {isMqttNode(node) && (
          <div className="space-y-3 mt-4">
            <h4 className="text-xs font-medium text-black border-b border-[#606060] pb-1">
              MQTT Configuration
            </h4>
            <div>
              <label className="block text-xs font-medium text-black mb-1">
                Broker Host
              </label>
              <input
                type="text"
                value={(node.data as MqttNodeData).brokerHost || ''}
                onChange={(e) => updateNodeData(node.id, { brokerHost: e.target.value })}
                placeholder="localhost"
                className="isa101-input w-full text-xs"
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-black mb-1">
                Broker Port
              </label>
              <input
                type="number"
                value={(node.data as MqttNodeData).brokerPort || 1883}
                onChange={(e) => updateNodeData(node.id, { brokerPort: parseInt(e.target.value) })}
                min={1}
                max={65535}
                className="isa101-input w-full text-xs"
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-black mb-1">
                Client ID
              </label>
              <input
                type="text"
                value={(node.data as MqttNodeData).clientId || ''}
                onChange={(e) => updateNodeData(node.id, { clientId: e.target.value })}
                placeholder="petra_client"
                className="isa101-input w-full text-xs"
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-black mb-1">
                Topic Prefix
              </label>
              <input
                type="text"
                value={(node.data as MqttNodeData).topicPrefix || ''}
                onChange={(e) => updateNodeData(node.id, { topicPrefix: e.target.value })}
                placeholder="petra/signals"
                className="isa101-input w-full text-xs"
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-black mb-1">
                Mode
              </label>
              <select
                value={(node.data as MqttNodeData).mode || 'read_write'}
                onChange={(e) => updateNodeData(node.id, { mode: e.target.value })}
                className="isa101-input w-full text-xs"
              >
                <option value="read">Subscribe Only</option>
                <option value="write">Publish Only</option>
                <option value="read_write">Both</option>
              </select>
            </div>
            {(node.data as MqttNodeData).mode !== 'write' && (
              <div>
                <label className="block text-xs font-medium text-black mb-1">
                  Subscribe Topics
                </label>
                <textarea
                  value={(node.data as MqttNodeData).subscriptions?.map(s => s.topic).join('\n') || ''}
                  onChange={(e) => {
                    const topics = e.target.value.split('\n').filter(t => t.trim())
                    updateNodeData(node.id, {
                      subscriptions: topics.map(topic => ({
                        topic,
                        signal: `mqtt_${node.id}_${topic.replace(/[^a-zA-Z0-9]/g, '_')}`,
                        qos: 1
                      }))
                    })
                  }}
                  rows={3}
                  placeholder="petra/+/temperature&#10;petra/+/status"
                  className="isa101-input w-full text-xs font-mono"
                />
                <p className="text-xs text-gray-500 mt-1">One topic per line. Use + and # for wildcards.</p>
              </div>
            )}
            {(node.data as MqttNodeData).mode !== 'read' && (
              <div>
                <label className="flex items-center gap-2 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={(node.data as MqttNodeData).publishOnChange || false}
                    onChange={(e) => updateNodeData(node.id, { publishOnChange: e.target.checked })}
                    className="w-3 h-3"
                  />
                  <span className="text-xs">Publish on value change</span>
                </label>
              </div>
            )}
            <div>
              <label className="block text-xs font-medium text-black mb-1">
                Username (optional)
              </label>
              <input
                type="text"
                value={(node.data as MqttNodeData).username || ''}
                onChange={(e) => updateNodeData(node.id, { username: e.target.value })}
                className="isa101-input w-full text-xs"
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-black mb-1">
                Password (optional)
              </label>
              <input
                type="password"
                value={(node.data as MqttNodeData).password || ''}
                onChange={(e) => updateNodeData(node.id, { password: e.target.value })}
                className="isa101-input w-full text-xs"
              />
            </div>
            <button
              onClick={() => {
                updateNodeData(node.id, { configured: true })
                toast.success('MQTT configuration saved')
              }}
              className="w-full mt-2 px-3 py-1 bg-green-600 text-white text-xs rounded hover:bg-green-700"
            >
              Save Configuration
            </button>
          </div>
        )}
        
        {/* S7 node properties */}
        {isS7Node(node) && (
          <div className="space-y-3 mt-4">
            <h4 className="text-xs font-medium text-black border-b border-[#606060] pb-1">
              S7 Configuration
            </h4>
            <div>
              <label className="block text-xs font-medium text-black mb-1">
                IP Address
              </label>
              <input
                type="text"
                value={(node.data as S7NodeData).ip || ''}
                onChange={(e) => updateNodeData(node.id, { ip: e.target.value })}
                placeholder="192.168.0.1"
                className="isa101-input w-full text-xs"
              />
            </div>
            <div className="grid grid-cols-2 gap-2">
              <div>
                <label className="block text-xs font-medium text-black mb-1">
                  Rack
                </label>
                <input
                  type="number"
                  value={(node.data as S7NodeData).rack || 0}
                  onChange={(e) => updateNodeData(node.id, { rack: parseInt(e.target.value) })}
                  min={0}
                  max={7}
                  className="isa101-input w-full text-xs"
                />
              </div>
              <div>
                <label className="block text-xs font-medium text-black mb-1">
                  Slot
                </label>
                <input
                  type="number"
                  value={(node.data as S7NodeData).slot || 2}
                  onChange={(e) => updateNodeData(node.id, { slot: parseInt(e.target.value) })}
                  min={0}
                  max={31}
                  className="isa101-input w-full text-xs"
                />
              </div>
            </div>
            <div>
              <label className="block text-xs font-medium text-black mb-1">
                Area
              </label>
              <select
                value={(node.data as S7NodeData).area || 'DB'}
                onChange={(e) => updateNodeData(node.id, { area: e.target.value })}
                className="isa101-input w-full text-xs"
              >
                <option value="DB">Data Block (DB)</option>
                <option value="I">Input (I)</option>
                <option value="Q">Output (Q)</option>
                <option value="M">Memory (M)</option>
              </select>
            </div>
            {(node.data as S7NodeData).area === 'DB' && (
              <div>
                <label className="block text-xs font-medium text-black mb-1">
                  DB Number
                </label>
                <input
                  type="number"
                  value={(node.data as S7NodeData).dbNumber || 1}
                  onChange={(e) => updateNodeData(node.id, { dbNumber: parseInt(e.target.value) })}
                  min={1}
                  className="isa101-input w-full text-xs"
                />
              </div>
            )}
            <div>
              <label className="block text-xs font-medium text-black mb-1">
                Address
              </label>
              <input
                type="number"
                value={(node.data as S7NodeData).address || 0}
                onChange={(e) => updateNodeData(node.id, { address: parseInt(e.target.value) })}
                min={0}
                className="isa101-input w-full text-xs"
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-black mb-1">
                Data Type
              </label>
              <select
                value={(node.data as S7NodeData).dataType || 'real'}
                onChange={(e) => updateNodeData(node.id, { dataType: e.target.value })}
                className="isa101-input w-full text-xs"
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
              <label className="block text-xs font-medium text-black mb-1">
                Direction
              </label>
              <select
                value={(node.data as S7NodeData).direction || 'read'}
                onChange={(e) => updateNodeData(node.id, { direction: e.target.value })}
                className="isa101-input w-full text-xs"
              >
                <option value="read">Read</option>
                <option value="write">Write</option>
                <option value="read_write">Read/Write</option>
              </select>
            </div>
          </div>
        )}
        
        {/* Modbus node properties */}
        {isModbusNode(node) && (
          <div className="space-y-3 mt-4">
            <h4 className="text-xs font-medium text-black border-b border-[#606060] pb-1">
              Modbus Configuration
            </h4>
            <div>
              <label className="block text-xs font-medium text-black mb-1">
                Host
              </label>
              <input
                type="text"
                value={(node.data as ModbusNodeData).host || ''}
                onChange={(e) => updateNodeData(node.id, { host: e.target.value })}
                placeholder="192.168.0.1"
                className="isa101-input w-full text-xs"
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-black mb-1">
                Port
              </label>
              <input
                type="number"
                value={(node.data as ModbusNodeData).port || 502}
                onChange={(e) => updateNodeData(node.id, { port: parseInt(e.target.value) })}
                min={1}
                max={65535}
                className="isa101-input w-full text-xs"
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-black mb-1">
                Unit ID
              </label>
              <input
                type="number"
                value={(node.data as ModbusNodeData).unitId || 1}
                onChange={(e) => updateNodeData(node.id, { unitId: parseInt(e.target.value) })}
                min={0}
                max={255}
                className="isa101-input w-full text-xs"
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-black mb-1">
                Register Type
              </label>
              <select
                value={(node.data as ModbusNodeData).dataType || 'holding_register'}
                onChange={(e) => updateNodeData(node.id, { dataType: e.target.value })}
                className="isa101-input w-full text-xs"
              >
                <option value="coil">Coil (0x)</option>
                <option value="discrete_input">Discrete Input (1x)</option>
                <option value="input_register">Input Register (3x)</option>
                <option value="holding_register">Holding Register (4x)</option>
              </select>
            </div>
            <div>
              <label className="block text-xs font-medium text-black mb-1">
                Address
              </label>
              <input
                type="number"
                value={(node.data as ModbusNodeData).address || 0}
                onChange={(e) => updateNodeData(node.id, { address: parseInt(e.target.value) })}
                min={0}
                max={65535}
                className="isa101-input w-full text-xs"
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-black mb-1">
                Direction
              </label>
              <select
                value={(node.data as ModbusNodeData).direction || 'read'}
                onChange={(e) => updateNodeData(node.id, { direction: e.target.value })}
                className="isa101-input w-full text-xs"
              >
                <option value="read">Read</option>
                <option value="write">Write</option>
                <option value="read_write">Read/Write</option>
              </select>
            </div>
          </div>
        )}
      </div>
      
      {/* Footer with actions */}
      <div className="isa101-panel-header border-t">
        <div className="flex justify-between items-center">
          <button
            onClick={() => {
              const result = validateLogic()
              if (result.valid) {
                toast.success('Configuration is valid')
              } else {
                toast.error(result.errors[0])
              }
            }}
            className="px-3 py-1 text-xs bg-blue-600 text-white rounded hover:bg-blue-700"
          >
            Validate
          </button>
          <span className="text-xs text-gray-600">
            ID: {node.id}
          </span>
        </div>
      </div>
    </div>
  )
}

export { BLOCK_PARAMETERS } w-full text-xs"
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-black mb-1">
                Type
              </label>
              <select
                value={(node.data as SignalNodeData).signalType || 'float'}
                onChange={(e) => updateNodeData(node.id, { signalType: e.target.value })}
                className="isa101-input w-full text-xs"
