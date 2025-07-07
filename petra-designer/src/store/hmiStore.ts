import { create } from 'zustand'
import type { HMIComponent, HMIDisplay } from '@/types/hmi'

interface HMIStore {
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
  
  // Actions
  addComponent: (component: HMIComponent) => void
  updateComponent: (id: string, updates: Partial<HMIComponent>) => void
  deleteComponent: (id: string) => void
  selectComponent: (id: string) => void
  clearSelection: () => void
  
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

export const useHMIStore = create<HMIStore>((set, get) => ({
  currentDisplay: null,
  components: [],
  selectedComponentId: null,
  showGrid: true,
  snapToGrid: true,
  gridSize: 20,

  addComponent: (component) => {
    set((state) => ({
      components: [...state.components, component],
      selectedComponentId: component.id,
    }))
  },

  updateComponent: (id, updates) => {
    set((state) => ({
      components: state.components.map((c) =>
        c.id === id ? { ...c, ...updates } : c
      ),
    }))
  },

  deleteComponent: (id) => {
    set((state) => ({
      components: state.components.filter((c) => c.id !== id),
      selectedComponentId: state.selectedComponentId === id ? null : state.selectedComponentId,
    }))
  },

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
    })
  },

  loadDisplay: (display) => {
    set({
      currentDisplay: display,
      components: display.components || [],
      selectedComponentId: null,
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
    
    state.addComponent(newComponent)
  },

  groupComponents: (ids) => {
    // Implementation for grouping components
    // This would create a new group component containing the selected components
  },

  ungroupComponent: (id) => {
    // Implementation for ungrouping
  },

  alignComponents: (ids, alignment) => {
    const state = get()
    const components = state.components.filter((c) => ids.includes(c.id))
    if (components.length < 2) return

    let updates: { id: string; position: { x: number; y: number } }[] = []

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

    updates.forEach(({ id, position }) => {
      state.updateComponent(id, { position })
    })
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

      sorted.forEach((c, i) => {
        if (i > 0 && i < sorted.length - 1) {
          state.updateComponent(c.id, {
            position: { ...c.position, x: first.position.x + spacing * i },
          })
        }
      })
    } else {
      const sorted = [...components].sort((a, b) => a.position.y - b.position.y)
      const first = sorted[0]
      const last = sorted[sorted.length - 1]
      const totalHeight = last.position.y - first.position.y
      const spacing = totalHeight / (sorted.length - 1)

      sorted.forEach((c, i) => {
        if (i > 0 && i < sorted.length - 1) {
          state.updateComponent(c.id, {
            position: { ...c.position, y: first.position.y + spacing * i },
          })
        }
      })
    }
  },
}))
