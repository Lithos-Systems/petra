// src/hooks/useProtocols.ts
import { useState } from 'react'

export interface ProtocolStatus {
  name: string
  type: 'modbus' | 's7' | 'opcua' | 'mqtt'
  connected: boolean
  lastError?: string
  statistics: {
    messagesIn: number
    messagesOut: number
    errors: number
    lastUpdate: number
  }
}

export function useProtocols() {
  const [protocols] = useState<Map<string, ProtocolStatus>>(new Map())

  const restartProtocol = (name: string) => {
    // TODO: implement restart logic
  }

  const getProtocolDiagnostics = (name: string) => protocols.get(name)

  return { protocols, restartProtocol, getProtocolDiagnostics }
}
