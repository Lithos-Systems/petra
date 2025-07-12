import { render, screen, fireEvent } from '@testing-library/react'
import Toolbar from '../Toolbar'
import { useOptimizedFlowStore } from '@/store/optimizedFlowStore'
import toast from 'react-hot-toast'
import { vi } from 'vitest'

vi.mock('react-hot-toast', () => ({ default: { success: vi.fn(), error: vi.fn() } }))

const mockedToast = toast as unknown as { success: any; error: any }

describe('Toolbar interactions', () => {
  beforeEach(() => {
    useOptimizedFlowStore.setState({ nodes: [], edges: [], selectedNode: null })
    mockedToast.success.mockClear()
    mockedToast.error.mockClear()
  })

  it('validates logic on button click', () => {
    const validateLogic = vi.fn(() => ({ valid: true, nodeCount: 1, connectionCount: 0, errors: [] }))
    useOptimizedFlowStore.setState({ validateLogic })

    render(<Toolbar />)
    fireEvent.click(screen.getByTitle('Validate Logic'))

    expect(validateLogic).toHaveBeenCalled()
    expect(mockedToast.success).toHaveBeenCalled()
  })

  it('deletes selected block', () => {
    const deleteSelectedNode = vi.fn()
    useOptimizedFlowStore.setState({ selectedNode: { id: '1' } as any, deleteSelectedNode })

    render(<Toolbar />)
    fireEvent.click(screen.getByTitle('Delete Selected Block'))

    expect(deleteSelectedNode).toHaveBeenCalled()
  })
})
