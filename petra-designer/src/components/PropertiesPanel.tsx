import { useFlowStore } from '@/store/flowStore'
import { BLOCK_TYPES } from '@/utils/blockIcons'

export default function PropertiesPanel() {
  const { selectedNode, updateNode } = useFlowStore()

  if (!selectedNode) return null

  const handleChange = (field: string, value: any) => {
    updateNode(selectedNode.id, { [field]: value })
  }

  return (
    <div className="w-80 bg-white border-l border-gray-200 p-4">
      <h3 className="text-lg font-semibold mb-4">Properties</h3>
      
      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Label
          </label>
          <input
            type="text"
            value={selectedNode.data.label || ''}
            onChange={(e) => handleChange('label', e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-md"
          />
        </div>

        {selectedNode.type === 'signal' && (
          <>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Signal Type
              </label>
              <select
                value={selectedNode.data.signalType}
                onChange={(e) => handleChange('signalType', e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-md"
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
              <input
                type={selectedNode.data.signalType === 'bool' ? 'checkbox' : 'number'}
                checked={selectedNode.data.signalType === 'bool' ? selectedNode.data.initial : undefined}
                value={selectedNode.data.signalType !== 'bool' ? selectedNode.data.initial : undefined}
                onChange={(e) => handleChange('initial', 
                  selectedNode.data.signalType === 'bool' ? e.target.checked : Number(e.target.value)
                )}
                className="w-full px-3 py-2 border border-gray-300 rounded-md"
              />
            </div>
          </>
        )}

        {selectedNode.type === 'block' && (
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Block Type
            </label>
            <select
              value={selectedNode.data.blockType}
              onChange={(e) => handleChange('blockType', e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md"
            >
              {BLOCK_TYPES.map(type => (
                <optgroup key={type.category} label={type.category}>
                  <option value={type.value}>{type.label}</option>
                </optgroup>
              ))}
            </select>
          </div>
        )}

        {selectedNode.type === 'twilio' && (
          <>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Action Type
              </label>
              <select
                value={selectedNode.data.actionType}
                onChange={(e) => handleChange('actionType', e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-md"
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
                value={selectedNode.data.toNumber || ''}
                onChange={(e) => handleChange('toNumber', e.target.value)}
                placeholder="+1234567890"
                className="w-full px-3 py-2 border border-gray-300 rounded-md"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Content
              </label>
              <textarea
                value={selectedNode.data.content || ''}
                onChange={(e) => handleChange('content', e.target.value)}
                rows={4}
                className="w-full px-3 py-2 border border-gray-300 rounded-md"
              />
            </div>
          </>
        )}
      </div>
    </div>
  )
}
