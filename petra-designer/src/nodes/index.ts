// src/nodes/index.ts (or wherever you assemble them)
import type { NodeTypes } from '@xyflow/react'
import type { PetraNode } from '@/types/nodes'

export const nodeTypes = {
  signal: SignalNode,
  block: BlockNode,
  twilio: TwilioNode,
  mqtt: MqttNode,
  s7: S7Node,
} as unknown as NodeTypes<PetraNode>
