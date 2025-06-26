// src/utils/validation.ts
import { Connection, Node, Edge } from '@xyflow/react'
import type {
  SignalNodeData,
  BlockNodeData,
  TwilioNodeData,
  MqttNodeData,
  S7NodeData,
} from '@/types/nodes'

interface ValidationResult {
  valid: boolean
  error?: string
}

export function validateConnection(
  connection: Connection,
  nodes: Node[],
  edges: Edge[]
): ValidationResult {
  // Get source and target nodes
  const sourceNode = nodes.find(n => n.id === connection.source)
  const targetNode = nodes.find(n => n.id === connection.target)

  if (!sourceNode || !targetNode) {
    return { valid: false, error: 'Invalid connection' }
  }

  // Check if connection already exists
  const existingEdge = edges.find(
    e => e.source === connection.source && 
         e.target === connection.target &&
         e.sourceHandle === connection.sourceHandle &&
         e.targetHandle === connection.targetHandle
  )

  if (existingEdge) {
    return { valid: false, error: 'Connection already exists' }
  }

  // Type-specific validation
  if (sourceNode.type === 'signal' && targetNode.type === 'block') {
    // Signals can connect to block inputs
    return { valid: true }
  }

  if (sourceNode.type === 'block' && targetNode.type === 'signal') {
    // Blocks can output to signals
    return { valid: true }
  }

  if (sourceNode.type === 'block' && targetNode.type === 'block') {
    // Blocks can connect to other blocks
    return { valid: true }
  }

  if (targetNode.type === 'twilio') {
    // Only bool signals can trigger Twilio
    if (sourceNode.type === 'signal' && sourceNode.data.signalType !== 'bool') {
      return { valid: false, error: 'Twilio can only be triggered by bool signals' }
    }
    return { valid: true }
  }

  return { valid: true }
}

export function validateNodeConfiguration(node: Node): ValidationResult {
  switch (node.type) {
    case 'signal':
      return validateSignalNode(node.data as SignalNodeData)
    case 'block':
      return validateBlockNode(node.data as BlockNodeData)
    case 'twilio':
      return validateTwilioNode(node.data as TwilioNodeData)
    case 'mqtt':
      return validateMqttNode(node.data as MqttNodeData)
    case 's7':
      return validateS7Node(node.data as S7NodeData)
    default:
      return { valid: true }
  }
}

function validateSignalNode(data: SignalNodeData): ValidationResult {
  if (!data.label || data.label.trim() === '') {
    return { valid: false, error: 'Signal name is required' }
  }
  
  if (!['bool', 'int', 'float'].includes(data.signalType)) {
    return { valid: false, error: 'Invalid signal type' }
  }

  return { valid: true }
}

function validateBlockNode(data: BlockNodeData): ValidationResult {
  if (!data.label || data.label.trim() === '') {
    return { valid: false, error: 'Block name is required' }
  }

  // Validate block-specific parameters
  if (data.blockType === 'TON' || data.blockType === 'TOF') {
    const preset = data.params?.preset_ms as number
    if (!preset || preset < 0 || preset > 3600000) {
      return { valid: false, error: 'Timer preset must be between 0 and 3600000 ms' }
    }
  }

  if (data.blockType === 'DATA_GENERATOR') {
    const freq = data.params?.frequency as number
    const amp = data.params?.amplitude as number
    if (!freq || freq < 0 || freq > 100) {
      return { valid: false, error: 'Frequency must be between 0 and 100 Hz' }
    }
    if (!amp || amp < 0) {
      return { valid: false, error: 'Amplitude must be positive' }
    }
  }

  return { valid: true }
}

function validateTwilioNode(data: TwilioNodeData): ValidationResult {
  if (!data.toNumber || !data.toNumber.match(/^\+[1-9]\d{1,14}$/)) {
    return { valid: false, error: 'Valid E.164 phone number required (+1234567890)' }
  }

  if (!data.content || data.content.trim() === '') {
    return { valid: false, error: 'Message content is required' }
  }

  if (data.content.length > 1600) {
    return { valid: false, error: 'Message too long (max 1600 characters)' }
  }

  return { valid: true }
}

function validateMqttNode(data: MqttNodeData): ValidationResult {
  if (!data.brokerHost || data.brokerHost.trim() === '') {
    return { valid: false, error: 'Broker host is required' }
  }

  if (!data.brokerPort || data.brokerPort < 1 || data.brokerPort > 65535) {
    return { valid: false, error: 'Valid port number required (1-65535)' }
  }

  if (!data.clientId || data.clientId.trim() === '') {
    return { valid: false, error: 'Client ID is required' }
  }

  if (!data.topicPrefix || data.topicPrefix.trim() === '') {
    return { valid: false, error: 'Topic prefix is required' }
  }

  // Validate MQTT topic format
  if (data.topicPrefix.includes('#') || data.topicPrefix.includes('+')) {
    return { valid: false, error: 'Topic prefix cannot contain wildcards' }
  }

  return { valid: true }
}

function validateS7Node(data: S7NodeData): ValidationResult {
  if (!data.signal || data.signal.trim() === '') {
    return { valid: false, error: 'Signal name is required' }
  }

  // Validate IP address
  const ipRegex = /^(\d{1,3}\.){3}\d{1,3}$/
  if (!data.ip || !ipRegex.test(data.ip)) {
    return { valid: false, error: 'Valid IP address required' }
  }

  const ipParts = data.ip.split('.').map(Number)
  if (ipParts.some(part => part > 255)) {
    return { valid: false, error: 'Invalid IP address' }
  }

  if (data.rack === undefined || data.rack < 0 || data.rack > 7) {
    return { valid: false, error: 'Rack must be 0-7' }
  }

  if (data.slot === undefined || data.slot < 0 || data.slot > 31) {
    return { valid: false, error: 'Slot must be 0-31' }
  }

  if (data.area === 'DB' && (!data.dbNumber || data.dbNumber < 1)) {
    return { valid: false, error: 'Valid DB number required' }
  }

  if (data.address === undefined || data.address < 0) {
    return { valid: false, error: 'Valid address required' }
  }

  if (data.dataType === 'bool' && (data.bit === undefined || data.bit < 0 || data.bit > 7)) {
    return { valid: false, error: 'Bit must be 0-7' }
  }

  return { valid: true }
}
