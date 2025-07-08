// src/store/hmiStore.ts

import { create } from 'zustand'
import type { HMIComponent, HMIDisplay } from '@/types/hmi'

interface HMIState {
  // Current display
  currentDisplay: HMIDisplay | null
  
  // Components in the current display
  components: HMIComponent[]
  
  // Selection
  selectedComponentId: string | null
  
  // Editor settings
  showGrid: boolean
  snapToGrid: boolean
  gridSize: number
  
  // History for undo/redo
  history: Array<{ components: HMIComponent[] }>
  historyIndex: number
}

interface HMIActions {
  // Actions
  addComponent: (component: HMIComponent) => void
  updateComponent: (id: string, updates: Partial<HMIComponent>) => void
  deleteComponent: (id: string) => void
  selectComponent: (id: string) => void
  clearSelection: () => void
  
  // History management
  undo: () => void
  redo: () => void
  canUndo: () => boolean
  canRedo: () => boolean
  
  // Display management
  createDisplay: (name: string, size: { width: number; height: number }) => void
  loadDisplay: (display: HMIDisplay) => void
  saveDisplay: () => HMIDisplay | null
  
  // Editor settings
  toggleGrid: () => void
  toggleSnapToGrid: () => void
  setGridSize: (size: number) => void
  
  // Bulk operations
  duplicateComponent: (id: string) => void
  groupComponents: (ids: string[]) => void
  ungroupComponent: (id: string) => void
  alignComponents: (ids: string[], alignment: 'left' | 'right' | 'top' | 'bottom' | 'center-h' | 'center-v') => void
  distributeComponents: (ids: string[], direction: 'horizontal' | 'vertical') => void
}

type HMIStore = HMIState & HMIActions

// Helper function to save to history
const saveToHistory = (get: () => HMIStore, set: (state: Partial<HMIStore>) => void) => {
  const state = get()
  const newHistory = state.history.slice(0, state.historyIndex + 1)
  newHistory.push({ components: [...state.components] })
  
  // Limit history to 50 items
  if (newHistory.length > 50) {
    newHistory.shift()
  }
  
  set({
    history: newHistory,
    historyIndex: newHistory.length - 1,
  })
}

