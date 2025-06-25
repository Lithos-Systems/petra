// src/nodes/index.ts
import type { NodeTypes } from '@xyflow/react'
import type { PetraNode } from '@/types/nodes'

import SignalNode from './SignalNode'
import BlockNode from './BlockNode'
import TwilioNode from './TwilioNode'
import MqttNode from './MqttNode'
import S7Node from './S7Node'

export const nodeTypes = {
  signal: SignalNode,
  block: BlockNode,
  twilio: TwilioNode,
  mqtt: MqttNode,
  s7: S7Node,
} satisfies NodeTypes
