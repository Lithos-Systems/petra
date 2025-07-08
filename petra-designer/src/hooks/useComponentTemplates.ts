// src/hooks/useComponentTemplates.ts
import type { ComponentTemplate } from '@/types/hmi'

export function useComponentTemplates() {
  const templates: ComponentTemplate[] = [
    {
      name: 'Motor Control',
      description: 'Standard motor with start/stop controls',
      component: {
        type: 'group'
      }
    },
    {
      name: 'PID Loop',
      description: 'PID controller with setpoint and PV display',
      component: {
        type: 'group'
      }
    }
  ]

  const instantiateTemplate = (name: string) => {
    return templates.find(t => t.name === name) || null
  }

  return { templates, instantiateTemplate }
}
