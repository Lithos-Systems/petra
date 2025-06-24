import { Node, Edge } from '@xyflow/react'
import * as yaml from 'yaml'

export function generateYaml(nodes: Node[], edges: Edge[]): string {
  const config: any = {
    signals: [],
    blocks: [],
    scan_time_ms: 100,
  }

  // Process signal nodes
  const signalNodes = nodes.filter(n => n.type === 'signal')
  config.signals = signalNodes.map(node => ({
    name: node.data.label.toLowerCase().replace(/\s+/g, '_'),
    type: node.data.signalType,
    initial: node.data.initial,
  }))

  // Process block nodes
  const blockNodes = nodes.filter(n => n.type === 'block')
  config.blocks = blockNodes.map(node => {
    const inputs: any = {}
    const outputs: any = {}

    // Map connections
    edges.forEach(edge => {
      if (edge.target === node.id) {
        const sourceNode = nodes.find(n => n.id === edge.source)
        if (sourceNode) {
          const inputName = edge.targetHandle || 'in'
          inputs[inputName] = sourceNode.data.label.toLowerCase().replace(/\s+/g, '_')
        }
      }
      if (edge.source === node.id) {
        const targetNode = nodes.find(n => n.id === edge.target)
        if (targetNode && edge.sourceHandle) {
          outputs[edge.sourceHandle] = targetNode.data.label.toLowerCase().replace(/\s+/g, '_')
        }
      }
    })

    return {
      name: node.data.label.toLowerCase().replace(/\s+/g, '_'),
      type: node.data.blockType,
      inputs,
      outputs,
      ...(Object.keys(node.data.params || {}).length > 0 ? { params: node.data.params } : {}),
    }
  })

  // Process Twilio nodes
  const twilioNodes = nodes.filter(n => n.type === 'twilio')
  if (twilioNodes.length > 0) {
    config.twilio = {
      from_number: '+1234567890', // Default, should be configured
      actions: twilioNodes.map(node => ({
        name: node.data.label.toLowerCase().replace(/\s+/g, '_'),
        trigger_signal: edges.find(e => e.target === node.id)?.source || '',
        action_type: node.data.actionType,
        to_number: node.data.toNumber,
        content: node.data.content,
      })),
    }
  }

  // Process MQTT nodes
  const mqttNode = nodes.find(n => n.type === 'mqtt')
  if (mqttNode) {
    config.mqtt = {
      broker_host: mqttNode.data.brokerHost,
      broker_port: mqttNode.data.brokerPort,
      client_id: mqttNode.data.clientId || 'petra-01',
      topic_prefix: mqttNode.data.topicPrefix || 'petra/plc',
      publish_on_change: true,
    }
  }

  // Process S7 nodes
  const s7Nodes = nodes.filter(n => n.type === 's7')
  if (s7Nodes.length > 0) {
    config.s7 = {
      ip: '192.168.1.100', // Default, should be configured
      rack: 0,
      slot: 2,
      poll_interval_ms: 100,
      mappings: s7Nodes.map(node => ({
        signal: node.data.signal,
        area: node.data.area,
        db_number: node.data.dbNumber,
        address: node.data.address,
        data_type: node.data.dataType,
        direction: node.data.direction,
      })),
    }
  }

  return yaml.stringify(config, {
    indent: 2,
    lineWidth: 0,
  })
}
