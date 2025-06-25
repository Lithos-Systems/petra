import type { Node } from '@xyflow/react'

/* ---------- Base data interfaces ---------- */
export interface BaseNodeData {
  label: string
  [key: string]: any // Index signature for ReactFlow compatibility
}

export interface SignalNodeData extends BaseNodeData {
  signalType: 'bool' | 'int' | 'float'
  initial: boolean | number
}

export interface BlockNodeData extends BaseNodeData {
  blockType: string
  inputs: { name: string; type: string }[]
  outputs: { name: string; type: string }[]
  params?: Record<string, unknown>
}

export interface TwilioNodeData extends BaseNodeData {
  configured: boolean
  actionType: 'sms' | 'call'
  toNumber: string
  content: string
}

export interface MqttNodeData extends BaseNodeData {
  configured: boolean
  brokerHost: string
  brokerPort: number
  clientId: string
  topicPrefix: string
}

export interface S7NodeData extends BaseNodeData {
  configured: boolean
  area: 'DB' | 'I' | 'Q' | 'M'
  dbNumber: number
  address: number
  dataType: 'bool' | 'byte' | 'word' | 'int' | 'dint' | 'real'
  direction: 'read' | 'write' | 'read_write'
  signal: string
}

// Type guards for runtime type checking
export function isSignalNode(node: Node): node is Node<SignalNodeData> {
  return node.type === 'signal'
}

export function isBlockNode(node: Node): node is Node<BlockNodeData> {
  return node.type === 'block'
}

export function isTwilioNode(node: Node): node is Node<TwilioNodeData> {
  return node.type === 'twilio'
}

export function isMqttNode(node: Node): node is Node<MqttNodeData> {
  return node.type === 'mqtt'
}

export function isS7Node(node: Node): node is Node<S7NodeData> {
  return node.type === 's7'
}
