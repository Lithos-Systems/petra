// src/hooks/useHMIKeyboardShortcuts.ts

import { useEffect } from 'react'
import { useHMIStore } from '@/store/hmiStore'
import toast from 'react-hot-toast'

export function useHMIKeyboardShortcuts() {
  const {
    selectedComponentId,
    deleteComponent,
    duplicateComponent,
    undo,
    redo,
    canUndo,
    canRedo,
    saveDisplay,
    components,
  } = useHMIStore()

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Don't handle if user is typing in an input
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
        return
      }

      // Delete selected component
      if ((e.key === 'Delete' || e.key === 'Backspace') && selectedComponentId) {
        e.preventDefault()
        deleteComponent(selectedComponentId)
        toast.success('Component deleted')
      }

      // Ctrl/Cmd + Z: Undo
      if ((e.ctrlKey || e.metaKey) && e.key === 'z' && !e.shiftKey) {
        e.preventDefault()
        if (canUndo()) {
          undo()
          toast.success('Undo')
        }
      }

      // Ctrl/Cmd + Y or Ctrl/Cmd + Shift + Z: Redo
      if ((e.ctrlKey || e.metaKey) && (e.key === 'y' || (e.key === 'z' && e.shiftKey))) {
        e.preventDefault()
        if (canRedo()) {
          redo()
          toast.success('Redo')
        }
      }

      // Ctrl/Cmd + D: Duplicate
      if ((e.ctrlKey || e.metaKey) && e.key === 'd' && selectedComponentId) {
        e.preventDefault()
        duplicateComponent(selectedComponentId)
        toast.success('Component duplicated')
      }

      // Ctrl/Cmd + S: Save
      if ((e.ctrlKey || e.metaKey) && e.key === 's') {
        e.preventDefault()
        const display = saveDisplay()
        if (display) {
          localStorage.setItem('petra-hmi-display', JSON.stringify(display))
          toast.success('Display saved')
        }
      }

      // Ctrl/Cmd + A: Select all
      if ((e.ctrlKey || e.metaKey) && e.key === 'a') {
        e.preventDefault()
        // Future: implement multi-select
        toast.info('Select all not yet implemented')
      }

      // Arrow keys: Move selected component
      if (selectedComponentId && ['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(e.key)) {
        e.preventDefault()
        const component = components.find(c => c.id === selectedComponentId)
        if (component) {
          const step = e.shiftKey ? 10 : 1
          const updates: any = { position: { ...component.position } }
          
          switch (e.key) {
            case 'ArrowUp':
              updates.position.y -= step
              break
            case 'ArrowDown':
              updates.position.y += step
              break
            case 'ArrowLeft':
              updates.position.x -= step
              break
            case 'ArrowRight':
              updates.position.x += step
              break
          }
          
          useHMIStore.getState().updateComponent(selectedComponentId, updates)
        }
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [selectedComponentId, deleteComponent, duplicateComponent, undo, redo, canUndo, canRedo, saveDisplay, components])
}
