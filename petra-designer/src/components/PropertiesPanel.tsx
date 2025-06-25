import { useFlowStore } from '@/store/flowStore'
import { BLOCK_TYPES } from '@/utils/blockIcons'

export default function PropertiesPanel() {
  const { selectedNode, updateNode } = useFlowStore()

  if (!selectedNode) return null

  const handleChange = (field: string, value: any) => {
    updateNode(selectedNode.id, { [field]: value })
  }

  // Group block types by category
  const blockTypesByCategory = BLOCK_TYPES.reduce((acc, type) => {
    if (!acc[type.category]) {
      acc[type.category] = []
    }
    acc[type.category].push(type)
    return acc
  }, {} as Record<string, typeof BLOCK_TYPES>)

  return (
    <div className="w-80 bg-white border-l border-gray-200 p-4 overflow-y-auto">
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
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
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
              {selectedNode.data.signalType === 'bool' ? (
                <label className="flex items-center">
                  <input
                    type="checkbox"
                    checked={selectedNode.data.initial || false}
                    onChange={(e) => handleChange('initial', e.target.checked)}
                    className="mr-2"
                  />
                  <span>{selectedNode.data.initial ? 'True' : 'False'}</span>
                </label>
              ) : (
                <input
                  type="number"
                  value={selectedNode.data.initial || 0}
                  onChange={(e) => handleChange('initial', Number(e.target.value))}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
                />
              )}
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
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
            >
              {Object.entries(blockTypesByCategory).map(([category, types]) => (
                <optgroup key={category} label={category}>
                  {types.map(type => (
                    <option key={type.value} value={type.value}>
                      {type.label}
                    </option>
                  ))}
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
                value={selectedNode.data.toNumber || ''}
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
                value={selectedNode.data.content || ''}
                onChange={(e) => handleChange('content', e.target.value)}
                rows={4}
                placeholder={selectedNode.data.actionType === 'sms' ? 'SMS message content' : 'TwiML or text for voice call'}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
              />
            </div>

            <div className="mt-2">
              <button
                onClick={() => {
                  const configured = selectedNode.data.toNumber && selectedNode.data.content
                  handleChange('configured', !!configured)
                }}
                className="px-3 py-1 bg-petra-500 text-white rounded hover:bg-petra-600"
              >
                Validate Configuration
              </button>
            </div>
          </>
        )}

        {selectedNode.type === 'mqtt' && (
          <>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Broker Host
              </label>
              <input
                type="text"
                value={selectedNode.data.brokerHost || ''}
                onChange={(e) => handleChange('brokerHost', e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Broker Port
              </label>
              <input
                type="number"
                value={selectedNode.data.brokerPort || 1883}
                onChange={(e) => handleChange('brokerPort', Number(e.target.value))}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Client ID
              </label>
              <input
                type="text"
                value={selectedNode.data.clientId || ''}
                onChange={(e) => handleChange('clientId', e.target.value)}
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
                value={selectedNode.data.topicPrefix || ''}
                onChange={(e) => handleChange('topicPrefix', e.target.value)}
                placeholder="petra/plc"
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
              />
            </div>
          </>
        )}

        {selectedNode.type === 's7' && (
          <>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Signal Name
              </label>
              <input
                type="text"
                value={selectedNode.data.signal || ''}
                onChange={(e) => handleChange('signal', e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Area
              </label>
              <select
                value={selectedNode.data.area}
                onChange={(e) => handleChange('area', e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
              >
                <option value="DB">DB - Data Block</option>
                <option value="I">I - Inputs</option>
                <option value="Q">Q - Outputs</option>
                <option value="M">M - Markers</option>
              </select>
            </div>

            {selectedNode.data.area === 'DB' && (
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  DB Number
                </label>
                <input
                  type="number"
                  value={selectedNode.data.dbNumber || 0}
                  onChange={(e) => handleChange('dbNumber', Number(e.target.value))}
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
                value={selectedNode.data.address || 0}
                onChange={(e) => handleChange('address', Number(e.target.value))}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Data Type
              </label>
              <select
                value={selectedNode.data.dataType}
                onChange={(e) => handleChange('dataType', e.target.value)}
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
                value={selectedNode.data.direction}
                onChange={(e) => handleChange('direction', e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-petra-500"
              >
                <option value="read">Read</option>
                <option value="write">Write</option>
                <option value="read_write">Read/Write</option>
              </select>
            </div>
          </>
        )}
      </div>
    </div>
  )
}
