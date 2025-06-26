// src/utils/yamlGenerator.ts
import { Edge, Node } from '@xyflow/react'
import * as yaml from 'yaml'
import type {
  SignalNodeData,
  BlockNodeData,
  TwilioNodeData,
  MqttNodeData,
  S7NodeData,
} from '@/types/nodes'

interface PetraConfig {
  signals: any[]
  blocks: any[]
  scan_time_ms: number
  twilio?: any
  mqtt?: any
  s7?: any
}

// Helper function to get label as string
function getNodeLabel(data: any): string {
  if (typeof data.label === 'string') return data.label
  return ''
}

export function generateYaml(nodes: Node[], edges: Edge[]): string {
  const config: PetraConfig = {
    signals: [],
    blocks: [],
    scan_time_ms: 100,
  }

  // Generate unique signal names
  const signalNameMap = new Map<string, string>()
  
  nodes
    .filter((n) => n.type === 'signal')
    .forEach((n, index) => {
      const data = n.data as SignalNodeData
      const label = getNodeLabel(data)
      const baseName = label || `signal_${index}`
      const cleanName = baseName.toLowerCase().replace(/[^a-z0-9_]/g, '_')
      signalNameMap.set(n.id, cleanName)
      
      config.signals.push({
        name: cleanName,
        type: data.signalType || 'float',
        initial: data.initial ?? 0,
      })
    })

  // Process blocks with proper connections
  nodes
    .filter((n) => n.type === 'block')
    .forEach((n, index) => {
      const data = n.data as BlockNodeData
      const label = getNodeLabel(data)
      const blockName = (label || `block_${index}`).toLowerCase().replace(/[^a-z0-9_]/g, '_')
      
      const inputs: Record<string, string> = {}
      const outputs: Record<string, string> = {}

      // Map inputs
      edges
        .filter(e => e.target === n.id)
        .forEach(e => {
          const sourceNode = nodes.find(m => m.id === e.source)
          if (sourceNode) {
            const sourceLabel = getNodeLabel(sourceNode.data)
            const signalName = signalNameMap.get(sourceNode.id) || 
                             (sourceLabel ? sourceLabel.toLowerCase().replace(/[^a-z0-9_]/g, '_') : 'unnamed')
            inputs[e.targetHandle || 'in'] = signalName
          }
        })

      // Map outputs
      edges
        .filter(e => e.source === n.id)
        .forEach(e => {
          const targetNode = nodes.find(m => m.id === e.target)
          if (targetNode) {
            const targetLabel = getNodeLabel(targetNode.data)
            const signalName = signalNameMap.get(targetNode.id) || 
                             (targetLabel ? targetLabel.toLowerCase().replace(/[^a-z0-9_]/g, '_') : 'unnamed')
            outputs[e.sourceHandle || 'out'] = signalName
          }
        })

      const block: any = {
        name: blockName,
        type: data.blockType || 'AND',
        inputs,
        outputs,
      }

      // Add params if they exist
      if (data.params && Object.keys(data.params).length > 0) {
        block.params = data.params
      }

      config.blocks.push(block)
    })

  // Process Twilio nodes
  const twilioNodes = nodes.filter((n) => n.type === 'twilio' && n.data.configured)
  if (twilioNodes.length > 0) {
    config.twilio = {
      from_number: '+1234567890', // Default, should be configured
      actions: twilioNodes.map((n, index) => {
        const data = n.data as TwilioNodeData
        const label = getNodeLabel(data)
        const name = (label || `twilio_${index}`).toLowerCase().replace(/[^a-z0-9_]/g, '_')
        
        // Find trigger signal
        const triggerEdge = edges.find(e => e.target === n.id)
        const triggerNode = triggerEdge ? nodes.find(m => m.id === triggerEdge.source) : null
        let triggerSignal = 'unknown_trigger'
        
        if (triggerNode) {
          const triggerLabel = getNodeLabel(triggerNode.data)
          triggerSignal = signalNameMap.get(triggerNode.id) || 
                         (triggerLabel ? triggerLabel.toLowerCase().replace(/[^a-z0-9_]/g, '_') : 'unknown_trigger')
        }

        return {
          name,
          trigger_signal: triggerSignal,
          action_type: data.actionType || 'sms',
          to_number: data.toNumber || '+1234567890',
          content: data.content || 'Alert from Petra',
          cooldown_seconds: 300,
        }
      }),
    }
  }

  // Process MQTT node
  const mqttNode = nodes.find((n) => n.type === 'mqtt' && n.data.configured)
  if (mqttNode) {
    const data = mqttNode.data as MqttNodeData
    config.mqtt = {
      broker_host: data.brokerHost || 'mqtt.lithos.systems',
      broker_port: data.brokerPort || 1883,
      client_id: data.clientId || 'petra-01',
      topic_prefix: data.topicPrefix || 'petra/plc',
      publish_on_change: true,
    }
  }

  // Process S7 nodes
  const s7Nodes = nodes.filter((n) => n.type === 's7' && n.data.configured)
  if (s7Nodes.length > 0) {
    config.s7 = {
      ip: '192.168.1.100',
      rack: 0,
      slot: 2,
      poll_interval_ms: 100,
      mappings: s7Nodes.map((n) => {
        const data = n.data as S7NodeData
        return {
          signal: data.signal || '',
          area: data.area || 'DB',
          db_number: data.dbNumber || 0,
          address: data.address || 0,
          data_type: data.dataType || 'bool',
          direction: data.direction || 'read',
          ...(data.dataType === 'bool' ? { bit: 0 } : {}),
        }
      }),
    }
  }

  return yaml.stringify(config, { 
    indent: 2, 
    lineWidth: 0,
    nullStr: '',
  })
}
