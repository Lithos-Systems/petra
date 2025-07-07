// src/hooks/usePetraConnection.ts

import { useEffect, useState, useCallback, useRef } from 'react'
import { toast } from 'react-hot-toast'

interface SignalUpdate {
  signal: string
  value: any
  timestamp: number
  quality?: 'good' | 'bad' | 'uncertain'
}

interface PetraConnectionOptions {
  url?: string
  reconnectInterval?: number
  maxReconnectAttempts?: number
}

export function usePetraConnection(options: PetraConnectionOptions = {}) {
  const {
    url = 'ws://localhost:8080/ws',
    reconnectInterval = 5000,
    maxReconnectAttempts = 10
  } = options

  const [connected, setConnected] = useState(false)
  const [signals, setSignals] = useState<Map<string, any>>(new Map())
  const [quality, setQuality] = useState<Map<string, string>>(new Map())
  
  const wsRef = useRef<WebSocket | null>(null)
  const reconnectCountRef = useRef(0)
  const reconnectTimeoutRef = useRef<NodeJS.Timeout>()
  const subscribedSignalsRef = useRef<Set<string>>(new Set())

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) return

    try {
      wsRef.current = new WebSocket(url)
      
      wsRef.current.onopen = () => {
        setConnected(true)
        reconnectCountRef.current = 0
        toast.success('Connected to PETRA')
        
        // Re-subscribe to all signals
        subscribedSignalsRef.current.forEach(signal => {
          wsRef.current?.send(JSON.stringify({
            action: 'subscribe',
            signals: [signal]
          }))
        })
      }

      wsRef.current.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data)
          
          if (data.type === 'signal_update') {
            const update = data as SignalUpdate
            setSignals(prev => new Map(prev).set(update.signal, update.value))
            
            if (update.quality) {
              setQuality(prev => new Map(prev).set(update.signal, update.quality!))
            }
          } else if (data.type === 'batch_update') {
            // Handle batch updates for efficiency
            const updates = data.updates as SignalUpdate[]
            setSignals(prev => {
              const newMap = new Map(prev)
              updates.forEach(update => {
                newMap.set(update.signal, update.value)
              })
              return newMap
            })
          }
        } catch (error) {
          console.error('Failed to parse WebSocket message:', error)
        }
      }

      wsRef.current.onerror = (error) => {
        console.error('WebSocket error:', error)
        toast.error('Connection error')
      }

      wsRef.current.onclose = () => {
        setConnected(false)
        wsRef.current = null
        
        // Attempt reconnection
        if (reconnectCountRef.current < maxReconnectAttempts) {
          reconnectCountRef.current++
          toast.error(`Disconnected. Reconnecting... (${reconnectCountRef.current}/${maxReconnectAttempts})`)
          
          reconnectTimeoutRef.current = setTimeout(() => {
            connect()
          }, reconnectInterval)
        } else {
          toast.error('Failed to connect to PETRA. Please check the connection.')
        }
      }
    } catch (error) {
      console.error('Failed to create WebSocket:', error)
      toast.error('Failed to connect to PETRA')
    }
  }, [url, reconnectInterval, maxReconnectAttempts])

  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current)
    }
    
    if (wsRef.current) {
      wsRef.current.close()
      wsRef.current = null
    }
    
    setConnected(false)
  }, [])

  const subscribe = useCallback((signalName: string) => {
    subscribedSignalsRef.current.add(signalName)
    
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        action: 'subscribe',
        signals: [signalName]
      }))
    }
  }, [])

  const unsubscribe = useCallback((signalName: string) => {
    subscribedSignalsRef.current.delete(signalName)
    
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        action: 'unsubscribe',
        signals: [signalName]
      }))
    }
  }, [])

  const setSignalValue = useCallback((signalName: string, value: any) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        action: 'set_signal',
        signal: signalName,
        value: value
      }))
    } else {
      toast.error('Not connected to PETRA')
    }
  }, [])

  // Auto-connect on mount
  useEffect(() => {
    connect()
    
    return () => {
      disconnect()
    }
  }, [connect, disconnect])

  return {
    connected,
    signals,
    quality,
    subscribe,
    unsubscribe,
    setSignalValue,
    reconnect: connect
  }
}

// Hook to use a specific signal
export function usePetraSignal(signalName: string, defaultValue: any = null) {
  const { signals, subscribe, unsubscribe } = usePetraConnection()
  
  useEffect(() => {
    subscribe(signalName)
    
    return () => {
      unsubscribe(signalName)
    }
  }, [signalName, subscribe, unsubscribe])
  
  return signals.get(signalName) ?? defaultValue
}
