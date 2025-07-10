// src/contexts/PetraContext.tsx

import { createContext, useContext, ReactNode } from 'react'
import { usePetraConnection } from '@/hooks/usePetraConnection'

interface PetraContextType {
  connected: boolean
  signals: Map<string, any>
  quality: Map<string, string>
  mqttData: Map<string, any>
  subscribeSignal: (signal: string) => void
  unsubscribeSignal: (signal: string) => void
  setSignalValue: (signal: string, value: any) => void
  subscribeMQTT: (topic: string) => void
  unsubscribeMQTT: (topic: string) => void
  publishMQTT: (topic: string, payload: any) => void
  reconnect: () => void
}

const PetraContext = createContext<PetraContextType | null>(null)

export function PetraProvider({ children }: { children: ReactNode }) {
  // Use the real connection with environment variable support
  const connection = usePetraConnection({
    url: import.meta.env.VITE_PETRA_WS_URL || 'ws://localhost:8080/ws',
    enableMQTT: true
  })

  // Map the connection interface to the context interface
  const contextValue: PetraContextType = {
    connected: connection.connected,
    signals: connection.signals,
    quality: connection.quality,
    mqttData: connection.mqttData,
    subscribeSignal: connection.subscribeSignal,
    unsubscribeSignal: connection.unsubscribeSignal,
    setSignalValue: connection.setSignalValue,
    subscribeMQTT: connection.subscribeMQTT,
    unsubscribeMQTT: connection.unsubscribeMQTT,
    publishMQTT: connection.publishMQTT,
    reconnect: connection.reconnect
  }

  return (
    <PetraContext.Provider value={contextValue}>
      {children}
    </PetraContext.Provider>
  )
}

export function usePetra() {
  const context = useContext(PetraContext)
  if (!context) {
    throw new Error('usePetra must be used within a PetraProvider')
  }
  return context
}

// Re-export hooks for convenience
export { usePetraSignal, useMQTTTopic } from '@/hooks/usePetraConnection'
