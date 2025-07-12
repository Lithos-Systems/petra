import { describe, it, expect } from 'vitest'
import { generateYaml } from '../utils/yamlGenerator'
import { parseYamlConfig } from '../utils/yamlParser'
import { validateConnection } from '../utils/validation'
import { useFlowStore } from '../store/flowStore'
import type { Node, Edge, Connection } from '@xyflow/react'

function makeSignal(id: string): Node {
  return { id, type: 'signal', position: { x:0, y:0 }, data: { label:id, signalName:id, signalType:'float', initial:0, mode:'write' } }
}

function makeBlock(id: string): Node {
  return { id, type: 'block', position: { x:0, y:0 }, data: { label:id, blockType:'ADD', inputs:[{name:'a', type:'float'}], outputs:[{name:'out', type:'float'}] } }
}

describe('integration workflow', () => {
  it('create flow -> export -> import', () => {
    const sig = makeSignal('sig1')
    const block = makeBlock('b1')
    const edges: Edge[] = [{ id:'e1', source:sig.id, target:block.id, targetHandle:'a' } as any]
    const yaml = generateYaml([sig, block], edges)
    const parsed = parseYamlConfig(yaml)
    expect(parsed.nodes.length).toBe(2)
    expect(parsed.edges.length).toBe(1)
  })

  it('drag and drop updates node position', () => {
    const node = makeSignal('drag')
    useFlowStore.setState({ nodes:[node], edges:[], selectedNode:null })
    useFlowStore.getState().onNodesChange([{ id: node.id, type:'position', position:{ x:10, y:20 } }] as any)
    const updated = useFlowStore.getState().nodes.find(n => n.id===node.id)
    expect(updated?.position).toEqual({ x:10, y:20 })
  })

  it('connection validation prevents duplicates', () => {
    const n1 = makeSignal('s1')
    const n2 = makeBlock('b1')
    const edges: Edge[] = []
    const conn: Connection = { source:n1.id, target:n2.id, sourceHandle:undefined, targetHandle:'a' }
    const valid = validateConnection(conn, [n1,n2], edges)
    expect(valid.valid).toBe(true)
    edges.push({ id:'e1', source:n1.id, target:n2.id, targetHandle:'a' } as any)
    const second = validateConnection(conn, [n1,n2], edges)
    expect(second.valid).toBe(false)
  })
})
