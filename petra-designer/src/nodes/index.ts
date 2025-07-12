// petra-designer/src/nodes/index.ts
import type { NodeTypes } from '@xyflow/react'
import SignalNode from './SignalNode'
import BlockNode from './BlockNode'
import TwilioNode from './TwilioNode'
import MqttNode from './MqttNode'
import S7Node from './S7Node'
import ModbusNode from './ModbusNode'

export const nodeTypes: NodeTypes = {
  signal: SignalNode,
  block: BlockNode,
  logicBlock: BlockNode, // Alias for compatibility
  twilio: TwilioNode,
  mqtt: MqttNode,
  s7: S7Node,
  modbus: ModbusNode,
  protocol: MqttNode, // Use MQTT node as default protocol node
}
