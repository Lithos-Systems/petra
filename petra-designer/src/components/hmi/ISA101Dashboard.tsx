import { useEffect, useState } from 'react'
import { FaBell, FaHome, FaTachometerAlt, FaChartLine, FaCog } from 'react-icons/fa'

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
    <div className="isa101-display h-full flex flex-col bg-gray-100">
      {/* Header */}
      <header className="bg-gray-800 text-white px-4 py-2 flex items-center justify-between shadow-md">
        <div className="flex items-center gap-6">
          <div className="flex items-center gap-2">
            <div className="w-8 h-8 bg-blue-600 rounded flex items-center justify-center font-bold">
              P
            </div>
            <span className="font-bold text-lg">PETRA HMI</span>
          </div>
          <div className="border-l border-gray-600 pl-6">
            <div className="text-xs text-gray-300">{area}</div>
            <div className="font-semibold">{title}</div>
          </div>
        </div>
        
        {/* Navigation */}
        <nav className="flex items-center gap-4">
          <button
            onClick={() => onNavigate?.('overview')}
            className={`px-3 py-1 rounded flex items-center gap-2 text-sm ${
              currentScreen === 'overview' ? 'bg-blue-600' : 'bg-gray-700 hover:bg-gray-600'
            }`}
          >
            <FaHome className="w-4 h-4" />
            Overview
          </button>
          <button
            onClick={() => onNavigate?.('trends')}
            className={`px-3 py-1 rounded flex items-center gap-2 text-sm ${
              currentScreen === 'trends' ? 'bg-blue-600' : 'bg-gray-700 hover:bg-gray-600'
            }`}
          >
            <FaChartLine className="w-4 h-4" />
            Trends
          </button>
          <button
            onClick={() => onNavigate?.('alarms')}
            className={`px-3 py-1 rounded flex items-center gap-2 text-sm ${
              currentScreen === 'alarms' ? 'bg-blue-600' : 'bg-gray-700 hover:bg-gray-600'
            }`}
          >
            <FaBell className="w-4 h-4" />
            Alarms
          </button>
        </nav>

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

      {/* Alarm Banner */}
      {showAlarmBanner && totalAlarms > 0 && (
        <div
          className={`px-4 py-2 flex items-center justify-between text-sm font-semibold ${
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
            <FaBell className="w-4 h-4" />
            <span>Active Alarms: {totalAlarms}</span>
            {alarmCounts.critical > 0 && (
              <span className="bg-red-800 text-white px-2 py-1 rounded text-xs">
                Critical: {alarmCounts.critical}
              </span>
            )}
            {alarmCounts.high > 0 && (
              <span className="bg-orange-700 text-white px-2 py-1 rounded text-xs">
                High: {alarmCounts.high}
              </span>
            )}
            {hasUnacknowledged && (
              <span className="bg-white text-black px-2 py-1 rounded text-xs">
                Unack: {alarmCounts.unacknowledged}
              </span>
            )}
          </div>
          <button
            onClick={() => setShowAlarmBanner(false)}
            className="text-xs hover:underline"
          >
            Hide Banner
          </button>
        </div>
      )}

      {/* Main Content Area */}
      <main className="flex-1 overflow-hidden bg-gray-200">
        {children}
      </main>
    </div>
  )
}
