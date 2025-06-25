import { Edge } from '@xyflow/react'
import * as yaml from 'yaml'
import type { PetraNode, BlockNodeData } from '@/types/nodes'
/* ---- TWILIO nodes ---- */
const twilioNodes = nodes.filter(
  (n): n is Extract<PetraNode, { type: 'twilio' }> => n.type === 'twilio',
)

/* ---- S7 nodes ---- */
const s7 = nodes.filter(
  (n): n is Extract<PetraNode, { type: 's7' }> => n.type === 's7',
)

export function generateYaml(nodes: PetraNode[], edges: Edge[]): string {
  const config: any = {
    signals: [],
    blocks: [],
    scan_time_ms: 100,
  }

  /* ---------- signals ---------- */
  nodes
    .filter((n): n is Extract<PetraNode, { type: 'signal' }> => n.type === 'signal')
    .forEach((node) =>
      config.signals.push({
        name: node.data.label.toLowerCase().replace(/\s+/g, '_'),
        type: node.data.signalType,
        initial: node.data.initial,
      }),
    )

  /* ---------- blocks ---------- */
  nodes
    .filter((n): n is Extract<PetraNode, { type: 'block' }> => n.type === 'block')
    .forEach((node) => {
      const inputs: Record<string, string> = {}
      const outputs: Record<string, string> = {}

      edges.forEach((e) => {
        if (e.target === node.id) {
          const src = nodes.find((n) => n.id === e.source)
          if (src)
            inputs[e.targetHandle ?? 'in'] = src.data.label
              .toLowerCase()
              .replace(/\s+/g, '_')
        }
        if (e.source === node.id) {
          const tgt = nodes.find((n) => n.id === e.target)
          if (tgt && e.sourceHandle)
            outputs[e.sourceHandle] = tgt.data.label.toLowerCase().replace(/\s+/g, '_')
        }
      })

      config.blocks.push({
        name: node.data.label.toLowerCase().replace(/\s+/g, '_'),
        type: node.data.blockType,
        inputs,
        outputs,
        ...(Object.keys((node.data as BlockNodeData).params ?? {}).length
          ? { params: (node.data as BlockNodeData).params }
          : {}),
      })
    })

  /* ---------- twilio ---------- */
  const twilioNodes = nodes.filter(
    (n): n is Extract<PetraNode, { type: 'twilio' }>,
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

  /* ---------- mqtt ---------- */
  const mqtt = nodes.find(
    (n): n is Extract<PetraNode, { type: 'mqtt' }> => n.type === 'mqtt',
  )
  if (mqtt) {
    config.mqtt = {
      broker_host: mqtt.data.brokerHost,
      broker_port: mqtt.data.brokerPort,
      client_id: mqtt.data.clientId || 'petra-01',
      topic_prefix: mqtt.data.topicPrefix || 'petra/plc',
      publish_on_change: true,
    }
  }

  /* ---------- s7 ---------- */
  const s7 = nodes.filter(
    (n): n is Extract<PetraNode, { type: 's7' }>,
  )
  if (s7.length) {
    config.s7 = {
      ip: '192.168.1.100',
      rack: 0,
      slot: 2,
      poll_interval_ms: 100,
      mappings: s7.map((n) => ({
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
