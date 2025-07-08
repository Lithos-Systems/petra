// src/hooks/useConfiguration.ts
import { useState } from 'react'
import { toast } from 'react-hot-toast'

interface Config {
  [key: string]: any
}

export function useConfiguration() {
  const [config, setConfig] = useState<Config | null>(null)
  const [isDirty, setIsDirty] = useState(false)

  const loadConfig = async () => {
    const response = await fetch('/api/config')
    const data = await response.json()
    setConfig(data)
  }

  const saveConfig = async (newConfig: Config) => {
    const response = await fetch('/api/config', {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(newConfig)
    })

    if (response.ok) {
      setConfig(newConfig)
      setIsDirty(false)
      toast.success('Configuration saved')
    }
  }

  const reloadConfig = async () => {
    await fetch('/api/config/reload', { method: 'POST' })
  }

  return { config, isDirty, loadConfig, saveConfig, reloadConfig }
}
