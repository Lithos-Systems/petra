function getDefaultNodeData(type: string): any {
  switch (type) {
    case 'signal':
      return {
        label: 'New Signal',
        signalType: 'float',
        initial: 0,
      }
    case 'block':
      return {
        label: 'New Block',
        blockType: 'AND',
        // Define default inputs/outputs based on block type
        inputs: [
          { name: 'in1', type: 'bool' },
          { name: 'in2', type: 'bool' }
        ],
        outputs: [
          { name: 'out', type: 'bool' }
        ],
        params: {},
      }
    case 'twilio':
      return {
        label: 'Twilio Action',
        actionType: 'sms',
        toNumber: '',
        content: '',
        configured: false,
      }
    case 'mqtt':
      return {
        label: 'MQTT Config',
        brokerHost: 'mqtt.lithos.systems',
        brokerPort: 1883,
        clientId: 'petra-01',
        topicPrefix: 'petra/plc',
        configured: false,
      }
    case 's7':
      return {
        label: 'S7 Mapping',
        signal: '',
        area: 'DB',
        dbNumber: 0,
        address: 0,
        dataType: 'bool',
        direction: 'read',
        configured: false,
      }
    default:
      return { label: 'New Node' }
  }
}
// Add this helper function
function getBlockInputsOutputs(blockType: string) {
  switch (blockType) {
    case 'AND':
    case 'OR':
      return {
        inputs: [
          { name: 'in1', type: 'bool' },
          { name: 'in2', type: 'bool' }
        ],
        outputs: [{ name: 'out', type: 'bool' }]
      }
    case 'NOT':
      return {
        inputs: [{ name: 'in', type: 'bool' }],
        outputs: [{ name: 'out', type: 'bool' }]
      }
    case 'GT':
    case 'LT':
    case 'EQ':
      return {
        inputs: [
          { name: 'in1', type: 'float' },
          { name: 'in2', type: 'float' }
        ],
        outputs: [{ name: 'out', type: 'bool' }]
      }
    case 'TON':
    case 'TOF':
      return {
        inputs: [{ name: 'in', type: 'bool' }],
        outputs: [{ name: 'q', type: 'bool' }]
      }
    case 'R_TRIG':
    case 'F_TRIG':
      return {
        inputs: [{ name: 'clk', type: 'bool' }],
        outputs: [{ name: 'q', type: 'bool' }]
      }
    case 'SR_LATCH':
      return {
        inputs: [
          { name: 'set', type: 'bool' },
          { name: 'reset', type: 'bool' }
        ],
        outputs: [{ name: 'q', type: 'bool' }]
      }
    case 'COUNTER':
      return {
        inputs: [{ name: 'enable', type: 'bool' }],
        outputs: [{ name: 'count', type: 'int' }]
      }
    case 'MULTIPLY':
      return {
        inputs: [
          { name: 'in1', type: 'float' },
          { name: 'in2', type: 'float' }
        ],
        outputs: [{ name: 'out', type: 'float' }]
      }
    case 'DATA_GENERATOR':
      return {
        inputs: [{ name: 'enable', type: 'bool' }],
        outputs: [
          { name: 'sine_out', type: 'float' },
          { name: 'count_out', type: 'int' }
        ]
      }
    default:
      return {
        inputs: [{ name: 'in', type: 'float' }],
        outputs: [{ name: 'out', type: 'float' }]
      }
  }
}

// Update the updateNode function to handle blockType changes
updateNode: (nodeId: string, data: any) => {
  set({
    nodes: get().nodes.map((node) => {
      if (node.id === nodeId) {
        // If blockType changed, update inputs/outputs
        if (data.blockType && data.blockType !== node.data.blockType) {
          const { inputs, outputs } = getBlockInputsOutputs(data.blockType)
          return {
            ...node,
            data: { ...node.data, ...data, inputs, outputs }
          }
        }
        return { ...node, data: { ...node.data, ...data } }
      }
      return node
    }),
  })
},
