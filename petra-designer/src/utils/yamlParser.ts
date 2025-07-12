import { Node, Edge } from '@xyflow/react'
import * as yaml from 'yaml'
import type { SignalNodeData, BlockNodeData } from '@/types/nodes'

export interface ParsedFlow {
  nodes: Node[]
  edges: Edge[]
}

export function parseYamlConfig(yamlString: string): ParsedFlow {
  const config = yaml.parse(yamlString)

  const nodes: Node[] = []
  const edges: Edge[] = []
  const signalMap = new Map<string, string>()

  const signals: any[] = Array.isArray(config?.signals) ? config.signals : []
  signals.forEach((sig, idx) => {
    const id = `signal_${idx}`
    signalMap.set(sig.name, id)

    const node: Node<SignalNodeData> = {
      id,
      type: 'signal',
      position: { x: 0, y: idx * 80 },
      data: {
        label: sig.name,
        signalName: sig.name,
        signalType: sig.type || 'float',
        initial: sig.initial ?? 0,
        mode: sig.mode || 'write',
      },
    }
    nodes.push(node)
  })

  const blocks: any[] = Array.isArray(config?.blocks) ? config.blocks : []
  blocks.forEach((blk, idx) => {
    const id = `block_${idx}`
    const inputEntries = Object.entries(blk.inputs || {}) as Array<[string, string]>
    const outputEntries = Object.entries(blk.outputs || {}) as Array<[string, string]>

    const node: Node<BlockNodeData> = {
      id,
      type: 'block',
      position: { x: 300, y: idx * 120 },
      data: {
        label: blk.name || id,
        blockType: blk.type,
        inputs: inputEntries.map(([name]) => ({ name, type: 'any' })),
        outputs: outputEntries.map(([name]) => ({ name, type: 'any' })),
        params: blk.params || {},
        inputCount: inputEntries.length,
      },
    }
    nodes.push(node)

    for (const [inputName, signalName] of inputEntries) {
      const sigId = signalMap.get(signalName)
      if (sigId) {
        edges.push({
          id: `${sigId}-${id}-${inputName}`,
          source: sigId,
          target: id,
          targetHandle: inputName,
        })
      }
    }

    for (const [outputName, signalName] of outputEntries) {
      const sigId = signalMap.get(signalName)
      if (sigId) {
        edges.push({
          id: `${id}-${sigId}-${outputName}`,
          source: id,
          target: sigId,
          sourceHandle: outputName,
        })
      }
    }
  })

  return { nodes, edges }
}
