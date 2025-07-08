// src/hooks/useOfflineQueue.ts
import { useState } from 'react'
import { nanoid } from 'nanoid'

interface QueuedMessage {
  id: string
  message: any
  timestamp: number
  retries: number
}

export function useOfflineQueue() {
  const [queue, setQueue] = useState<QueuedMessage[]>([])

  const queueMessage = (message: any) => {
    setQueue(prev => [
      ...prev,
      { id: nanoid(), message, timestamp: Date.now(), retries: 0 }
    ])
  }

  const sendMessage = async (_msg: any) => {
    // Placeholder for actual send logic
    return Promise.resolve()
  }

  const flushQueue = async () => {
    for (const item of queue) {
      try {
        await sendMessage(item.message)
        setQueue(prev => prev.filter(i => i.id !== item.id))
      } catch {
        item.retries++
        if (item.retries > 3) {
          setQueue(prev => prev.filter(i => i.id !== item.id))
        }
      }
    }
  }

  return { queue, queueMessage, flushQueue }
}
