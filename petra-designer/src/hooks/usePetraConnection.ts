// src/hooks/usePetraConnection.ts

import { useEffect, useState, useCallback, useRef } from 'react'
import { toast } from 'react-hot-toast'

interface SignalUpdate {
  type: 'signal_update'
  signal: string
  value: any
  timestamp: number
  quality?: 'good' | 'bad' | 'uncertain'
  source?: 'mqtt' | 'internal' | 's7' | 'modbus'
}

interface MQTTUpdate {
  type: 'mqtt_update'
  topic: string
  payload: any
  timestamp: number
}

interface PetraMessage {
  type: 'signal_update' | 'mqtt_update' | 'batch_update' | 'error' | 'connected'
  data?: any
  updates?: SignalUpdate[]
  error?: string
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
  _enableHistory = false,
  _onConnect,
  _onDisconnect,
  _onError,
}: PetraConnectionOptions = {}) {

  const [connected, setConnected] = useState(false)
  const [signals, setSignals] = useState<Map<string, any>>(new Map())
  const [quality, setQuality] = useState<Map<string, string>>(new Map())
  const [mqttData, setMqttData] = useState<Map<string, any>>(new Map())
  
  const wsRef = useRef<WebSocket | null>(null)
  const reconnectCountRef = useRef(0)
  const reconnectTimeoutRef = useRef<number>()
  const subscribedSignalsRef = useRef<Set<string>>(new Set())
  const subscribedTopicsRef = useRef<Set<string>>(new Set())
  const pingIntervalRef = useRef<number>()

  const connect = useCallback(() => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) return

    console.log(`Connecting to PETRA at ${url}...`)
    
    try {
      wsRef.current = new WebSocket(url)
      
      wsRef.current.onopen = () => {
        console.log('Connected to PETRA')
        setConnected(true)
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
          const message: PetraMessage = JSON.parse(event.data)
          
          switch (message.type) {
            case 'signal_update':
              const update = message.data as SignalUpdate
              setSignals(prev => new Map(prev).set(update.signal, update.value))
              
              if (update.quality) {
                setQuality(prev => new Map(prev).set(update.signal, update.quality!))
              }
              break
              
            case 'mqtt_update':
              const mqttUpdate = message.data as MQTTUpdate
              setMqttData(prev => new Map(prev).set(mqttUpdate.topic, mqttUpdate.payload))
              
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
                  newMap.set(update.signal, update.value)
                  if (update.quality) {
                    setQuality(prev => new Map(prev).set(update.signal, update.quality!))
                  }
                })
                return newMap
              })
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
        console.error('WebSocket error:', error)
      }

      wsRef.current.onclose = () => {
        setConnected(false)
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
          toast.error('Failed to connect to PETRA. Please check the connection.')
        }
      }
    } catch (error) {
      console.error('Failed to create WebSocket:', error)
      toast.error('Failed to connect to PETRA')
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
        value: value
      }))
    } else {
      toast.error('Not connected to PETRA')
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
    mqttData,
    // Signal operations
    subscribeSignal,
    unsubscribeSignal,
    setSignalValue,
    // MQTT operations
    subscribeMQTT,
    unsubscribeMQTT,
    publishMQTT,
    // Connection management
    reconnect: connect,
    disconnect
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
