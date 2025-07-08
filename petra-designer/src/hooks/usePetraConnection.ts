// src/hooks/usePetraConnection.ts

import { useEffect, useState, useCallback, useRef } from 'react'
import { toast } from 'react-hot-toast'

export interface Value {
  type: 'Bool' | 'Integer' | 'Float'
  value: boolean | number
}

/**
 * Convert a backend value (which may be a plain primitive) into an adjacently
 * tagged enum representation.
 */
const toBackendValue = (value: any): Value => {
  if (typeof value === 'boolean') {
    return { type: 'Bool', value }
  }
  if (Number.isInteger(value)) {
    return { type: 'Integer', value }
  }
  if (typeof value === 'number') {
    return { type: 'Float', value }
  }
  // Assume value is already in the correct shape
  return value as Value
}

/**
 * Convert a value received from the backend to a plain primitive. The backend
 * may send either adjacently tagged values or plain primitives depending on the
 * version. This helper normalises the format for the rest of the frontend.
 */
const fromBackendValue = (value: any): any => {
  if (value && typeof value === 'object' && 'type' in value && 'value' in value) {
    return (value as Value).value
  }
  return value
}

export interface SignalUpdate {
  type: 'signal_update'
  signal: string
  value: any
  timestamp: number
  quality?: 'good' | 'bad' | 'uncertain'
  source?: 'mqtt' | 'internal' | 's7' | 'modbus'
}

export interface MQTTUpdate {
  type: 'mqtt_update'
  topic: string
  payload: any
  timestamp: number
}

export interface PetraMessage {
  type:
    | 'signal_update'
    | 'mqtt_update'
    | 'batch_update'
    | 'error'
    | 'connected'
    | 'alarm'
    | 'history_data'
    | 'system_status'
    | 'block_status'
    | 'protocol_status'
    | 'config_change'
    | 'pong'
  data?: any
  updates?: SignalUpdate[]
  error?: string
  timestamp?: number
}

export interface AlarmUpdate {
  type: 'alarm'
  alarm_id: string
  signal: string
  severity: 'critical' | 'warning' | 'info'
  message: string
  timestamp: number
  acknowledged: boolean
}

export interface HistoryData {
  signal: string
  data: Array<{
    timestamp: number
    value: any
    quality?: string
  }>
}

interface PetraConnectionOptions {
  url?: string
  reconnectInterval?: number
  maxReconnectAttempts?: number
  enableMQTT?: boolean
  enableHistory?: boolean
  onConnect?: () => void
  onDisconnect?: () => void
  onError?: (error: any) => void
}

