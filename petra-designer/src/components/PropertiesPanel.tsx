import { useFlowStore } from '@/store/flowStore'
import { BLOCK_TYPES } from '@/utils/blockIcons'
import type { Node } from '@xyflow/react'
import type {
  SignalNodeData,
  BlockNodeData,
  TwilioNodeData,
  MqttNodeData,
  S7NodeData,
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
  const { selectedNode, updateNode } = useFlowStore()

  if (!selectedNode) return null
  
  const node = selectedNode as Node

  function handleChange(field: string, value: any) {
    updateNode(node.id, { [field]: value })
  }

  const labelInput = (
    <div>
      <label className="block text-sm font-medium text-gray-700 mb-1">Label</label>
      <input
        type="text"
        value={String(node.data.label || '')}
        onChange={(e) => handleChange('label', e.target.value)}
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
                value={String((node.data as SignalNodeData).signalType || 'float')}
                onChange={(e) => handleChange('signalType', e.target.value)}
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
                    onChange={(e) => handleChange('initial', e.target.checked)}
                    className="mr-2"
                  />
                  <span>{(node.data as SignalNodeData).initial ? 'True' : 'False'}</span>
                </label>
              ) : (
                <input
                  type="number"
                  value={String((node.data as SignalNodeData).initial ?? 0)}
                  onChange={(e) => handleChange('initial', Number(e.target.value))}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
                />
              )}
            </div>
          </>
        )}

        {/* Block node */}
        {isBlockNode(node) && (
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Block Type
            </label>
            <select
              value={String((node.data as BlockNodeData).blockType || 'AND')}
              onChange={(e) => handleChange('blockType', e.target.value)}
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
        )}

        {/* Twilio node */}
        {isTwilioNode(node) && (
          <>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Action Type
              </label>
              <select
                value={String((node.data as TwilioNodeData).actionType || 'sms')}
                onChange={(e) => handleChange('actionType', e.target.value)}
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
                value={String((node.data as TwilioNodeData).toNumber || '')}
                onChange={(e) => handleChange('toNumber', e.target.value)}
                placeholder="+1234567890"
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Content
              </label>
              <textarea
                value={String((node.data as TwilioNodeData).content || '')}
                onChange={(e) => handleChange('content', e.target.value)}
                rows={4}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
              />
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
                value={String((node.data as MqttNodeData).brokerHost || '')}
                onChange={(e) => handleChange('brokerHost', e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
              />
            </div>
            {/* Add other MQTT fields similarly */}
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
                value={String((node.data as S7NodeData).signal || '')}
                onChange={(e) => handleChange('signal', e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
              />
            </div>
            {/* Add other S7 fields similarly */}
          </>
        )}
      </div>
    </div>
  )
}
