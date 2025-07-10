import { useState, useEffect, useCallback, useRef } from 'react'
import toast from 'react-hot-toast'

// Types
interface SignalUpdate {
  signal: string
  value: any
  quality?: string
  timestamp?: number
}

interface MQTTUpdate {
  topic: string
  payload: any
}

interface PetraMessage {
  type: string
  data?: any
  updates?: SignalUpdate[]
  error?: string
  timestamp?: number
}

interface UsePetraConnectionOptions {
  url?: string
  autoConnect?: boolean
  onConnect?: () => void
  onDisconnect?: () => void
  onError?: (error: any) => void
  enableMQTT?: boolean
  maxReconnectAttempts?: number
  reconnectDelay?: number
}

interface PerformanceMetrics {
  latency: number
  messageRate: number
  connectedAt?: number
  reconnectAttempts: number
}

export function usePetraConnection(options: UsePetraConnectionOptions = {}) {
  const {
    url = 'ws://localhost:8080/ws',
    autoConnect = true,
    onConnect,
    onDisconnect,
    onError,
    enableMQTT = true,
    maxReconnectAttempts = 5,
    reconnectDelay = 3000
  } = options

  // Connection state
  const [connected, setConnected] = useState(false)
  const [connectionState, setConnectionState] = useState<'disconnected' | 'connecting' | 'connected' | 'error'>('disconnected')
  const [lastError, setLastError] = useState<string | null>(null)
  
  // Data state
  const [signals, setSignals] = useState<Map<string, any>>(new Map())
  const [quality, setQuality] = useState<Map<string, string>>(new Map())
  const [mqttData, setMqttData] = useState<Map<string, any>>(new Map())
  
  // Performance metrics
  const [performance, setPerformance] = useState<PerformanceMetrics>({
    latency: 0,
    messageRate: 0,
    reconnectAttempts: 0
  })

  // Refs
  const wsRef = useRef<WebSocket | null>(null)
  const reconnectCountRef = useRef(0)
  const pingIntervalRef = useRef<NodeJS.Timeout>()
  const messageRateRef = useRef<number[]>([])
  const subscribedSignalsRef = useRef<Set<string>>(new Set())
  const subscribedTopicsRef = useRef<Set<string>>(new Set())

  // Helper functions
  const toBackendValue = (value: any) => {
    if (typeof value === 'boolean') return { Bool: value }
    if (typeof value === 'number') {
      return Number.isInteger(value) ? { Integer: value } : { Float: value }
    }
    if (typeof value === 'string') return { String: value }
    return value
  }

  const fromBackendValue = (value: any) => {
    if (value && typeof value === 'object') {
      if ('Bool' in value) return value.Bool
      if ('Integer' in value) return value.Integer
      if ('Float' in value) return value.Float
      if ('String' in value) return value.String
    }
    return value
  }

  const handleConnectionError = useCallback((error: any) => {
    if (onError) {
      onError(error)
    }
    
    // Set appropriate error message
    if (error.code === 'ECONNREFUSED') {
      setLastError('Connection refused. Is PETRA running?')
    } else if (error.code === 'ETIMEDOUT') {
      setLastError('Connection timeout. Check network settings.')
    } else {
      setLastError(error.message || 'Unknown connection error')
    }
    setConnectionState('error')
  }, [onError])

  const getConnectionDiagnostics = useCallback(() => {
    return {
      state: connectionState,
      connected,
      url,
      lastError,
      reconnectAttempts: reconnectCountRef.current,
      subscribedSignals: subscribedSignalsRef.current.size,
      subscribedTopics: subscribedTopicsRef.current.size
    }
  }, [connectionState, connected, url, lastError])

  const connect = useCallback(() => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) return

    console.log(`Connecting to PETRA at ${url}...`)
    
    try {
      wsRef.current = new WebSocket(url)
      setConnectionState('connecting')
      
      wsRef.current.onopen = () => {
        console.log('Connected to PETRA')
        setConnected(true)
        setConnectionState('connected')
        reconnectCountRef.current = 0
        setPerformance(prev => ({
          ...prev,
          connectedAt: Date.now(),
          reconnectAttempts: reconnectCountRef.current
        }))
        
        if (onConnect) {
          onConnect()
        }
        
        // Start ping interval to keep connection alive
        pingIntervalRef.current = setInterval(() => {
          if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
            wsRef.current.send(JSON.stringify({ type: 'ping', timestamp: Date.now() }))
          }
        }, 30000) // Ping every 30 seconds
        
        // Re-subscribe to all signals
        if (wsRef.current && subscribedSignalsRef.current.size > 0) {
          wsRef.current.send(JSON.stringify({
            type: 'subscribe_signals',
            signals: Array.from(subscribedSignalsRef.current)
          }))
        }
        
        // Re-subscribe to all MQTT topics
        if (wsRef.current && enableMQTT && subscribedTopicsRef.current.size > 0) {
          wsRef.current.send(JSON.stringify({
            type: 'subscribe_mqtt',
            topics: Array.from(subscribedTopicsRef.current)
          }))
        }
      }

      wsRef.current.onmessage = (event) => {
        try {
          messageRateRef.current.push(Date.now())
          const message: PetraMessage = JSON.parse(event.data)
          
          switch (message.type) {
            case 'signal_update':
              const update = message.data as SignalUpdate
              setSignals(prev => new Map(prev).set(update.signal, fromBackendValue(update.value)))
              
              if (update.quality) {
                setQuality(prev => new Map(prev).set(update.signal, update.quality!))
              }
              break
              
            case 'mqtt_update':
              const mqttUpdate = message.data as MQTTUpdate
              setMqttData(prev => new Map(prev).set(mqttUpdate.topic, fromBackendValue(mqttUpdate.payload)))
              
              // Also update signals if MQTT topic maps to a signal
              if (mqttUpdate.topic.startsWith('petra/signals/')) {
                const signalName = mqttUpdate.topic.replace('petra/signals/', '')
                setSignals(prev => new Map(prev).set(signalName, mqttUpdate.payload))
              }
              break
              
            case 'batch_update':
              const updates = message.updates || []
              setSignals(prev => {
                const newMap = new Map(prev)
                updates.forEach(update => {
                  newMap.set(update.signal, fromBackendValue(update.value))
                  if (update.quality) {
                    setQuality(prev => new Map(prev).set(update.signal, update.quality!))
                  }
                })
                return newMap
              })
              break

            case 'pong':
              const start = message.timestamp ?? Date.now()
              setPerformance(prev => ({
                ...prev,
                latency: Date.now() - start
              }))
              break
              
            case 'error':
              console.error('PETRA error:', message.error)
              // Only show error toasts, not disconnect messages
              if (message.error && !message.error.includes('disconnect')) {
                toast.error(`PETRA: ${message.error}`)
              }
              break
          }
        } catch (error) {
          console.error('Failed to parse WebSocket message:', error)
        }
      }

      wsRef.current.onerror = (error) => {
        handleConnectionError(error)
      }

      wsRef.current.onclose = () => {
        setConnected(false)
        setConnectionState('disconnected')
        wsRef.current = null

        setPerformance(prev => ({
          ...prev,
          reconnectAttempts: reconnectCountRef.current
        }))
        
        // Clear ping interval
        if (pingIntervalRef.current) {
          clearInterval(pingIntervalRef.current)
        }
        
        if (onDisconnect) {
          onDisconnect()
        }
        
        // Attempt reconnection silently
        if (reconnectCountRef.current < maxReconnectAttempts) {
          reconnectCountRef.current++
          
          // Silent reconnection - no toast notification
          setTimeout(() => {
            connect()
          }, reconnectDelay)
        }
      }
    } catch (error) {
      handleConnectionError(error)
    }
  }, [url, onConnect, onDisconnect, handleConnectionError, enableMQTT, maxReconnectAttempts, reconnectDelay])

  const disconnect = useCallback(() => {
    if (wsRef.current) {
      wsRef.current.close()
      wsRef.current = null
    }
    setConnected(false)
    setConnectionState('disconnected')
    
    if (pingIntervalRef.current) {
      clearInterval(pingIntervalRef.current)
    }
  }, [])

  const subscribeToSignal = useCallback((signalName: string) => {
    subscribedSignalsRef.current.add(signalName)
    
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        type: 'subscribe_signal',
        signal: signalName
      }))
    }
  }, [])

  const unsubscribeFromSignal = useCallback((signalName: string) => {
    subscribedSignalsRef.current.delete(signalName)
    
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        type: 'unsubscribe_signal',
        signal: signalName
      }))
    }
  }, [])

  const subscribeToMQTT = useCallback((topic: string) => {
    if (!enableMQTT) return
    
    subscribedTopicsRef.current.add(topic)
    
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        type: 'subscribe_mqtt',
        topic: topic
      }))
    }
  }, [enableMQTT])

  const unsubscribeFromMQTT = useCallback((topic: string) => {
    if (!enableMQTT) return
    
    subscribedTopicsRef.current.delete(topic)
    
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        type: 'unsubscribe_mqtt',
        topic: topic
      }))
    }
  }, [enableMQTT])

  const setSignalValue = useCallback((signalName: string, value: any) => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        type: 'set_signal',
        signal: signalName,
        value: toBackendValue(value)
      }))
    } else {
      toast.error('Not connected to PETRA')
    }
  }, [])

  const batchSetSignals = useCallback((updates: Record<string, any>) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(
        JSON.stringify({
          type: 'batch_set',
          updates: Object.entries(updates).map(([signal, value]) => ({
            signal,
            value: toBackendValue(value),
            timestamp: Date.now()
          }))
        })
      )
    }
  }, [])

  const getSignalMetadata = useCallback((signal: string) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(
        JSON.stringify({
          type: 'get_metadata',
          signal
        })
      )
    }
  }, [])

  const subscribeSignalGroup = useCallback((group: string) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(
        JSON.stringify({
          type: 'subscribe_group',
          group
        })
      )
    }
  }, [])

  const publishMQTT = useCallback((topic: string, payload: any) => {
    if (!enableMQTT) return
    
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        type: 'publish_mqtt',
        topic: topic,
        payload: toBackendValue(payload)
      }))
    }
  }, [enableMQTT])

  // Calculate message rate
  useEffect(() => {
    const interval = setInterval(() => {
      const now = Date.now()
      const recentMessages = messageRateRef.current.filter(t => now - t < 1000)
      messageRateRef.current = recentMessages
      
      setPerformance(prev => ({
        ...prev,
        messageRate: recentMessages.length
      }))
    }, 1000)
    
    return () => clearInterval(interval)
  }, [])

  // Auto-connect on mount
  useEffect(() => {
    if (autoConnect) {
      connect()
    }
    
    return () => {
      disconnect()
    }
  }, [autoConnect, connect, disconnect])

  return {
    // Connection state
    connected,
    connectionState,
    lastError,
    
    // Data
    signals,
    quality,
    mqttData,
    
    // Performance
    performance,
    
    // Methods
    connect,
    disconnect,
    subscribeToSignal,
    unsubscribeFromSignal,
    subscribeToMQTT,
    unsubscribeFromMQTT,
    setSignalValue,
    batchSetSignals,
    publishMQTT,
    getSignalMetadata,
    subscribeSignalGroup,
    getConnectionDiagnostics
  }
}