export function usePetraConnection({
  url = (typeof window !== 'undefined' && window.location.hostname === 'localhost')
    ? 'ws://localhost:8080/ws'
    : `wss://${window.location.host}/ws`,
  reconnectInterval = 5000,
  maxReconnectAttempts = 10,
  enableMQTT = false,
  enableHistory: _enableHistory = false,
  onConnect: _onConnect,
  onDisconnect: _onDisconnect,
  onError: _onError,
}: PetraConnectionOptions = {}) {

  const [connected, setConnected] = useState(false)
  const [connectionState, setConnectionState] = useState<
    'disconnected' | 'connecting' | 'connected' | 'error'
  >('disconnected')
  const [lastError, setLastError] = useState<string | null>(null)
  const [signals, setSignals] = useState<Map<string, any>>(new Map())
  const [quality, setQuality] = useState<Map<string, string>>(new Map())
  const [mqttData, setMqttData] = useState<Map<string, any>>(new Map())
  const [performance, setPerformance] = useState({
    latency: 0,
    messageRate: 0,
    queueSize: 0
  })
  
  const wsRef = useRef<WebSocket | null>(null)
  const reconnectCountRef = useRef(0)
  const reconnectTimeoutRef = useRef<number>()
  const subscribedSignalsRef = useRef<Set<string>>(new Set())
  const subscribedTopicsRef = useRef<Set<string>>(new Set())
  const pingIntervalRef = useRef<number>()
  const messageRateRef = useRef<number[]>([])

  const handleConnectionError = useCallback((error: any) => {
    console.error('Connection error:', error)
    if (error.code === 'ECONNREFUSED') {
      setLastError('Cannot connect to PETRA server. Is it running?')
    } else if (error.code === 'ETIMEDOUT') {
      setLastError('Connection timeout. Check network settings.')
    } else {
      setLastError(error.message || 'Unknown connection error')
    }
    setConnectionState('error')
  }, [])

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
        toast.success('Connected to PETRA')
        
        // Start ping interval to keep connection alive
        pingIntervalRef.current = setInterval(() => {
          if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
            wsRef.current.send(JSON.stringify({ type: 'ping' }))
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

            case 'alarm':
              // handle alarm update
              break

            case 'history_data':
              // handle history data
              break

            case 'system_status':
              // handle system status
              break

            case 'block_status':
              // handle block status
              break

            case 'protocol_status':
              // handle protocol status
              break

            case 'config_change':
              // config changed
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
              toast.error(`PETRA: ${message.error}`)
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
        
        // Clear ping interval
        if (pingIntervalRef.current) {
          clearInterval(pingIntervalRef.current)
        }
        
        // Attempt reconnection
        if (reconnectCountRef.current < maxReconnectAttempts) {
          reconnectCountRef.current++
          const attemptMsg = `Disconnected. Reconnecting... (${reconnectCountRef.current}/${maxReconnectAttempts})`
          console.log(attemptMsg)
          toast.error(attemptMsg)
          
          reconnectTimeoutRef.current = setTimeout(() => {
            connect()
          }, reconnectInterval)
        } else {
          setLastError('Failed to connect to PETRA. Please check the connection.')
          setConnectionState('error')
          toast.error('Failed to connect to PETRA. Please check the connection.')
        }
      }
    } catch (error) {
      console.error('Failed to create WebSocket:', error)
      handleConnectionError(error)
    }
  }, [url, reconnectInterval, maxReconnectAttempts, enableMQTT])

  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current)
    }
    
    if (pingIntervalRef.current) {
      clearInterval(pingIntervalRef.current)
    }
    
    if (wsRef.current) {
      wsRef.current.close()
      wsRef.current = null
    }
    
    setConnected(false)
  }, [])

  const subscribeSignal = useCallback((signalName: string) => {
    subscribedSignalsRef.current.add(signalName)
    
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        type: 'subscribe_signal',
        signal: signalName
      }))
    }
  }, [])

  const unsubscribeSignal = useCallback((signalName: string) => {
    subscribedSignalsRef.current.delete(signalName)
    
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        type: 'unsubscribe_signal',
        signal: signalName
      }))
    }
  }, [])

  const subscribeMQTT = useCallback((topic: string) => {
    if (!enableMQTT) return
    
    subscribedTopicsRef.current.add(topic)
    
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        type: 'subscribe_mqtt',
        topic: topic
      }))
    }
  }, [enableMQTT])

  const unsubscribeMQTT = useCallback((topic: string) => {
    subscribedTopicsRef.current.delete(topic)
    
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        type: 'unsubscribe_mqtt',
        topic: topic
      }))
    }
  }, [])

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
        payload: payload
      }))
    } else {
      toast.error('Not connected to PETRA')
    }
  }, [enableMQTT])

  const measureLatency = useCallback(() => {
    const start = Date.now()
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(
        JSON.stringify({
          type: 'ping',
          timestamp: start
        })
      )
    }
  }, [])

  useEffect(() => {
    const interval = setInterval(() => {
      const rate = messageRateRef.current.length
      setPerformance(prev => ({ ...prev, messageRate: rate }))
      messageRateRef.current = []
    }, 1000)
    return () => clearInterval(interval)
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
    connectionState,
    lastError,
    signals,
    quality,
    mqttData,
    performance,
    // Signal operations
    subscribeSignal,
    unsubscribeSignal,
    setSignalValue,
    batchSetSignals,
    getSignalMetadata,
    subscribeSignalGroup,
    // MQTT operations
    subscribeMQTT,
    unsubscribeMQTT,
    publishMQTT,
    // Connection management
    reconnect: connect,
    disconnect,
    getConnectionDiagnostics,
    measureLatency
  }
}

// Hook to use a specific signal
export function usePetraSignal(signalName: string, defaultValue: any = null) {
  const { signals, subscribeSignal, unsubscribeSignal } = usePetraConnection()
  
  useEffect(() => {
    subscribeSignal(signalName)
    
    return () => {
      unsubscribeSignal(signalName)
    }
  }, [signalName, subscribeSignal, unsubscribeSignal])
  
  return signals.get(signalName) ?? defaultValue
}

// Hook to use MQTT topic data
export function useMQTTTopic(topic: string, defaultValue: any = null) {
  const { mqttData, subscribeMQTT, unsubscribeMQTT } = usePetraConnection()
  
  useEffect(() => {
    subscribeMQTT(topic)
    
    return () => {
      unsubscribeMQTT(topic)
    }
  }, [topic, subscribeMQTT, unsubscribeMQTT])
  
  return mqttData.get(topic) ?? defaultValue
}
