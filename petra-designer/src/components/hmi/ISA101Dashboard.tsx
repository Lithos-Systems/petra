// File: petra-designer/src/components/hmi/ISA101Dashboard.tsx
// ISA-101 Compliant Dashboard Layout Component
import { useState, useEffect } from 'react'
import { FaBell, FaExclamationTriangle, FaHome, FaChartLine, FaCog, FaBook } from 'react-icons/fa'

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
  currentScreen = 'overview'
}: ISA101DashboardProps) {
  const [currentTime, setCurrentTime] = useState(new Date())
  const [showAlarmBanner, setShowAlarmBanner] = useState(true)
  
  // Update time every second
  useEffect(() => {
    const timer = setInterval(() => {
      setCurrentTime(new Date())
    }, 1000)
    return () => clearInterval(timer)
  }, [])
  
  // Get alarm counts by priority
  const getAlarmCounts = () => {
    const counts = {
      critical: 0,
      high: 0,
      medium: 0,
      low: 0,
      unacknowledged: 0
    }
    
    alarms.forEach(alarm => {
      counts[alarm.priority]++
      if (!alarm.acknowledged) counts.unacknowledged++
    })
    
    return counts
  }
  
  const alarmCounts = getAlarmCounts()
  const totalAlarms = alarms.length
  const hasUnacknowledged = alarmCounts.unacknowledged > 0
  
  // Get highest priority alarm
  const getHighestPriorityAlarm = () => {
    if (alarmCounts.critical > 0) return 'critical'
    if (alarmCounts.high > 0) return 'high'
    if (alarmCounts.medium > 0) return 'medium'
    if (alarmCounts.low > 0) return 'low'
    return null
  }
  
  const highestPriority = getHighestPriorityAlarm()
  
  return (
    <div className="isa101-display h-screen flex flex-col">
      {/* Header Bar */}
      <header className="bg-gray-800 text-white px-4 py-2 flex items-center justify-between">
        <div className="flex items-center gap-6">
          {/* Logo/System Name */}
          <div className="flex items-center gap-2">
            <div className="w-8 h-8 bg-blue-600 rounded flex items-center justify-center">
              <span className="font-bold">P</span>
            </div>
            <span className="font-bold">PETRA HMI</span>
          </div>
          
          {/* Area/Title */}
          <div className="border-l border-gray-600 pl-6">
            <div className="text-xs text-gray-300">{area}</div>
            <div className="font-semibold">{title}</div>
          </div>
        </div>
        
        {/* Status Bar */}
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
      
      {/* Alarm Banner (ISA-101 recommendation) */}
      {showAlarmBanner && totalAlarms > 0 && (
        <div className={`px-4 py-2 flex items-center justify-between ${
          highestPriority === 'critical' ? 'bg-red-600 text-white' :
          highestPriority === 'high' ? 'bg-orange-500 text-white' :
          highestPriority === 'medium' ? 'bg-yellow-400 text-black' :
          'bg-cyan-400 text-black'
        } ${hasUnacknowledged ? 'animate-pulse' : ''}`}>
          <div className="flex items-center gap-4">
            <FaBell className="w
