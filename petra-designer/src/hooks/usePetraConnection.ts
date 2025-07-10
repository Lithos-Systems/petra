import { useState, useEffect, useCallback, useRef } from 'react'
import toast from 'react-hot-toast'

// Types
export interface SignalUpdate {
  signal: string
  value: any
  quality?: string
  timestamp?: number
}

export interface MQTTUpdate {
  topic: string
  payload: any
}

export interface AlarmUpdate {
  id: string
  message: string
  severity: 'low' | 'medium' | 'high' | 'critical'
  timestamp: number
  acknowledged: boolean
  source?: string
}

export interface HistoryData {
  timestamp: number
  value: any
  quality?: string
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
  const pingIntervalRef = useRef<number>()
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
    } else if (error.type === 'error') {
      setLastError('WebSocket connection failed')
    } else {
      setLastError(error.message || 'Connection error')
    }
    
    setConnectionState('error')
  }, [onError])

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      return // Already connected
    }

    setConnectionState('connecting')
    setLastError(null)

    try {
      wsRef.current = new WebSocket(url)

      wsRef.current.onopen = () => {
        setConnected(true)
        setConnectionState('connected')
        setLastError(null)
        reconnectCountRef.current = 0

        setPerformance(prev => ({
          ...prev,
          connectedAt: Date.now(),
          reconnectAttempts: 0
        }))

        // Re-subscribe to all signals and topics
        subscribedSignalsRef.current.forEach(signal => {
          wsRef.current?.send(JSON.stringify({
            type: 'subscribe_signal',
            signal
          }))
        })

        if (enableMQTT) {
          subscribedTopicsRef.current.forEach(topic => {
            wsRef.current?.send(JSON.stringify({
              type: 'subscribe_mqtt',
              topic
            }))
          })
        }

        // Start ping interval for latency monitoring
        pingIntervalRef.current = setInterval(() => {
          if (wsRef.current?.readyState === WebSocket.OPEN) {
            wsRef.current.send(JSON.stringify({
              type: 'ping',
              timestamp: Date.now()
            }))
          }
        }, 5000)

        if (onConnect) {
          onConnect()
        }
      }

      wsRef.current.onmessage = (event) => {
        try {
          const message: PetraMessage = JSON.parse(event.data)
          
          // Track message rate
          messageRateRef.current.push(Date.now())
          
          switch (message.type) {
            case 'signal_update':
              const signalUpdate = message.data as SignalUpdate
              setSignals(prev => new Map(prev).set(signalUpdate.signal, fromBackendValue(signalUpdate.value)))
              if (signalUpdate.quality) {
                setQuality(prev => new Map(prev).set(signalUpdate.signal, signalUpdate.quality!))
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

  const getConnectionDiagnostics = useCallback(() => {
    return {
      connected,
      connectionState,
      lastError,
      performance,
      subscribedSignals: Array.from(subscribedSignalsRef.current),
      subscribedTopics: Array.from(subscribedTopicsRef.current),
      signalCount: signals.size,
      mqttTopicCount: mqttData.size
    }
  }, [connected, connectionState, lastError, performance, signals.size, mqttData.size])

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
    subscribeSignal: subscribeToSignal,
    unsubscribeSignal: unsubscribeFromSignal,
    subscribeToSignal,
    unsubscribeFromSignal,
    subscribeToMQTT,
    unsubscribeFromMQTT,
    subscribeMQTT: subscribeToMQTT,
    unsubscribeMQTT: unsubscribeFromMQTT,
    setSignalValue,
    batchSetSignals,
    publishMQTT,
    getSignalMetadata,
    subscribeSignalGroup,
    getConnectionDiagnostics,
    reconnect: connect
  }
}

// Individual signal hook
export function usePetraSignal(signalName: string) {
  const [value, setValue] = useState<any>(null)
  const [quality, setQuality] = useState<string>('good')
  const connection = usePetraConnection()

  useEffect(() => {
    if (!signalName) return

    // Subscribe to the signal
    connection.subscribeToSignal(signalName)

    // Set initial value if it exists
    const currentValue = connection.signals.get(signalName)
    if (currentValue !== undefined) {
      setValue(connection.signals.get(signalName))
    }

    const currentQuality = connection.quality.get(signalName)
    if (currentQuality) {
      setQuality(currentQuality)
    }

    return () => {
      connection.unsubscribeFromSignal(signalName)
    }
  }, [signalName, connection])

  // Update value when signal changes
  useEffect(() => {
    const signalValue = connection.signals.get(signalName)
    if (signalValue !== undefined) {
      setValue(signalValue)
    }

    const signalQuality = connection.quality.get(signalName)
    if (signalQuality) {
      setQuality(signalQuality)
    }
  }, [connection.signals, connection.quality, signalName])

  const updateValue = useCallback((newValue: any) => {
    connection.setSignalValue(signalName, newValue)
  }, [connection, signalName])

  return {
    value,
    quality,
    updateValue,
    connected: connection.connected
  }
}

// Individual MQTT topic hook
export function useMQTTTopic(topic: string) {
  const [value, setValue] = useState<any>(null)
  const connection = usePetraConnection()

  useEffect(() => {
    if (!topic) return

    // Subscribe to the MQTT topic
    connection.subscribeToMQTT(topic)

    // Set initial value if it exists
    const currentValue = connection.mqttData.get(topic)
    if (currentValue !== undefined) {
      setValue(currentValue)
    }

    return () => {
      connection.unsubscribeFromMQTT(topic)
    }
  }, [topic, connection])

  // Update value when topic data changes
  useEffect(() => {
    const topicValue = connection.mqttData.get(topic)
    if (topicValue !== undefined) {
      setValue(topicValue)
    }
  }, [connection.mqttData, topic])

  const publish = useCallback((payload: any) => {
    connection.publishMQTT(topic, payload)
  }, [connection, topic])

  return {
    value,
    publish,
    connected: connection.connected
  }
}
