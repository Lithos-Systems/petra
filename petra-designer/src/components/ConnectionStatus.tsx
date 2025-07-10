import React from 'react'
import { usePetraConnection } from '@/hooks/usePetraConnection'

export default function ConnectionStatus() {
  const { connected, connectionState, performance } = usePetraConnection()

  const getStatusColor = () => {
    switch (connectionState) {
      case 'connected':
        return '#00C800'
      case 'connecting':
        return '#FFD700'
      case 'error':
        return '#FF0000'
      default:
        return '#808080'
    }
  }

  const getStatusText = () => {
    switch (connectionState) {
      case 'connected':
        return `PETRA Connected (${performance.latency}ms)`
      case 'connecting':
        return 'Connecting to PETRA...'
      case 'error':
        return 'PETRA Connection Error'
      default:
        return 'PETRA Disconnected'
    }
  }

  return (
    <div className="isa101-connection-status fixed bottom-0 right-0">
      <div
        className="isa101-connection-indicator"
        style={{ backgroundColor: getStatusColor() }}
      />
      <span className="text-xs">{getStatusText()}</span>
    </div>
  )
}
