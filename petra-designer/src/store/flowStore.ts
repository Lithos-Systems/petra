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
