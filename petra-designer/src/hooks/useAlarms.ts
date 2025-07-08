// src/hooks/useAlarms.ts
import { useEffect, useState } from 'react'
import type { AlarmUpdate } from './usePetraConnection'
import { usePetraConnection } from './usePetraConnection'

export function useAlarms() {
  const [alarms, setAlarms] = useState<Map<string, AlarmUpdate>>(new Map())
  const [activeCount, setActiveCount] = useState(0)
  const { subscribeSignal, unsubscribeSignal } = usePetraConnection()

  useEffect(() => {
    subscribeSignal('alarms')
    return () => unsubscribeSignal('alarms')
  }, [subscribeSignal, unsubscribeSignal])

  const acknowledgeAlarm = (id: string) => {
    setAlarms(prev => {
      const a = new Map(prev)
      const alarm = a.get(id)
      if (alarm) alarm.acknowledged = true
      return a
    })
  }

  const clearAlarm = (id: string) => {
    setAlarms(prev => {
      const a = new Map(prev)
      a.delete(id)
      return a
    })
  }

  useEffect(() => {
    setActiveCount(Array.from(alarms.values()).filter(a => !a.acknowledged).length)
  }, [alarms])

  return { alarms, activeCount, acknowledgeAlarm, clearAlarm }
}
