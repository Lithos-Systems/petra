// petra-designer/src/components/ConnectionStatus.tsx
import React from 'react'

interface ConnectionStatusProps {
  connected: boolean
  connectionState: string
  signals: Map<string, any>
  performance: any
  lastError: string | null
}

export default function ConnectionStatus({
  connected,
  connectionState,
  signals,
  performance,
  lastError
}: ConnectionStatusProps) {
  return (
    <div className="flex items-center gap-2 text-xs">
      <div className={`w-2 h-2 rounded-full ${
        connected ? 'bg-green-500' : 'bg-red-500'
      }`} />
      <span className="text-gray-300">
        {connectionState === 'connected' ? 'Connected' : 'Disconnected'}
      </span>
      {signals.size > 0 && (
        <span className="text-gray-400">
          ({signals.size} signals)
        </span>
      )}
    </div>
  )
}
