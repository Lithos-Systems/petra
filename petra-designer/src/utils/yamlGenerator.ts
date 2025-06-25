import { Edge, Node } from '@xyflow/react'
import * as yaml from 'yaml'

interface PetraConfig {
  signals: any[]
  blocks: any[]
  scan_time_ms: number
  twilio?: any
  mqtt?: any
  s7?: any
}

export function generateYaml(nodes: Node[], edges: Edge[]): string {
  const config: PetraConfig = {
    signals: [],
    blocks: [],
    scan_time_ms: 100,
  }

  /* ------------------------------------------------ signals ------ */
  nodes
    .filter((n) => n.type === 'signal')
    .forEach((n) =>
      config.signals.push({
        name: String(n.data.label || 'unnamed_signal').toLowerCase().replace(/\s+/g, '_'),
        type: n.data.signalType || 'float',
        initial: n.data.initial ?? 0,
      }),
    )

  /* ------------------------------------------------ blocks ------- */
  nodes
    .filter((n) => n.type === 'block')
    .forEach((n) => {
      const inputs: Record<string, string> = {}
      const outputs: Record<string, string> = {}

      edges.forEach((e) => {
        if (e.target === n.id) {
          const src = nodes.find((m) => m.id === e.source)
          if (src)
            inputs[e.targetHandle ?? 'in'] = String(src.data.label || 'unnamed')
              .toLowerCase()
              .replace(/\s+/g, '_')
        }
        if (e.source === n.id) {
          const tgt = nodes.find((m) => m.id === e.target)
          if (tgt && e.sourceHandle)
            outputs[e.sourceHandle] = String(tgt.data.label || 'unnamed').toLowerCase().replace(/\s+/g, '_')
        }
      })

      config.blocks.push({
        name: String(n.data.label || 'unnamed_block').toLowerCase().replace(/\s+/g, '_'),
        type: n.data.blockType || 'AND',
        inputs,
        outputs,
        ...(n.data.params && Object.keys(n.data.params).length
          ? { params: n.data.params }
          : {}),
      })
    })

  /* ------------------------------------------------ twilio ------- */
  const twilioNodes = nodes.filter((n) => n.type === 'twilio')
  if (twilioNodes.length) {
    config.twilio = {
      from_number: '+1234567890',
      actions: twilioNodes.map((n) => ({
        name: String(n.data.label || 'unnamed_twilio').toLowerCase().replace(/\s+/g, '_'),
        trigger_signal: edges.find((e) => e.target === n.id)?.source ?? '',
        action_type: n.data.actionType || 'sms',
        to_number: n.data.toNumber || '',
        content: n.data.content || '',
      })),
    }
  }

  /* ------------------------------------------------ mqtt --------- */
  const mqttNode = nodes.find((n) => n.type === 'mqtt')
  if (mqttNode) {
    config.mqtt = {
      broker_host: mqttNode.data.brokerHost || 'mqtt.lithos.systems',
      broker_port: mqttNode.data.brokerPort || 1883,
      client_id: mqttNode.data.clientId || 'petra-01',
      topic_prefix: mqttNode.data.topicPrefix || 'petra/plc',
      publish_on_change: true,
    }
  }

  /* ------------------------------------------------ s7 ----------- */
  const s7Nodes = nodes.filter((n) => n.type === 's7')
  if (s7Nodes.length) {
    config.s7 = {
      ip: '192.168.1.100',
      rack: 0,
      slot: 2,
      poll_interval_ms: 100,
      mappings: s7Nodes.map((n) => ({
        signal: n.data.signal || '',
        area: n.data.area || 'DB',
        db_number: n.data.dbNumber || 0,
        address: n.data.address || 0,
        data_type: n.data.dataType || 'bool',
        direction: n.data.direction || 'read',
      })),
    }
  }

  return yaml.stringify(config, { indent: 2, lineWidth: 0 })
}
