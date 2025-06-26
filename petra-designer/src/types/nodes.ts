// src/types/nodes.ts
import type { Node } from '@xyflow/react'

/* ---------- Base data interfaces ---------- */
export interface BaseNodeData {
  label: string
  [key: string]: any // Index signature for ReactFlow compatibility
}

export interface SignalNodeData extends BaseNodeData {
  signalType: 'bool' | 'int' | 'float'
  initial: boolean | number
  mode: 'read' | 'write'
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
  username?: string
  password?: string
  mode: 'read' | 'write' | 'read_write'
  publishOnChange: boolean
  signalName?: string  // The specific signal to read/write
  signalType?: 'bool' | 'int' | 'float'  // Type of the signal
}

export interface S7NodeData extends BaseNodeData {
  configured: boolean
  ip: string
  rack: number
  slot: number
  area: 'DB' | 'I' | 'Q' | 'M'
  dbNumber: number
  address: number
  dataType: 'bool' | 'byte' | 'word' | 'int' | 'dint' | 'real'
  bit?: number
  direction: 'read' | 'write' | 'read_write'
  signal: string
}
export interface ContactNodeData extends BaseNodeData {
  configured: boolean
  name: string
  email?: string
  phone?: string
  preferredMethod: 'email' | 'sms' | 'call'
  priority: number  // 1 = highest priority
  escalationDelay: number  // seconds before escalating to next contact
  workHoursOnly: boolean
  timezone: string
}

export interface AlarmNodeData extends BaseNodeData {
  configured: boolean
  condition: 'above' | 'below' | 'equals' | 'not_equals' | 'deadband'
  severity: 'info' | 'warning' | 'critical' | 'emergency'
  delaySeconds: number  // Time condition must be true before alarming
  repeatInterval: number  // Seconds between repeated notifications
  requireAcknowledgment: boolean
  autoReset: boolean
  message: string
}

export interface EmailNodeData extends BaseNodeData {
  configured: boolean
  toEmail: string
  ccEmails?: string[]
  subject: string
  body: string
  provider: 'smtp' | 'sendgrid' | 'ses'
  smtpHost?: string
  smtpPort?: number
  smtpUser?: string
  smtpPass?: string
}

// Type alias for backwards compatibility
export type PetraNode = Node

// Type guards for runtime type checking (exported as values, not types)
export function isSignalNode(node: Node): node is Node & { data: SignalNodeData } {
  return node.type === 'signal'
}

export function isBlockNode(node: Node): node is Node & { data: BlockNodeData } {
  return node.type === 'block'
}

export function isTwilioNode(node: Node): node is Node & { data: TwilioNodeData } {
  return node.type === 'twilio'
}

export function isMqttNode(node: Node): node is Node & { data: MqttNodeData } {
  return node.type === 'mqtt'
}

export function isS7Node(node: Node): node is Node & { data: S7NodeData } {
  return node.type === 's7'
}
