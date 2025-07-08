// src/hooks/useSignalHistory.ts
import { useState } from 'react'
import type { HistoryData } from './usePetraConnection'

interface TimeRange {
  start: number
  end: number
}

export function useSignalHistory(_signal: string, _timeRange: TimeRange) {
  const [history] = useState<HistoryData[]>([])
  const [loading, setLoading] = useState(false)

  const refresh = async () => {
    setLoading(true)
    // TODO: fetch historical data from backend
    setLoading(false)
  }

  return { history, loading, refresh }
}
