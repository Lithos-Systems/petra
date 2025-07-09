import { useEffect, useState } from 'react'
import { FaBell } from 'react-icons/fa'

interface AlarmItem {
  id: string
  timestamp: Date
  tagName: string
  description: string
  value: number
  units: string
  priority: 'critical' | 'high' | 'medium' | 'low'
  acknowledged: boolean
  area: string
}

interface ISA101DashboardProps {
  title: string
  area: string
  children: React.ReactNode
  alarms?: AlarmItem[]
  onNavigate?: (screen: string) => void
  currentUser?: string
  currentScreen?: string
}

export default function ISA101Dashboard({
  title,
  area,
  children,
  alarms = [],
  onNavigate,
  currentUser = 'Operator',
  currentScreen = 'overview',
}: ISA101DashboardProps) {
  const [currentTime, setCurrentTime] = useState(new Date())
  const [showAlarmBanner, setShowAlarmBanner] = useState(true)

  useEffect(() => {
    const timer = setInterval(() => {
      setCurrentTime(new Date())
    }, 1000)
    return () => clearInterval(timer)
  }, [])

  const alarmCounts = alarms.reduce(
    (acc, alarm) => {
      acc[alarm.priority]++
      if (!alarm.acknowledged) acc.unacknowledged++
      return acc
    },
    { critical: 0, high: 0, medium: 0, low: 0, unacknowledged: 0 }
  )

  const totalAlarms = alarms.length
  const hasUnacknowledged = alarmCounts.unacknowledged > 0

  const highestPriority =
    alarmCounts.critical > 0
      ? 'critical'
      : alarmCounts.high > 0
      ? 'high'
      : alarmCounts.medium > 0
      ? 'medium'
      : alarmCounts.low > 0
      ? 'low'
      : null

  return (
    <div className="isa101-display h-full flex flex-col">
      <header className="bg-gray-800 text-white px-4 py-2 flex items-center justify-between">
        <div className="flex items-center gap-6">
          <div className="flex items-center gap-2">
            <div className="w-8 h-8 bg-blue-600 rounded flex items-center justify-center">
              <span className="font-bold">P</span>
            </div>
            <span className="font-bold">PETRA HMI</span>
          </div>
          <div className="border-l border-gray-600 pl-6">
            <div className="text-xs text-gray-300">{area}</div>
            <div className="font-semibold">{title}</div>
          </div>
        </div>
        <div className="flex items-center gap-6 text-sm">
          <div className="flex items-center gap-2">
            <span className="text-gray-300">User:</span>
            <span>{currentUser}</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-gray-300">Time:</span>
            <span className="font-mono">{currentTime.toLocaleTimeString()}</span>
          </div>
        </div>
      </header>

      {showAlarmBanner && totalAlarms > 0 && (
        <div
          className={`px-4 py-2 flex items-center justify-between ${
            highestPriority === 'critical'
              ? 'bg-red-600 text-white'
              : highestPriority === 'high'
              ? 'bg-orange-500 text-white'
              : highestPriority === 'medium'
              ? 'bg-yellow-400 text-black'
              : 'bg-cyan-400 text-black'
          } ${hasUnacknowledged ? 'animate-pulse' : ''}`}
        >
          <div className="flex items-center gap-4">
            <FaBell className="w-5 h-5" />
            <span>Alarms: {totalAlarms}</span>
          </div>
          <button onClick={() => setShowAlarmBanner(false)}>Hide</button>
        </div>
      )}

      <main className="flex-1 overflow-hidden">{children}</main>
    </div>
  )
}
