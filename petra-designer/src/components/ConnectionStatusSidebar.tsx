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
        return performance.latency > 0 ? `Connected (${performance.latency}ms)` : 'Connected'
      case 'connecting':
        return 'Connecting...'
      case 'error':
        return 'Connection Error'
      default:
        return 'Disconnected'
    }
  }
  
  // Only show indicator if not connected or connecting
  const shouldShow = connectionState !== 'connected' || performance.latency > 100
  
  return (
    <div 
      className="isa101-connection-status"
      style={{
        opacity: shouldShow ? 1 : 0.7,
        transition: 'opacity 0.3s ease'
      }}
    >
      <div 
        className="isa101-connection-indicator"
        style={{ 
          backgroundColor: getStatusColor(),
          animation: connectionState === 'connecting' ? 'pulse 1.5s infinite' : 'none'
        }}
      />
      <span className="text-xs font-medium">
        PETRA: {getStatusText()}
      </span>
      {connectionState === 'connected' && performance.messageRate > 0 && (
        <span className="text-xs opacity-75 ml-2">
          {performance.messageRate} msg/s
        </span>
      )}
    </div>
  )
}

// Add pulse animation
const style = document.createElement('style')
style.textContent = `
  @keyframes pulse {
    0% {
      opacity: 1;
      transform: scale(1);
    }
    50% {
      opacity: 0.5;
      transform: scale(0.8);
    }
    100% {
      opacity: 1;
      transform: scale(1);
    }
  }
`
document.head.appendChild(style)
