// src/hooks/useComponentTemplates.ts
import type { ComponentTemplate } from '@/types/hmi'

export function useComponentTemplates() {
  const templates: ComponentTemplate[] = [
    {
      name: 'Motor Control',
      description: 'Standard motor with start/stop controls',
      component: {
        type: 'group',
        children: ['motor', 'button', 'indicator'] as any
      }
    },
    {
      name: 'PID Loop',
      description: 'PID controller with setpoint and PV display',
      component: {
        type: 'group',
        children: ['gauge', 'trend', 'text'] as any
      }
    }
  ]

  const instantiateTemplate = (name: string) => {
    return templates.find(t => t.name === name) || null
  }

  return { templates, instantiateTemplate }
}
