// src/hooks/useSystemMetrics.ts
import { useState } from 'react'

interface SystemMetrics {
  cpuUsage: number
  memoryUsage: number
  scanTime: number
  signalCount: number
  blockCount: number
}

export function useSystemMetrics() {
  const [metrics] = useState<SystemMetrics>({
    cpuUsage: 0,
    memoryUsage: 0,
    scanTime: 0,
    signalCount: 0,
    blockCount: 0
  })

  return metrics
}
