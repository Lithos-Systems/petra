// src/hooks/useKeyboardShortcuts.ts
import { useEffect } from 'react'
import { useOptimizedFlowStore } from '@/store/optimizedFlowStore'
import toast from 'react-hot-toast'

export function useKeyboardShortcuts() {
  const { selectedNode, deleteNode, clearFlow, nodes, edges } = useOptimizedFlowStore()

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ignore events when typing inside inputs or other editable elements
      const target = e.target as HTMLElement | null
      const isEditable =
        target &&
        (target.tagName === 'INPUT' ||
          target.tagName === 'TEXTAREA' ||
          target.isContentEditable)
      if (isEditable) return

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
