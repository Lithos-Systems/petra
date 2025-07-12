import { describe, it, expect } from 'vitest'
import { generateYaml, validateYamlConfig } from '../yamlGenerator'
import type { Node, Edge } from '@xyflow/react'
import type { SignalNodeData, BlockNodeData, MqttNodeData, S7NodeData } from '@/types/nodes'

function makeSignal(id: string, data: Partial<SignalNodeData> = {}): Node {
  return {
    id,
    type: 'signal',
    position: { x: 0, y: 0 },
    data: {
      label: data.label ?? id,
      signalName: data.signalName ?? id,
      signalType: data.signalType ?? 'float',
      initial: data.initial ?? 0,
      mode: data.mode ?? 'write'
    }
  }
}

function makeBlock(id: string, blockType: string, inputs: string[], outputs: string[]): Node {
  return {
    id,
    type: 'block',
    position: { x: 0, y: 0 },
    data: {
      label: id,
      blockType,
      inputs: inputs.map(name => ({ name, type: 'float' })),
      outputs: outputs.map(name => ({ name, type: 'float' }))
    }
  }
}

describe('generateYaml', () => {
  it('Empty flow generates valid YAML', () => {
    const yaml = generateYaml([], [])
    const res = validateYamlConfig(yaml)
    expect(res.valid).toBe(true)
    expect(yaml).toContain('signals: []')
    expect(yaml).toContain('blocks: []')
  })

  it('Signal nodes convert correctly', () => {
    const nodes: Node[] = [makeSignal('sig1', { signalType: 'int' })]
    const yaml = generateYaml(nodes, [])
    expect(yaml).toMatch(/name: sig1/)
    expect(yaml).toMatch(/type: int/)
    expect(validateYamlConfig(yaml).valid).toBe(true)
  })

  it('Block nodes with connections', () => {
    const sig = makeSignal('input')
    const block = makeBlock('b1', 'ADD', ['a', 'b'], ['out'])
    const nodes = [sig, block]
    const edges: Edge[] = [
      { id: 'e1', source: sig.id, target: block.id, targetHandle: 'a' },
      { id: 'e2', source: block.id, sourceHandle: 'out', target: sig.id }
    ] as any
    const yaml = generateYaml(nodes, edges)
    expect(yaml).toMatch(/blocks:/)
    expect(yaml).toMatch(/inputs:/)
    expect(validateYamlConfig(yaml).valid).toBe(true)
  })

  it('Protocol nodes (MQTT, S7)', () => {
    const mqttNode: Node = {
      id: 'mqtt1',
      type: 'mqtt',
      position: { x: 0, y: 0 },
      data: {
        label: 'mqtt',
        configured: true,
        brokerHost: 'localhost',
        brokerPort: 1883,
        clientId: 'client',
        topicPrefix: 'petra',
        mode: 'read',
        publishOnChange: false
      } as MqttNodeData
    }
    const s7Node: Node = {
      id: 's7_1',
      type: 's7',
      position: { x: 0, y: 0 },
      data: {
        label: 's7',
        configured: true,
        ip: '192.168.0.1',
        rack: 0,
        slot: 1,
        area: 'DB',
        dbNumber: 1,
        address: 0,
        dataType: 'int',
        direction: 'read',
        signal: 'sig'
      } as S7NodeData
    }
    const yaml = generateYaml([mqttNode, s7Node], [])
    expect(yaml).toMatch(/mqtt:/)
    expect(yaml).toMatch(/s7:/)
    expect(validateYamlConfig(yaml).valid).toBe(true)
  })

  it('Edge cases and error handling', () => {
    // invalid yaml should return errors
    const res = validateYamlConfig('bad: [yaml')
    expect(res.valid).toBe(false)
    expect(res.errors.length).toBeGreaterThan(0)
  })
})
