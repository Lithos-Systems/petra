import { render, screen } from '@testing-library/react'
import YamlPreview from '../YamlPreview'
import { useOptimizedFlowStore } from '@/store/optimizedFlowStore'

function makeSignal() {
  return { id:'s1', type:'signal', position:{x:0,y:0}, data:{ label:'s1', signalName:'s1', signalType:'float', initial:0, mode:'write' } }
}

describe('YamlPreview component', () => {
  it('renders YAML from store', () => {
    useOptimizedFlowStore.setState({ nodes:[makeSignal()], edges:[] })
    render(<YamlPreview />)
    expect(screen.getByText('YAML Preview')).toBeInTheDocument()
    expect(screen.getByText(/signals:/)).toBeInTheDocument()
  })
})
