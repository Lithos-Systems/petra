// src/contexts/PetraContext.tsx

import React, { createContext, useContext, ReactNode, useState, useEffect } from 'react'

interface PetraContextType {
  connected: boolean
  signals: Map<string, any>
  quality: Map<string, string>
  subscribe: (signal: string) => void
  unsubscribe: (signal: string) => void
  setSignalValue: (signal: string, value: any) => void
  reconnect: () => void
}

const PetraContext = createContext<PetraContextType | null>(null)

export function PetraProvider({ children }: { children: ReactNode }) {
  const [connected] = useState(false) // Mock: always disconnected for now
  const [signals] = useState(new Map<string, any>())
  const [quality] = useState(new Map<string, string>())

  // Mock implementation - in production, use usePetraConnection hook
  const mockConnection: PetraContextType = {
    connected,
    signals,
    quality,
    subscribe: (signal: string) => {
      console.log('Mock subscribe:', signal)
    },
    unsubscribe: (signal: string) => {
      console.log('Mock unsubscribe:', signal)
    },
    setSignalValue: (signal: string, value: any) => {
      console.log('Mock set signal:', signal, value)
    },
    reconnect: () => {
      console.log('Mock reconnect')
    }
  }

  return (
    <PetraContext.Provider value={mockConnection}>
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

// Hook to use a specific signal
export function usePetraSignal(signalName: string, defaultValue: any = null) {
  const { signals, subscribe, unsubscribe } = usePetra()
  
  useEffect(() => {
    subscribe(signalName)
    
    return () => {
      unsubscribe(signalName)
    }
  }, [signalName, subscribe, unsubscribe])
  
  return signals.get(signalName) ?? defaultValue
}
