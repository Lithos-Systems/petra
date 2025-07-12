import { render, screen, fireEvent } from '@testing-library/react'
import PropertiesPanel from '../PropertiesPanel'
import { useFlowStore } from '@/store/flowStore'
import { vi } from 'vitest'

function makeSignal() {
  return { id:'s1', type:'signal', position:{x:0,y:0}, data:{ label:'sig', signalName:'sig', signalType:'bool', initial:false, mode:'write' } }
}

describe('PropertiesPanel', () => {
  it('updates label via updateNodeData', () => {
    const node = makeSignal()
    const updateNodeData = vi.fn()
    useFlowStore.setState({ nodes:[node], edges:[], selectedNode:node, updateNodeData })

    render(<PropertiesPanel />)
    const input = screen.getAllByDisplayValue('sig')[0] as HTMLInputElement
    fireEvent.change(input, { target: { value: 'new' } })
    expect(updateNodeData).toHaveBeenCalledWith(node.id, { label: 'new' })
  })
})
