import { Connection, Node, Edge } from '@xyflow/react'

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
