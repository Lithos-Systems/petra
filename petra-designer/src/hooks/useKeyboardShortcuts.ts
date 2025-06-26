// src/hooks/useKeyboardShortcuts.ts
import { useEffect } from 'react'
import { useFlowStore } from '@/store/flowStore'
import toast from 'react-hot-toast'

export function useKeyboardShortcuts() {
  const { selectedNode, deleteNode, clearFlow, nodes, edges } = useFlowStore()

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Delete selected node
      if ((e.key === 'Delete' || e.key === 'Backspace') && selectedNode) {
        e.preventDefault()
        deleteNode(selectedNode.id)
        toast.success('Node deleted')
      }

      // Save (Ctrl/Cmd + S)
      if ((e.ctrlKey || e.metaKey) && e.key === 's') {
        e.preventDefault()
        localStorage.setItem('petra-flow', JSON.stringify({ nodes, edges }))
        toast.success('Flow saved')
      }

      // Clear all (Ctrl/Cmd + Shift + D)
      if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'd') {
        e.preventDefault()
        if (confirm('Clear all nodes and connections?')) {
          clearFlow()
          toast.success('Canvas cleared')
        }
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [selectedNode, deleteNode, clearFlow, nodes, edges])
}
