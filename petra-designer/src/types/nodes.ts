import type { Node } from '@xyflow/react'

/* ---------- base ---------- */
export interface BaseNodeData {
  label: string
}

/* ---------- signal ---------- */
export type SignalKind = 'bool' | 'int' | 'float'
export interface SignalNodeData extends BaseNodeData {
  signalType: SignalKind
  initial: boolean | number
}
export type SignalNode = Node<SignalNodeData>

/* ---------- block ---------- */
export type BlockType =
  | 'AND'
  | 'OR'
  | 'NOT'
  | 'GT'
  | 'LT'
  | 'EQ'
  | 'TON'
  | 'TOF'
  | 'R_TRIG'
  | 'F_TRIG'
  | 'SR_LATCH'
  | 'COUNTER'
  | 'MULTIPLY'
  | 'DIVIDE'
  | 'DATA_GENERATOR'

export interface BlockNodeData extends BaseNodeData {
  blockType: BlockType
  inputs: { name: string; type: string }[]
  outputs: { name: string; type: string }[]
  params?: Record<string, unknown>
}
export type BlockNode = Node<BlockNodeData>

/* ---------- mqtt ---------- */
export interface MqttNodeData extends BaseNodeData {
  configured: boolean
  brokerHost: string
  brokerPort: number
  clientId: string
  topicPrefix: string
}
export type MqttNode = Node<MqttNodeData>

/* ---------- s7 ---------- */
export type S7Area = 'DB' | 'I' | 'Q' | 'M'
export type S7Direction = 'read' | 'write' | 'read_write'
export interface S7NodeData extends BaseNodeData {
  configured: boolean
  area: S7Area
  dbNumber: number
  address: number
  dataType: 'bool' | 'byte' | 'word' | 'int' | 'dint' | 'real'
  direction: S7Direction
  signal: string
}
export type S7Node = Node<S7NodeData>

/* ---------- twilio ---------- */
export type TwilioAction = 'sms' | 'call'
export interface TwilioNodeData extends BaseNodeData {
  configured: boolean
  actionType: TwilioAction
  toNumber: string
  content: string
}
export type TwilioNode = Node<TwilioNodeData>

/* ---------- union ---------- */
export type PetraNode =
  | SignalNode
  | BlockNode
  | MqttNode
  | S7Node
  | TwilioNode
