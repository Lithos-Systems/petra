import { Edge } from '@xyflow/react'
import * as yaml from 'yaml'
import type { PetraNode } from '@/types/nodes'

interface PetraConfig {
  signals: any[]
  blocks: any[]
  scan_time_ms: number
  twilio?: any
  mqtt?: any
  s7?: any
}

export function generateYaml(nodes: PetraNode[], edges: Edge[]): string {
  const config: PetraConfig = {
    signals: [],
    blocks: [],
    scan_time_ms: 100,
  }

  /* ------------------------------------------------ signals ------ */
  nodes
    .filter((n): n is Extract<PetraNode, { type: 'signal' }> => n.type === 'signal')
    .forEach((n) =>
      config.signals.push({
        name: n.data.label.toLowerCase().replace(/\s+/g, '_'),
        type: n.data.signalType,
        initial: n.data.initial,
      }),
    )

  /* ------------------------------------------------ blocks ------- */
  nodes
    .filter((n): n is Extract<PetraNode, { type: 'block' }> => n.type === 'block')
    .forEach((n) => {
      const inputs: Record<string, string> = {}
      const outputs: Record<string, string> = {}

      edges.forEach((e) => {
        if (e.target === n.id) {
          const src = nodes.find((m) => m.id === e.source)
          if (src)
            inputs[e.targetHandle ?? 'in'] = src.data.label
              .toLowerCase()
              .replace(/\s+/g, '_')
        }
        if (e.source === n.id) {
          const tgt = nodes.find((m) => m.id === e.target)
          if (tgt && e.sourceHandle)
            outputs[e.sourceHandle] = tgt.data.label.toLowerCase().replace(/\s+/g, '_')
        }
      })

      config.blocks.push({
        name: n.data.label.toLowerCase().replace(/\s+/g, '_'),
        type: n.data.blockType,
        inputs,
        outputs,
        ...(n.data.params && Object.keys(n.data.params).length
          ? { params: n.data.params }
          : {}),
      })
    })

  /* ------------------------------------------------ twilio ------- */
  const twilioNodes = nodes.filter(
    (n): n is Extract<PetraNode, { type: 'twilio' }> => n.type === 'twilio',
  )
  if (twilioNodes.length) {
    config.twilio = {
      from_number: '+1234567890',
      actions: twilioNodes.map((n) => ({
        name: n.data.label.toLowerCase().replace(/\s+/g, '_'),
        trigger_signal: edges.find((e) => e.target === n.id)?.source ?? '',
        action_type: n.data.actionType,
        to_number: n.data.toNumber,
        content: n.data.content,
      })),
    }
  }

  /* ------------------------------------------------ mqtt --------- */
  const mqttNode = nodes.find(
    (n): n is Extract<PetraNode, { type: 'mqtt' }> => n.type === 'mqtt',
  )
  if (mqttNode) {
    config.mqtt = {
      broker_host: mqttNode.data.brokerHost,
      broker_port: mqttNode.data.brokerPort,
      client_id: mqttNode.data.clientId || 'petra-01',
      topic_prefix: mqttNode.data.topicPrefix || 'petra/plc',
      publish_on_change: true,
    }
  }

  /* ------------------------------------------------ s7 ----------- */
  const s7Nodes = nodes.filter(
    (n): n is Extract<PetraNode, { type: 's7' }> => n.type === 's7',
  )
  if (s7Nodes.length) {
    config.s7 = {
      ip: '192.168.1.100',
      rack: 0,
      slot: 2,
      poll_interval_ms: 100,
      mappings: s7Nodes.map((n) => ({
        signal: n.data.signal,
        area: n.data.area,
        db_number: n.data.dbNumber,
        address: n.data.address,
        data_type: n.data.dataType,
        direction: n.data.direction,
      })),
    }
  }

  return yaml.stringify(config, { indent: 2, lineWidth: 0 })
}
