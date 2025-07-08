// src/hooks/useBlockStatus.ts
import { useState } from 'react'
import { usePetraConnection } from './usePetraConnection'

interface BlockStatus {
  id: string
  healthy: boolean
  lastExecution: number
  errorCount: number
}

export function useBlockStatus() {
  const [blockStatus, setBlockStatus] = useState<Map<string, BlockStatus>>(new Map())
  usePetraConnection() // ensure connection is active

  const getBlockHealth = (id: string) => blockStatus.get(id)

  return { blockStatus, getBlockHealth }
}
