// petra-designer/src/utils/yamlGenerator.ts
import { Edge, Node } from '@xyflow/react'
import * as yaml from 'yaml'
import type {
  SignalNodeData,
  BlockNodeData,
  TwilioNodeData,
  MqttNodeData,
  S7NodeData,
  ModbusNodeData,
  ProtocolNodeData,
} from '@/types/nodes'

interface PetraConfig {
  signals: any[]
  blocks: any[]
  scan_time_ms: number
  twilio?: any
  mqtt?: any
  s7?: any
  modbus?: any
  protocols?: any
}

// Helper function to get label as string
function getNodeLabel(data: any): string {
  if (typeof data.label === 'string') return data.label
  return ''
}

// Helper to clean signal names for YAML
function cleanSignalName(name: string): string {
  return name.toLowerCase().replace(/[^a-z0-9_]/g, '_')
}

export function generateYaml(nodes: Node[], edges: Edge[]): string {
  const config: PetraConfig = {
    signals: [],
    blocks: [],
    scan_time_ms: 100,
  }

  // If no nodes selected, return empty config
  if (!nodes || nodes.length === 0) {
    return yaml.stringify(config, { 
      indent: 2, 
      lineWidth: 0,
      nullStr: '',
    })
  }

  // Maps to track signals and connections
  const signalNameMap = new Map<string, string>()
  const connectionMap = new Map<string, string>()
  let signalCounter = 1

  // First pass: Process signal nodes
  nodes
    .filter((n) => n.type === 'signal')
    .forEach((n) => {
      const data = n.data as SignalNodeData
      const signalName = cleanSignalName(data.signalName || data.label || `signal_${signalCounter++}`)
      signalNameMap.set(n.id, signalName)
      
      config.signals.push({
        name: signalName,
        type: data.signalType || 'float',
        initial: data.initial ?? 0,
        mode: data.mode || 'write',
      })
    })

  // Second pass: Create signals for inter-block connections
  edges.forEach(edge => {
    const sourceNode = nodes.find(n => n.id === edge.source)
    const targetNode = nodes.find(n => n.id === edge.target)
    
    // Skip if either end is already a signal node
    if (sourceNode?.type === 'signal' || targetNode?.type === 'signal') {
      return
    }
    
    // Create connection mapping
    const connectionKey = `${edge.source}.${edge.sourceHandle}_to_${edge.target}.${edge.targetHandle}`
    const signalName = cleanSignalName(`sig_${connectionKey}`)
    
    connectionMap.set(`${edge.source}.${edge.sourceHandle}`, signalName)
    connectionMap.set(`${edge.target}.${edge.targetHandle}`, signalName)
    
    // Add signal if not already present
    if (!config.signals.find(s => s.name === signalName)) {
      config.signals.push({
        name: signalName,
        type: 'float', // Default type, could be inferred from block types
        initial: 0
      })
    }
  })

  // Third pass: Process blocks
  nodes
    .filter((n) => n.type === 'block' || n.type === 'logicBlock')
    .forEach((n) => {
      const data = n.data as BlockNodeData
      const blockName = cleanSignalName(data.label || `${data.blockType}_${n.id}`)
      
      const inputs: Record<string, string> = {}
      const outputs: Record<string, string> = {}

      // Map inputs based on connections
      if (data.blockType === 'AND' || data.blockType === 'OR') {
        // For AND/OR gates, only map connected inputs
        const inputCount = data.inputCount || 2
        for (let i = 0; i < inputCount && i < data.inputs.length; i++) {
          const inputName = data.inputs[i].name
          const connectionKey = `${n.id}.${inputName}`
          
          // Check if there's a connection to this input
          const connectedEdge = edges.find(e => e.target === n.id && e.targetHandle === inputName)
          if (connectedEdge) {
            const sourceNode = nodes.find(node => node.id === connectedEdge.source)
            if (sourceNode?.type === 'signal') {
              inputs[inputName] = signalNameMap.get(sourceNode.id) || 'unconnected'
            } else {
              inputs[inputName] = connectionMap.get(connectionKey) || 'unconnected'
            }
          }
        }
      } else {
        // For other blocks, map all defined inputs
        data.inputs.forEach(input => {
          const connectionKey = `${n.id}.${input.name}`
          const connectedEdge = edges.find(e => e.target === n.id && e.targetHandle === input.name)
          
          if (connectedEdge) {
            const sourceNode = nodes.find(node => node.id === connectedEdge.source)
            if (sourceNode?.type === 'signal') {
              inputs[input.name] = signalNameMap.get(sourceNode.id) || 'unconnected'
            } else {
              inputs[input.name] = connectionMap.get(connectionKey) || 'unconnected'
            }
          }
        })
      }

      // Map outputs
      data.outputs.forEach(output => {
        const connectionKey = `${n.id}.${output.name}`
        const connectedEdge = edges.find(e => e.source === n.id && e.sourceHandle === output.name)
        
        if (connectedEdge) {
          const targetNode = nodes.find(node => node.id === connectedEdge.target)
          if (targetNode?.type === 'signal') {
            outputs[output.name] = signalNameMap.get(targetNode.id) || `${blockName}_${output.name}`
          } else {
            outputs[output.name] = connectionMap.get(connectionKey) || `${blockName}_${output.name}`
          }
        } else {
          // Create default output signal name
          outputs[output.name] = `${blockName}_${output.name}`
        }
      })

      const block: any = {
        name: blockName,
        type: data.blockType,
        inputs,
        outputs,
      }

      // Add parameters if present
      if (data.params && Object.keys(data.params).length > 0) {
        block.params = data.params
      }

      config.blocks.push(block)
    })

  // Fourth pass: Process protocol nodes
  const mqttNodes = nodes.filter(n => 
    (n.type === 'mqtt') || 
    (n.type === 'protocol' && (n.data as ProtocolNodeData).protocolType === 'MQTT')
  )
  
  if (mqttNodes.length > 0) {
    // Combine all MQTT configurations
    const mqttConfig: any = {
      broker_host: 'localhost',
      broker_port: 1883,
      client_id: `petra_${Date.now()}`,
      subscriptions: [],
      publications: []
    }

    mqttNodes.forEach(node => {
      const data = node.data as MqttNodeData | ProtocolNodeData
      
      if ('brokerHost' in data) {
        // MqttNodeData
        mqttConfig.broker_host = data.brokerHost
        mqttConfig.broker_port = data.brokerPort
        mqttConfig.client_id = data.clientId || mqttConfig.client_id
        
        if (data.username) mqttConfig.username = data.username
        if (data.password) mqttConfig.password = data.password
      } else if ('config' in data) {
        // ProtocolNodeData
        if (data.config.broker) mqttConfig.broker_host = data.config.broker
        if (data.config.port) mqttConfig.broker_port = data.config.port
        
        // Handle subscriptions/publications based on direction
        const signalName = signalNameMap.get(node.id) || `mqtt_${node.id}_data`
        
        if (data.config.direction !== 'publish') {
          mqttConfig.subscriptions.push({
            topic: data.config.topic || 'petra/+',
            signal: signalName,
            qos: 1
          })
        }
        
        if (data.config.direction !== 'subscribe') {
          mqttConfig.publications.push({
            topic: data.config.topic || 'petra/out',
            signal: signalName,
            qos: 1,
            retain: false
          })
        }
      }
    })

    config.mqtt = mqttConfig
  }

  // Process S7 nodes
  const s7Nodes = nodes.filter(n => n.type === 's7')
  if (s7Nodes.length > 0) {
    config.s7 = {
      connections: s7Nodes.map(node => {
        const data = node.data as S7NodeData
        return {
          name: data.label,
          ip: data.ip,
          rack: data.rack,
          slot: data.slot,
          tags: [{
            name: data.signal || `s7_${node.id}_data`,
            area: data.area,
            db_number: data.dbNumber,
            address: data.address,
            data_type: data.dataType,
            bit: data.bit,
            direction: data.direction
          }]
        }
      })
    }
  }

  // Process Modbus nodes
  const modbusNodes = nodes.filter(n => n.type === 'modbus')
  if (modbusNodes.length > 0) {
    config.modbus = {
      connections: modbusNodes.map(node => {
        const data = node.data as ModbusNodeData
        return {
          name: data.label,
          host: data.host,
          port: data.port,
          unit_id: data.unitId,
          registers: [{
            signal: data.signal || `modbus_${node.id}_data`,
            address: data.address,
            data_type: data.dataType,
            direction: data.direction
          }]
        }
      })
    }
  }

  // Process Twilio nodes
  const twilioNodes = nodes.filter(n => n.type === 'twilio')
  if (twilioNodes.length > 0) {
    config.twilio = {
      account_sid: 'YOUR_ACCOUNT_SID',
      auth_token: 'YOUR_AUTH_TOKEN',
      from_number: '+1234567890',
      actions: twilioNodes.map(node => {
        const data = node.data as TwilioNodeData
        return {
          name: data.label,
          type: data.actionType,
          to_number: data.toNumber,
          content: data.content,
          trigger_signal: signalNameMap.get(node.id) || `twilio_${node.id}_trigger`
        }
      })
    }
  }

  // Generate clean YAML with proper formatting
  const yamlOptions = {
    indent: 2,
    lineWidth: 0,
    nullStr: '',
    flowLevel: -1,
    sortKeys: false,
  }

  return yaml.stringify(config, yamlOptions)
}

// Helper function to validate the generated configuration
export function validateYamlConfig(yamlString: string): { valid: boolean; errors: string[] } {
  const errors: string[] = []
  
  try {
    const config = yaml.parse(yamlString)
    
    // Check for required fields
    if (!config.signals || !Array.isArray(config.signals)) {
      errors.push('Missing or invalid signals array')
    }
    
    if (!config.blocks || !Array.isArray(config.blocks)) {
      errors.push('Missing or invalid blocks array')
    }
    
    if (typeof config.scan_time_ms !== 'number') {
      errors.push('Missing or invalid scan_time_ms')
    }
    
    // Validate signal references
    const signalNames = new Set(config.signals?.map((s: any) => s.name) || [])
    
    config.blocks?.forEach((block: any) => {
      Object.values(block.inputs || {}).forEach((signal: any) => {
        if (signal !== 'unconnected' && !signalNames.has(signal)) {
          errors.push(`Block ${block.name}: Input references undefined signal '${signal}'`)
        }
      })
    })
    
  } catch (e) {
    errors.push(`YAML parsing error: ${e}`)
  }
  
  return {
    valid: errors.length === 0,
    errors
  }
}