export const useHMIStore = create<HMIStore>((set, get) => ({
  // State
  currentDisplay: null,
  components: [],
  selectedComponentId: null,
  showGrid: true,
  snapToGrid: true,
  gridSize: 20,
  history: [{ components: [] }],
  historyIndex: 0,

  // Actions
  addComponent: (component) => {
    set((state) => ({
      components: [...state.components, component],
      selectedComponentId: component.id,
    }))
    saveToHistory(get, set)
  },

  updateComponent: (id, updates) => {
    set((state) => ({
      components: state.components.map((c) =>
        c.id === id ? { ...c, ...updates } : c
      ),
    }))
    saveToHistory(get, set)
  },

  deleteComponent: (id) => {
    set((state) => ({
      components: state.components.filter((c) => c.id !== id),
      selectedComponentId: state.selectedComponentId === id ? null : state.selectedComponentId,
    }))
    saveToHistory(get, set)
  },

  undo: () => {
    const state = get()
    if (state.historyIndex > 0) {
      const newIndex = state.historyIndex - 1
      set({
        components: [...state.history[newIndex].components],
        historyIndex: newIndex,
        selectedComponentId: null,
      })
    }
  },

  redo: () => {
    const state = get()
    if (state.historyIndex < state.history.length - 1) {
      const newIndex = state.historyIndex + 1
      set({
        components: [...state.history[newIndex].components],
        historyIndex: newIndex,
        selectedComponentId: null,
      })
    }
  },

  canUndo: () => get().historyIndex > 0,
  canRedo: () => get().historyIndex < get().history.length - 1,

  selectComponent: (id) => {
    set({ selectedComponentId: id })
  },

  clearSelection: () => {
    set({ selectedComponentId: null })
  },

  createDisplay: (name, size) => {
    const display: HMIDisplay = {
      id: Date.now().toString(),
      name,
      size,
      components: [],
      background: '#f0f0f0',
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    }
    set({
      currentDisplay: display,
      components: [],
      selectedComponentId: null,
      history: [{ components: [] }],
      historyIndex: 0,
    })
  },

  loadDisplay: (display) => {
    set({
      currentDisplay: display,
      components: display.components || [],
      selectedComponentId: null,
      history: [{ components: display.components || [] }],
      historyIndex: 0,
    })
  },

  saveDisplay: () => {
    const state = get()
    if (!state.currentDisplay) return null
    
    return {
      ...state.currentDisplay,
      components: state.components,
      updatedAt: new Date().toISOString(),
    }
  },

  toggleGrid: () => {
    set((state) => ({ showGrid: !state.showGrid }))
  },

  toggleSnapToGrid: () => {
    set((state) => ({ snapToGrid: !state.snapToGrid }))
  },

  setGridSize: (size) => {
    set({ gridSize: size })
  },

  duplicateComponent: (id) => {
    const state = get()
    const component = state.components.find((c) => c.id === id)
    if (!component) return

    const newComponent: HMIComponent = {
      ...component,
      id: Date.now().toString(),
      position: {
        x: component.position.x + 20,
        y: component.position.y + 20,
      },
    }
    
    set((state) => ({
      components: [...state.components, newComponent],
      selectedComponentId: newComponent.id,
    }))
    saveToHistory(get, set)
  },

  groupComponents: (ids) => {
    // Implementation for grouping components
    // This would create a new group component containing the selected components
    console.log('Grouping components:', ids)
  },

  ungroupComponent: (id) => {
    // Implementation for ungrouping
    console.log('Ungrouping component:', id)
  },

  alignComponents: (ids, alignment) => {
    const state = get()
    const components = state.components.filter((c) => ids.includes(c.id))
    if (components.length < 2) return

    let updates: Array<{ id: string; position: { x: number; y: number } }> = []

    switch (alignment) {
      case 'left':
        const minX = Math.min(...components.map((c) => c.position.x))
        updates = components.map((c) => ({
          id: c.id,
          position: { ...c.position, x: minX },
        }))
        break
      case 'right':
        const maxX = Math.max(...components.map((c) => c.position.x + c.size.width))
        updates = components.map((c) => ({
          id: c.id,
          position: { ...c.position, x: maxX - c.size.width },
        }))
        break
      case 'top':
        const minY = Math.min(...components.map((c) => c.position.y))
        updates = components.map((c) => ({
          id: c.id,
          position: { ...c.position, y: minY },
        }))
        break
      case 'bottom':
        const maxY = Math.max(...components.map((c) => c.position.y + c.size.height))
        updates = components.map((c) => ({
          id: c.id,
          position: { ...c.position, y: maxY - c.size.height },
        }))
        break
      case 'center-h':
        const avgX = components.reduce((sum, c) => sum + c.position.x + c.size.width / 2, 0) / components.length
        updates = components.map((c) => ({
          id: c.id,
          position: { ...c.position, x: avgX - c.size.width / 2 },
        }))
        break
      case 'center-v':
        const avgY = components.reduce((sum, c) => sum + c.position.y + c.size.height / 2, 0) / components.length
        updates = components.map((c) => ({
          id: c.id,
          position: { ...c.position, y: avgY - c.size.height / 2 },
        }))
        break
    }

    set((state) => ({
      components: state.components.map((c) => {
        const update = updates.find((u) => u.id === c.id)
        return update ? { ...c, position: update.position } : c
      }),
    }))
    saveToHistory(get, set)
  },

  distributeComponents: (ids, direction) => {
    const state = get()
    const components = state.components.filter((c) => ids.includes(c.id))
    if (components.length < 3) return

    if (direction === 'horizontal') {
      const sorted = [...components].sort((a, b) => a.position.x - b.position.x)
      const first = sorted[0]
      const last = sorted[sorted.length - 1]
      const totalWidth = last.position.x - first.position.x
      const spacing = totalWidth / (sorted.length - 1)

      const updates = sorted.map((c, i) => ({
        id: c.id,
        position: {
          ...c.position,
          x: first.position.x + spacing * i,
        },
      }))

      set((state) => ({
        components: state.components.map((c) => {
          const update = updates.find((u) => u.id === c.id)
          return update ? { ...c, position: update.position } : c
        }),
      }))
    } else {
      const sorted = [...components].sort((a, b) => a.position.y - b.position.y)
      const first = sorted[0]
      const last = sorted[sorted.length - 1]
      const totalHeight = last.position.y - first.position.y
      const spacing = totalHeight / (sorted.length - 1)

      const updates = sorted.map((c, i) => ({
        id: c.id,
        position: {
          ...c.position,
          y: first.position.y + spacing * i,
        },
      }))

      set((state) => ({
        components: state.components.map((c) => {
          const update = updates.find((u) => u.id === c.id)
          return update ? { ...c, position: update.position } : c
        }),
      }))
    }
    saveToHistory(get, set)
  },
}))
