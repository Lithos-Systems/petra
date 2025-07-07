// src/components/hmi/WaterPlantDemo.tsx

import { useState, useEffect, useRef } from 'react'
import { Stage, Layer, Group, Rect, Circle, Line, Text, Path } from 'react-konva'
import { FaPlay, FaPause, FaCog, FaChartLine, FaExclamationTriangle } from 'react-icons/fa'
import TankComponent from './components/TankComponent'
import PumpComponent from './components/PumpComponent'
import ValveComponent from './components/ValveComponent'
import GaugeComponent from './components/GaugeComponent'

interface PumpSetpoints {
  leadStartPressure: number // PSI - Lead pump starts
  leadStopPressure: number // PSI - Lead pump stops  
  lagStartPressure: number // PSI - Lag pumps start
  lagStopPressure: number // PSI - Lag pumps stop
  alarmLowPressure: number // Low pressure alarm (psi)
  alarmHighPressure: number // High pressure alarm (psi)
}

interface WellSetpoints {
  startLevel: number // Tank level feet to start
  stopLevel: number // Tank level feet to stop
  alarmHighFlow: number // High flow alarm (gpm)
  flowRate: number // Well pump flow rate when running
}

interface HydrotankSetpoints {
  targetAirBlanket: number // Target air blanket percentage
  compressorStartLevel: number // Air blanket % to start compressor
  compressorStopLevel: number // Air blanket % to stop compressor
}

interface SimulationState {
  // Tank parameters
  tankCapacity: number // gallons
  tankLevel: number // gallons
  tankLevelPercent: number // %
  tankLevelFeet: number // feet
  tankHeight: number // feet
  
  // Well parameters
  wellFlowRate: number // Current flow rate (0 when stopped)
  wellRunning: boolean
  wellSetpoints: WellSetpoints
  
  // System pressure setpoints (common for all pumps)
  pumpSetpoints: PumpSetpoints
  
  // Pump parameters
  boosterPumps: Array<{
    id: string
    name: string
    maxFlowRate: number // Maximum flow rate capability
    flowRate: number // Current setpoint flow rate
    running: boolean
    efficiency: number // %
    pressure: number // psi
    currentFlow: number // actual flow (0 when stopped)
    pumpNumber: number // 1, 2, 3 for rotation sequence
    inAlarm: boolean
    alarmMessage?: string
    startDelay?: number // seconds until start (for staging)
  }>
  
  // Lead pump tracking for round-robin
  currentLeadPump: number // 1, 2, or 3
  
  // Hydrotank parameters
  hydrotanks: Array<{
    id: string
    name: string
    capacity: number // gallons
    waterLevel: number // gallons
    waterLevelPercent: number // %
    airBlanketPercent: number // %
    pressure: number // psi
    compressorRunning: boolean
  }>
  hydrotankSetpoints: HydrotankSetpoints
  
  // System parameters
  demand: number // gpm
  systemPressure: number // psi
  targetPressure: number // psi
  
  // Simulation
  running: boolean
  timeMultiplier: number
  
  // Alarms
  activeAlarms: Array<{
    id: string
    source: string
    message: string
    severity: 'low' | 'medium' | 'high'
    timestamp: Date
  }>
}

export default function WaterPlantDemo() {
  const [simulation, setSimulation] = useState<SimulationState>({
    tankCapacity: 200000,
    tankLevel: 100000,
    tankLevelPercent: 50,
    tankLevelFeet: 12.5, // 50% of 25ft
    tankHeight: 25,
    wellFlowRate: 0,
    wellRunning: false,
    wellSetpoints: {
      startLevel: 8,  // 8 feet
      stopLevel: 20,  // 20 feet
      alarmHighFlow: 3000,
      flowRate: 2000, // Well pump flow rate when running
    },
    pumpSetpoints: {
      leadStartPressure: 55,
      leadStopPressure: 65,
      lagStartPressure: 50,
      lagStopPressure: 70,
      alarmLowPressure: 40,
      alarmHighPressure: 80,
    },
    boosterPumps: [
      { 
        id: 'bp1', 
        name: 'Pump 1', 
        maxFlowRate: 3000,
        flowRate: 1500,
        currentFlow: 0,
        running: false, 
        efficiency: 85, 
        pressure: 60,
        pumpNumber: 1,
        inAlarm: false
      },
      { 
        id: 'bp2', 
        name: 'Pump 2', 
        maxFlowRate: 3000,
        flowRate: 1500,
        currentFlow: 0,
        running: false, 
        efficiency: 85, 
        pressure: 60,
        pumpNumber: 2,
        inAlarm: false
      },
      { 
        id: 'bp3', 
        name: 'Pump 3', 
        maxFlowRate: 3000,
        flowRate: 2000,
        currentFlow: 0,
        running: false, 
        efficiency: 90, 
        pressure: 65,
        pumpNumber: 3,
        inAlarm: false
      },
    ],
    currentLeadPump: 1, // Start with pump 1 as lead
    hydrotanks: [
      {
        id: 'ht1',
        name: 'Hydrotank 1',
        capacity: 5000,
        waterLevel: 2500,
        waterLevelPercent: 50,
        airBlanketPercent: 50,
        pressure: 60,
        compressorRunning: false,
      },
      {
        id: 'ht2',
        name: 'Hydrotank 2',
        capacity: 5000,
        waterLevel: 2500,
        waterLevelPercent: 50,
        airBlanketPercent: 50,
        pressure: 60,
        compressorRunning: false,
      },
    ],
    hydrotankSetpoints: {
      targetAirBlanket: 50,
      compressorStartLevel: 45,
      compressorStopLevel: 55,
    },
    demand: 2500,
    systemPressure: 60,
    targetPressure: 60,
    running: true,
    timeMultiplier: 10,
    activeAlarms: [],
  })
  
  const [showControls, setShowControls] = useState(true)
  const [stageSize, setStageSize] = useState({ width: 800, height: 600 })
  
  // Update stage size on window resize
  useEffect(() => {
    const updateSize = () => {
      setStageSize({
        width: window.innerWidth - (showControls ? 384 : 0),
        height: window.innerHeight - 60
      })
    }
    updateSize()
    window.addEventListener('resize', updateSize)
    return () => window.removeEventListener('resize', updateSize)
  }, [showControls])
  
  // Check alarms
  const checkAlarms = (state: SimulationState) => {
    const newAlarms: typeof state.activeAlarms = []
    const updatedPumps = state.boosterPumps.map(pump => {
      const pumpAlarms: string[] = []
      
      if (pump.running) {
        if (pump.pressure < state.pumpSetpoints.alarmLowPressure) {
          pumpAlarms.push(`Low pressure: ${pump.pressure} psi`)
        }
        if (pump.pressure > state.pumpSetpoints.alarmHighPressure) {
          pumpAlarms.push(`High pressure: ${pump.pressure} psi`)
        }
      }
      
      const inAlarm = pumpAlarms.length > 0
      if (inAlarm) {
        newAlarms.push({
          id: `${pump.id}-${Date.now()}`,
          source: pump.name,
          message: pumpAlarms.join(', '),
          severity: 'medium' as const,
          timestamp: new Date(),
        })
      }
      
      return { ...pump, inAlarm, alarmMessage: pumpAlarms[0] }
    })
    
    // Check well alarms
    if (state.wellRunning && state.wellFlowRate > state.wellSetpoints.alarmHighFlow) {
      newAlarms.push({
        id: `well-${Date.now()}`,
        source: 'Well Pump',
        message: `High flow: ${state.wellFlowRate} gpm`,
        severity: 'medium' as const,
        timestamp: new Date(),
      })
    }
    
    return { updatedPumps, newAlarms }
  }
  
  // Simulation loop
  useEffect(() => {
    if (!simulation.running) return
    
    const interval = setInterval(() => {
      setSimulation(prev => {
        const timeStep = 1 / 60 * prev.timeMultiplier // 1 second of simulation time
        
        // Calculate tank level in feet
        const currentLevelFeet = (prev.tankLevelPercent / 100) * prev.tankHeight
        
        // Well pump control based solely on tank level
        let wellRunning = prev.wellRunning
        let wellFlowRate = prev.wellFlowRate
        
        if (currentLevelFeet <= prev.wellSetpoints.startLevel && !wellRunning) {
          wellRunning = true
          wellFlowRate = prev.wellSetpoints.flowRate
        } else if (currentLevelFeet >= prev.wellSetpoints.stopLevel && wellRunning) {
          wellRunning = false
          wellFlowRate = 0
        } else if (wellRunning) {
          wellFlowRate = prev.wellSetpoints.flowRate
        }
        
        // Round-robin lead/lag pump control
        let currentLeadPump = prev.currentLeadPump
        const leadPump = prev.boosterPumps.find(p => p.pumpNumber === currentLeadPump)

        let updatedPumps = [...prev.boosterPumps]

        const runningCount = updatedPumps.filter(p => p.running).length

        // Start lead pump when pressure drops below lead start
        if (!leadPump?.running && prev.systemPressure <= prev.pumpSetpoints.leadStartPressure) {
          updatedPumps = updatedPumps.map(pump =>
            pump.pumpNumber === currentLeadPump ? { ...pump, running: true } : pump
          )
        }

        // Start lag pumps sequentially if pressure remains low
        if (prev.systemPressure <= prev.pumpSetpoints.lagStartPressure && runningCount > 0) {
          const nextPump = updatedPumps.find(p => !p.running && !p.startDelay)
          if (nextPump) {
            updatedPumps = updatedPumps.map(pump =>
              pump.id === nextPump.id ? { ...pump, startDelay: 5 } : pump
            )
          }
        }

        // Stop lag pumps once pressure recovers
        if (prev.systemPressure >= prev.pumpSetpoints.lagStopPressure) {
          const lagRunning = updatedPumps.filter(p => p.running && p.pumpNumber !== currentLeadPump)
          if (lagRunning.length > 0) {
            const pumpToStop = lagRunning[lagRunning.length - 1]
            updatedPumps = updatedPumps.map(pump =>
              pump.id === pumpToStop.id ? { ...pump, running: false, currentFlow: 0, startDelay: undefined } : pump
            )
          } else if (leadPump?.running && prev.systemPressure >= prev.pumpSetpoints.leadStopPressure) {
            // No lag pumps running and pressure high - stop lead and rotate
            updatedPumps = updatedPumps.map(pump =>
              pump.pumpNumber === currentLeadPump ? { ...pump, running: false, currentFlow: 0, startDelay: undefined } : pump
            )
            currentLeadPump = (currentLeadPump % 3) + 1
          }
        }
        
        // Handle staging delays
        updatedPumps = updatedPumps.map(pump => {
          let { running, currentFlow, startDelay } = pump
          
          if (startDelay && startDelay > 0) {
            startDelay = Math.max(0, startDelay - timeStep)
            if (startDelay === 0) {
              running = true
              startDelay = undefined
            }
          }
          
          // Update current flow based on running state and setpoint
          currentFlow = running ? pump.flowRate * (pump.efficiency / 100) : 0
          
          return { ...pump, running, currentFlow, startDelay }
        })
        
        // Calculate total pump output
        const totalPumpOutput = updatedPumps.reduce((sum, pump) => sum + pump.currentFlow, 0)
        
        // Calculate net flow to storage tank
        const netFlow = wellFlowRate - totalPumpOutput // Pumps draw from tank
        
        // Update tank level
        const newLevel = Math.max(0, Math.min(prev.tankCapacity, prev.tankLevel + (netFlow * timeStep)))
        const newLevelPercent = (newLevel / prev.tankCapacity) * 100
        const newLevelFeet = (newLevelPercent / 100) * prev.tankHeight
        
        // Calculate target pressure based on pump capacity vs demand
        let targetSystemPressure = 0
        const runningPumps = updatedPumps.filter(p => p.running)
        
        if (runningPumps.length > 0 && totalPumpOutput > 0) {
          // Base pressure when supply meets demand
          const basePressure = 60
          
          // Calculate supply/demand ratio
          const supplyDemandRatio = totalPumpOutput / Math.max(1, prev.demand)
          
          if (supplyDemandRatio >= 1) {
            // Excess capacity increases pressure (up to max 100 psi)
            targetSystemPressure = Math.min(100, basePressure + (supplyDemandRatio - 1) * 20)
          } else {
            // Insufficient capacity decreases pressure
            targetSystemPressure = basePressure * supplyDemandRatio
          }
        }
        
        // Update hydrotanks
        const totalHydrotankCapacity = prev.hydrotanks.reduce((sum, ht) => sum + ht.capacity, 0)
        const netSystemFlow = totalPumpOutput - prev.demand // Net flow into hydrotanks
        
        const updatedHydrotanks = prev.hydrotanks.map(ht => {
          // Distribute flow proportionally
          const htFlowShare = netSystemFlow * (ht.capacity / totalHydrotankCapacity) * timeStep
          
          // Update water level
          let newWaterLevel = Math.max(0, Math.min(ht.capacity, ht.waterLevel + htFlowShare))
          let newWaterPercent = (newWaterLevel / ht.capacity) * 100
          let newAirBlanket = 100 - newWaterPercent
          
          // Ensure values stay within bounds
          newWaterPercent = Math.max(0, Math.min(100, newWaterPercent))
          newAirBlanket = Math.max(0, Math.min(100, newAirBlanket))
          
          // Air compressor control
          let compressorRunning = ht.compressorRunning
          if (newAirBlanket < prev.hydrotankSetpoints.compressorStartLevel) {
            compressorRunning = true
          } else if (newAirBlanket > prev.hydrotankSetpoints.compressorStopLevel) {
            compressorRunning = false
          }
          
          // Simulate air injection (increases air blanket)
          if (compressorRunning) {
            const airInjectionRate = 50 // gallons per minute equivalent
            newWaterLevel = Math.max(0, newWaterLevel - (airInjectionRate * timeStep))
            newWaterPercent = (newWaterLevel / ht.capacity) * 100
            newAirBlanket = 100 - newWaterPercent
            
            // Ensure bounds after air injection
            newWaterPercent = Math.max(0, Math.min(100, newWaterPercent))
            newAirBlanket = Math.max(0, Math.min(100, newAirBlanket))
          }
          
          return {
            ...ht,
            waterLevel: newWaterLevel,
            waterLevelPercent: newWaterPercent,
            airBlanketPercent: newAirBlanket,
            pressure: 0, // Will be set to system pressure
            compressorRunning,
          }
        })
        
        // Apply hydrotank dampening to pressure changes
        const pressureChangeRate = 0.1 // 10% change per time step
        let newPressure = prev.systemPressure
        
        if (targetSystemPressure > prev.systemPressure) {
          newPressure = Math.min(targetSystemPressure, prev.systemPressure + (targetSystemPressure - prev.systemPressure) * pressureChangeRate)
        } else {
          newPressure = Math.max(targetSystemPressure, prev.systemPressure - (prev.systemPressure - targetSystemPressure) * pressureChangeRate)
        }
        
        // Ensure pressure stays within bounds
        newPressure = Math.max(0, Math.min(100, newPressure))
        
        // Update hydrotank pressures to match system
        const finalHydrotanks = updatedHydrotanks.map(ht => ({
          ...ht,
          pressure: Math.round(newPressure)
        }))
        
        // Check alarms
        const { updatedPumps: alarmedPumps, newAlarms } = checkAlarms({
          ...prev,
          boosterPumps: updatedPumps,
          wellRunning,
          wellFlowRate,
          systemPressure: Math.round(newPressure),
        })
        
        const allAlarms = [...prev.activeAlarms, ...newAlarms].slice(-10)
        
        return {
          ...prev,
          wellRunning,
          wellFlowRate,
          boosterPumps: alarmedPumps,
          hydrotanks: finalHydrotanks,
          tankLevel: newLevel,
          tankLevelPercent: newLevelPercent,
          tankLevelFeet: newLevelFeet,
          systemPressure: Math.round(newPressure),
          activeAlarms: allAlarms,
          currentLeadPump,
        }
      })
    }, 100)
    
    return () => clearInterval(interval)
  }, [simulation.running, simulation.timeMultiplier])
  
  const togglePump = (pumpId: string) => {
    setSimulation(prev => ({
      ...prev,
      boosterPumps: prev.boosterPumps.map(pump =>
        pump.id === pumpId ? { ...pump, running: !pump.running, startDelay: undefined } : pump
      )
    }))
  }
  
  const updatePumpSetpoint = (field: keyof PumpSetpoints, value: number) => {
    setSimulation(prev => ({
      ...prev,
      pumpSetpoints: { ...prev.pumpSetpoints, [field]: value }
    }))
  }
  
  const updateWellSetpoint = (field: keyof WellSetpoints, value: number) => {
    setSimulation(prev => ({
      ...prev,
      wellSetpoints: { ...prev.wellSetpoints, [field]: value }
    }))
  }
  
  const updateHydrotankSetpoint = (field: keyof HydrotankSetpoints, value: number) => {
    setSimulation(prev => ({
      ...prev,
      hydrotankSetpoints: { ...prev.hydrotankSetpoints, [field]: value }
    }))
  }
  
  const updatePumpFlowRate = (pumpId: string, flowRate: number) => {
    setSimulation(prev => ({
      ...prev,
      boosterPumps: prev.boosterPumps.map(pump =>
        pump.id === pumpId ? { ...pump, flowRate: Math.min(flowRate, pump.maxFlowRate) } : pump
      )
    }))
  }
  
  const updatePumpEfficiency = (pumpId: string, efficiency: number) => {
    setSimulation(prev => ({
      ...prev,
      boosterPumps: prev.boosterPumps.map(pump =>
        pump.id === pumpId ? { ...pump, efficiency: Math.max(0, Math.min(100, efficiency)) } : pump
      )
    }))
  }
  
  return (
    <div className="flex h-full bg-gray-50">
      {/* Control Panel */}
      {showControls && (
        <div className="w-96 bg-white border-r border-gray-200 p-4 overflow-y-auto">
          <h2 className="text-xl font-bold mb-4 flex items-center gap-2">
            <FaCog className="text-petra-600" />
            Water Plant Controls
          </h2>
          
          {/* Simulation Control */}
          <div className="mb-6 p-4 bg-gray-50 rounded-lg">
            <div className="flex items-center justify-between mb-3">
              <h3 className="font-semibold">Simulation</h3>
              <button
                onClick={() => setSimulation(prev => ({ ...prev, running: !prev.running }))}
                className={`px-3 py-1 rounded flex items-center gap-2 text-white transition-colors ${
                  simulation.running ? 'bg-red-500 hover:bg-red-600' : 'bg-green-500 hover:bg-green-600'
                }`}
              >
                {simulation.running ? <FaPause /> : <FaPlay />}
                {simulation.running ? 'Pause' : 'Start'}
              </button>
            </div>
            
            <div>
              <label className="text-sm text-gray-600">Speed Multiplier</label>
              <input
                type="range"
                min="1"
                max="60"
                value={simulation.timeMultiplier}
                onChange={(e) => setSimulation(prev => ({ ...prev, timeMultiplier: parseInt(e.target.value) }))}
                className="w-full"
              />
              <span className="text-sm text-gray-500">{simulation.timeMultiplier}x</span>
            </div>

            {/* Demand Control */}
            <div className="mt-4">
              <label className="text-sm text-gray-600">System Demand (gpm)</label>
              <input
                type="range"
                min="0"
                max="10000"
                step="100"
                value={simulation.demand}
                onChange={(e) => setSimulation(prev => ({ ...prev, demand: parseInt(e.target.value) }))}
                className="w-full"
              />
              <div className="text-sm text-gray-500 mt-1">{simulation.demand.toLocaleString()} gpm</div>
            </div>
          </div>
          
          {/* System Status */}
          <div className="mb-6 p-4 bg-blue-50 rounded-lg">
            <h3 className="font-semibold mb-3">System Status</h3>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span>System Pressure:</span>
                <span className={`font-bold ${
                  simulation.systemPressure < simulation.targetPressure * 0.9 ? 'text-red-600' :
                  simulation.systemPressure > simulation.targetPressure * 1.1 ? 'text-orange-600' :
                  'text-green-600'
                }`}>
                  {simulation.systemPressure} psi
                </span>
              </div>
              <div className="flex justify-between">
                <span>Tank Level:</span>
                <span className="font-medium">{simulation.tankLevelFeet.toFixed(1)} ft ({simulation.tankLevelPercent.toFixed(0)}%)</span>
              </div>
              <div className="flex justify-between">
                <span>Total Flow:</span>
                <span className="font-medium">{simulation.boosterPumps.reduce((sum, p) => sum + p.currentFlow, 0).toFixed(0)} gpm</span>
              </div>
            </div>
          </div>
          
          {/* Well Pump Setpoints */}
          <div className="mb-6">
            <h3 className="font-semibold mb-3">Well Pump Control</h3>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between items-center">
                <span>Start Level (ft):</span>
                <input
                  type="number"
                  value={simulation.wellSetpoints.startLevel}
                  onChange={(e) => updateWellSetpoint('startLevel', parseFloat(e.target.value))}
                  className="w-20 px-2 py-1 border rounded text-right"
                  min="0"
                  max={simulation.tankHeight}
                  step="0.5"
                />
              </div>
              <div className="flex justify-between items-center">
                <span>Stop Level (ft):</span>
                <input
                  type="number"
                  value={simulation.wellSetpoints.stopLevel}
                  onChange={(e) => updateWellSetpoint('stopLevel', parseFloat(e.target.value))}
                  className="w-20 px-2 py-1 border rounded text-right"
                  min="0"
                  max={simulation.tankHeight}
                  step="0.5"
                />
              </div>
              <div className="flex justify-between items-center">
                <span>Flow Rate (gpm):</span>
                <input
                  type="number"
                  value={simulation.wellSetpoints.flowRate}
                  onChange={(e) => updateWellSetpoint('flowRate', parseInt(e.target.value))}
                  className="w-20 px-2 py-1 border rounded text-right"
                  min="0"
                  max="5000"
                  step="100"
                />
              </div>
              <div className="flex justify-between items-center">
                <span>High Flow Alarm:</span>
                <input
                  type="number"
                  value={simulation.wellSetpoints.alarmHighFlow}
                  onChange={(e) => updateWellSetpoint('alarmHighFlow', parseInt(e.target.value))}
                  className="w-20 px-2 py-1 border rounded text-right"
                  min="0"
                  max="5000"
                />
              </div>
              <div className="mt-2 p-2 bg-gray-100 rounded">
                <div className="text-xs text-gray-600">
                  Current: {simulation.wellRunning ? 'Running' : 'Stopped'} at {simulation.wellFlowRate} gpm
                </div>
              </div>
            </div>
          </div>
          
          {/* Pump Pressure Setpoints */}
          <div className="mb-6">
            <h3 className="font-semibold mb-3">Pump Pressure Control</h3>
            <div className="space-y-2 text-sm">
              <div className="font-medium text-gray-700">Lead Pump</div>
              <div className="flex justify-between items-center">
                <span>Start Pressure:</span>
                <input
                  type="number"
                  value={simulation.pumpSetpoints.leadStartPressure}
                  onChange={(e) => updatePumpSetpoint('leadStartPressure', parseInt(e.target.value))}
                  className="w-20 px-2 py-1 border rounded text-right"
                />
              </div>
              <div className="flex justify-between items-center">
                <span>Stop Pressure:</span>
                <input
                  type="number"
                  value={simulation.pumpSetpoints.leadStopPressure}
                  onChange={(e) => updatePumpSetpoint('leadStopPressure', parseInt(e.target.value))}
                  className="w-20 px-2 py-1 border rounded text-right"
                />
              </div>
              
              <div className="font-medium text-gray-700 mt-3">Lag Pumps</div>
              <div className="flex justify-between items-center">
                <span>Start Pressure:</span>
                <input
                  type="number"
                  value={simulation.pumpSetpoints.lagStartPressure}
                  onChange={(e) => updatePumpSetpoint('lagStartPressure', parseInt(e.target.value))}
                  className="w-20 px-2 py-1 border rounded text-right"
                />
              </div>
              <div className="flex justify-between items-center">
                <span>Stop Pressure:</span>
                <input
                  type="number"
                  value={simulation.pumpSetpoints.lagStopPressure}
                  onChange={(e) => updatePumpSetpoint('lagStopPressure', parseInt(e.target.value))}
                  className="w-20 px-2 py-1 border rounded text-right"
                />
              </div>
              
              <div className="font-medium text-gray-700 mt-3">Alarms</div>
              <div className="flex justify-between items-center">
                <span>Low Pressure:</span>
                <input
                  type="number"
                  value={simulation.pumpSetpoints.alarmLowPressure}
                  onChange={(e) => updatePumpSetpoint('alarmLowPressure', parseInt(e.target.value))}
                  className="w-20 px-2 py-1 border rounded text-right"
                />
              </div>
              <div className="flex justify-between items-center">
                <span>High Pressure:</span>
                <input
                  type="number"
                  value={simulation.pumpSetpoints.alarmHighPressure}
                  onChange={(e) => updatePumpSetpoint('alarmHighPressure', parseInt(e.target.value))}
                  className="w-20 px-2 py-1 border rounded text-right"
                />
              </div>
            </div>
          </div>
          
          {/* Hydrotank Setpoints */}
          <div className="mb-6">
            <h3 className="font-semibold mb-3">Hydrotank Air Blanket Control</h3>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between items-center">
                <span>Target Air Blanket (%):</span>
                <input
                  type="number"
                  value={simulation.hydrotankSetpoints.targetAirBlanket}
                  onChange={(e) => updateHydrotankSetpoint('targetAirBlanket', parseInt(e.target.value))}
                  className="w-20 px-2 py-1 border rounded text-right"
                  min="20"
                  max="80"
                />
              </div>
              <div className="flex justify-between items-center">
                <span>Compressor Start (%):</span>
                <input
                  type="number"
                  value={simulation.hydrotankSetpoints.compressorStartLevel}
                  onChange={(e) => updateHydrotankSetpoint('compressorStartLevel', parseInt(e.target.value))}
                  className="w-20 px-2 py-1 border rounded text-right"
                  min="20"
                  max="80"
                />
              </div>
              <div className="flex justify-between items-center">
                <span>Compressor Stop (%):</span>
                <input
                  type="number"
                  value={simulation.hydrotankSetpoints.compressorStopLevel}
                  onChange={(e) => updateHydrotankSetpoint('compressorStopLevel', parseInt(e.target.value))}
                  className="w-20 px-2 py-1 border rounded text-right"
                  min="20"
                  max="80"
                />
              </div>
            </div>
          </div>
          
          {/* Current Lead Pump Indicator */}
          <div className="mb-6 p-4 bg-green-50 rounded-lg">
            <h3 className="font-semibold mb-2">Lead Pump Rotation</h3>
            <div className="text-sm">
              <span>Current Lead: </span>
              <span className="font-bold text-green-700">Pump {simulation.currentLeadPump}</span>
            </div>
            <div className="text-xs text-gray-600 mt-1">
              Lead pump rotates when stop pressure is reached
            </div>
          </div>
          
          {/* Booster Pump Status */}
          <div className="mb-6">
            <h3 className="font-semibold mb-3">Booster Pump Status</h3>
            <div className="space-y-3">
              {simulation.boosterPumps.map(pump => (
                <div 
                  key={pump.id} 
                  className={`p-3 border rounded-lg ${
                    pump.inAlarm ? 'border-red-500 bg-red-50' : 
                    pump.pumpNumber === simulation.currentLeadPump ? 'border-green-500 bg-green-50' :
                    'border-gray-200'
                  }`}
                >
                  <div className="flex items-center justify-between mb-2">
                    <span className="font-medium">
                      {pump.name} 
                      {pump.pumpNumber === simulation.currentLeadPump && 
                        <span className="ml-2 text-xs bg-green-600 text-white px-2 py-0.5 rounded">LEAD</span>
                      }
                    </span>
                    <button
                      onClick={() => togglePump(pump.id)}
                      className={`px-3 py-1 rounded text-sm ${
                        pump.running 
                          ? 'bg-green-500 text-white' 
                          : pump.startDelay 
                          ? 'bg-yellow-500 text-white'
                          : 'bg-gray-300 text-gray-700'
                      }`}
                    >
                      {pump.running ? 'Running' : pump.startDelay ? `Starting (${Math.ceil(pump.startDelay)}s)` : 'Stopped'}
                    </button>
                  </div>
                  
                  {pump.inAlarm && (
                    <div className="flex items-center gap-2 text-red-600 text-sm mb-2">
                      <FaExclamationTriangle />
                      <span>{pump.alarmMessage}</span>
                    </div>
                  )}
                  
                  <div className="text-sm text-gray-600 space-y-1">
                    <div>Flow: {pump.currentFlow.toFixed(0)} / {pump.flowRate} gpm</div>
                    <div className="flex items-center gap-2">
                      <span>Setpoint:</span>
                      <input
                        type="number"
                        value={pump.flowRate}
                        onChange={(e) => updatePumpFlowRate(pump.id, parseInt(e.target.value))}
                        className="w-16 px-1 py-0.5 border rounded text-right text-xs"
                        min="0"
                        max={pump.maxFlowRate}
                        step="100"
                      />
                      <span className="text-xs text-gray-500">gpm</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <span>Efficiency:</span>
                      <input
                        type="number"
                        value={pump.efficiency}
                        onChange={(e) => updatePumpEfficiency(pump.id, parseInt(e.target.value))}
                        className="w-16 px-1 py-0.5 border rounded text-right text-xs"
                        min="0"
                        max="100"
                        step="5"
                      />
                      <span className="text-xs text-gray-500">%</span>
                    </div>
                    <div>Pressure: {pump.pressure} psi</div>
                  </div>
                </div>
              ))}
            </div>
          </div>
          
          {/* Hydrotank Status */}
          <div className="mb-6">
            <h3 className="font-semibold mb-3">Hydrotank Status</h3>
            <div className="space-y-3">
              {simulation.hydrotanks.map(ht => (
                <div key={ht.id} className="p-3 border rounded-lg">
                  <div className="font-medium mb-2">{ht.name}</div>
                  <div className="text-sm text-gray-600 space-y-1">
                    <div>Air Blanket: {ht.airBlanketPercent.toFixed(1)}%</div>
                    <div>Water Level: {ht.waterLevelPercent.toFixed(1)}%</div>
                    <div>Pressure: {ht.pressure} psi</div>
                    <div className="flex items-center gap-2">
                      <span>Compressor:</span>
                      <span className={`font-medium ${ht.compressorRunning ? 'text-green-600' : 'text-gray-600'}`}>
                        {ht.compressorRunning ? 'Running' : 'Stopped'}
                      </span>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
          
          {/* Active Alarms */}
          {simulation.activeAlarms.length > 0 && (
            <div className="mb-6">
              <h3 className="font-semibold mb-3 flex items-center gap-2">
                <FaExclamationTriangle className="text-red-600" />
                Active Alarms
              </h3>
              <div className="space-y-2 max-h-40 overflow-y-auto">
                {simulation.activeAlarms.map(alarm => (
                  <div key={alarm.id} className="text-sm p-2 bg-red-50 rounded border border-red-200">
                    <div className="font-medium">{alarm.source}</div>
                    <div className="text-red-600">{alarm.message}</div>
                    <div className="text-xs text-gray-500">{alarm.timestamp.toLocaleTimeString()}</div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
      
      {/* Toggle Controls Button */}
      <button
        onClick={() => setShowControls(!showControls)}
        className="absolute left-0 top-1/2 transform -translate-y-1/2 bg-white border border-gray-300 rounded-r-md px-2 py-4 shadow-md hover:bg-gray-50 z-10"
      >
        {showControls ? '◀' : '▶'}
      </button>
      
      {/* Main Display */}
      <div className="flex-1 relative">
        <Stage width={stageSize.width} height={stageSize.height}>
          <Layer>
            {/* Background */}
            <Rect
              x={0}
              y={0}
              width={stageSize.width}
              height={stageSize.height}
              fill="#f8fafc"
            />
            
            {/* Title */}
            <Text
              x={20}
              y={20}
              text="Water Treatment Plant - Real-Time Simulation"
              fontSize={24}
              fontStyle="bold"
              fill="#1e293b"
            />
            
            {/* Ground Storage Tank */}
            <Group x={100} y={200}>
              <TankComponent
                x={0}
                y={0}
                width={200}
                height={250}
                properties={{
                  maxLevel: 100,
                  currentLevel: simulation.tankLevelPercent,
                  alarmHigh: 90,
                  alarmLow: 20,
                  showLabel: true,
                  units: '%',
                  showWaveAnimation: true,
                }}
                bindings={[]}
                style={{
                  fill: '#e0f2fe',
                  stroke: '#0284c7',
                  strokeWidth: 3,
                }}
              />
              <Text
                x={0}
                y={-30}
                width={200}
                text="Ground Storage Tank"
                fontSize={16}
                fontStyle="bold"
                align="center"
                fill="#0284c7"
              />
              <Text
                x={0}
                y={260}
                width={200}
                text={`${(simulation.tankCapacity / 1000).toFixed(0)}k gal capacity`}
                fontSize={12}
                align="center"
                fill="#64748b"
              />
              <Text
                x={0}
                y={275}
                width={200}
                text={`Level: ${simulation.tankLevelFeet.toFixed(1)} ft`}
                fontSize={12}
                align="center"
                fill="#64748b"
              />
              <Text
                x={0}
                y={290}
                width={200}
                text={`Start: ${simulation.wellSetpoints.startLevel} ft | Stop: ${simulation.wellSetpoints.stopLevel} ft`}
                fontSize={10}
                align="center"
                fill="#94a3b8"
              />
            </Group>
            
            {/* Well Pump */}
            <Group x={100} y={520}>
              <PumpComponent
                x={0}
                y={0}
                width={80}
                height={80}
                properties={{
                  running: simulation.wellRunning,
                  fault: simulation.wellFlowRate > simulation.wellSetpoints.alarmHighFlow,
                  speed: simulation.wellRunning ? 100 : 0,
                  showStatus: true,
                  runAnimation: true,
                }}
                bindings={[]}
                style={{
                  fill: '#ffffff',
                  stroke: '#7c3aed',
                  strokeWidth: 2,
                }}
              />
              <Text
                x={-20}
                y={85}
                width={120}
                text="Well Pump"
                fontSize={14}
                align="center"
                fill="#7c3aed"
              />
              <Text
                x={-20}
                y={100}
                width={120}
                text={`${simulation.wellFlowRate} gpm`}
                fontSize={12}
                align="center"
                fill="#64748b"
              />
              
              {/* Pipe from well to tank */}
              <Line
                points={[40, 0, 40, -70]}
                stroke="#7c3aed"
                strokeWidth={6}
              />
            </Group>
            
            {/* Booster Pumps */}
            {simulation.boosterPumps.map((pump, index) => (
              <Group key={pump.id} x={400 + index * 150} y={350}>
                <PumpComponent
                  x={0}
                  y={0}
                  width={80}
                  height={80}
                  properties={{
                    running: pump.running,
                    fault: pump.inAlarm,
                    speed: pump.running ? (pump.efficiency / 100) * 100 : 0,
                    showStatus: true,
                    runAnimation: true,
                  }}
                  bindings={[]}
                  style={{
                    fill: '#ffffff',
                    stroke: pump.running ? '#16a34a' : '#6b7280',
                    strokeWidth: 2,
                  }}
                />
                <Text
                  x={-20}
                  y={-25}
                  width={120}
                  text={pump.name}
                  fontSize={14}
                  align="center"
                  fontStyle="bold"
                  fill={pump.running ? '#16a34a' : '#6b7280'}
                />
                {pump.pumpNumber === simulation.currentLeadPump && (
                  <Rect
                    x={30}
                    y={-45}
                    width={40}
                    height={16}
                    fill="#16a34a"
                    cornerRadius={3}
                  />
                )}
                {pump.pumpNumber === simulation.currentLeadPump && (
                  <Text
                    x={30}
                    y={-45}
                    width={40}
                    height={16}
                    text="LEAD"
                    fontSize={10}
                    fill="white"
                    align="center"
                    verticalAlign="middle"
                  />
                )}
                <Text
                  x={-20}
                  y={85}
                  width={120}
                  text={`${pump.currentFlow.toFixed(0)} gpm`}
                  fontSize={12}
                  align="center"
                  fill="#64748b"
                />
                <Text
                  x={-20}
                  y={100}
                  width={120}
                  text={`${pump.pressure} psi`}
                  fontSize={11}
                  align="center"
                  fill="#64748b"
                />
                
                {/* Pipe from pump to main */}
                <Line
                  points={[40, 80, 40, 120, 0, 120, 0, -50, -100 + index * 150, -50]}
                  stroke={pump.running ? '#16a34a' : '#6b7280'}
                  strokeWidth={4}
                />
              </Group>
            ))}
            
            {/* Main distribution pipe */}
            <Line
              points={[300, 300, 850, 300]}
              stroke="#0284c7"
              strokeWidth={8}
            />
            
            {/* Connect tank to distribution */}
            <Line
              points={[300, 325, 350, 325, 350, 300]}
              stroke="#0284c7"
              strokeWidth={6}
            />
            
            {/* Hydrotanks */}
            {simulation.hydrotanks.map((ht, index) => {
              // Ensure percentages are valid
              const waterHeight = Math.max(0, Math.min(174, (174 * ht.waterLevelPercent / 100)))
              const airHeight = Math.max(0, Math.min(174, (174 * ht.airBlanketPercent / 100)))
              
              return (
                <Group key={ht.id} x={900 + index * 180} y={320}>
                  {/* Tank body */}
                  <Rect
                    x={0}
                    y={0}
                    width={120}
                    height={180}
                    fill="#ffffff"
                    stroke="#0284c7"
                    strokeWidth={3}
                    cornerRadius={10}
                  />
                  
                  {/* Water level */}
                  <Rect
                    x={3}
                    y={180 - waterHeight - 3}
                    width={114}
                    height={waterHeight}
                    fill="#3b82f6"
                    opacity={0.3}
                    cornerRadius={8}
                  />
                  
                  {/* Air blanket indicator */}
                  <Rect
                    x={3}
                    y={3}
                    width={114}
                    height={airHeight - 6}
                    fill="#e0f2fe"
                    opacity={0.5}
                    cornerRadius={8}
                  />
                  
                  {/* Compressor */}
                  <Circle
                    x={60}
                    y={-30}
                    radius={15}
                    fill={ht.compressorRunning ? '#10b981' : '#e5e7eb'}
                    stroke="#374151"
                    strokeWidth={2}
                  />
                  <Text
                    x={60 - 5}
                    y={-35}
                    text="C"
                    fontSize={14}
                    fontStyle="bold"
                    fill="#374151"
                  />
                  
                  {/* Air line */}
                  <Line
                    points={[60, -15, 60, 0]}
                    stroke="#374151"
                    strokeWidth={3}
                  />
                  
                  {/* Labels */}
                  <Text
                    x={0}
                    y={-55}
                    width={120}
                    text={ht.name}
                    fontSize={14}
                    align="center"
                    fontStyle="bold"
                    fill="#0284c7"
                  />
                  
                  <Text
                    x={0}
                    y={190}
                    width={120}
                    text={`Air: ${ht.airBlanketPercent.toFixed(0)}%`}
                    fontSize={11}
                    align="center"
                    fill="#374151"
                  />
                  
                  <Text
                    x={0}
                    y={205}
                    width={120}
                    text={`${ht.pressure} psi`}
                    fontSize={11}
                    align="center"
                    fill="#374151"
                  />
                  
                  {/* Connection pipe */}
                  <Line
                    points={[60, 180, 60, 220, -50 + index * 180, 220, -50 + index * 180, 300]}
                    stroke="#0284c7"
                    strokeWidth={4}
                  />
                </Group>
              )
            })}
            
            {/* To distribution */}
            <Line
              points={[850, 300, 850, 200]}
              stroke="#0284c7"
              strokeWidth={8}
            />
            
            {/* System Pressure Gauge */}
            <Group x={750} y={50}>
              <GaugeComponent
                x={0}
                y={0}
                width={150}
                height={150}
                properties={{
                  min: 0,
                  max: 100,
                  value: simulation.systemPressure,
                  units: 'psi',
                  showTitle: true,
                  title: 'System Pressure',
                  alarmLow: simulation.pumpSetpoints.alarmLowPressure,
                  alarmHigh: simulation.pumpSetpoints.alarmHighPressure,
                }}
                bindings={[]}
                style={{
                  backgroundColor: '#ffffff',
                  borderColor: '#0284c7',
                  borderWidth: 2,
                }}
              />
            </Group>
            
            {/* Demand Indicator */}
            <Group x={950} y={50}>
              <Rect
                x={0}
                y={0}
                width={150}
                height={80}
                fill="#ffffff"
                stroke="#0284c7"
                strokeWidth={2}
                cornerRadius={5}
              />
              <Text
                x={0}
                y={10}
                width={150}
                text="System Demand"
                fontSize={14}
                fontStyle="bold"
                align="center"
                fill="#0284c7"
              />
              <Text
                x={0}
                y={35}
                width={150}
                text={`${simulation.demand.toLocaleString()}`}
                fontSize={24}
                fontStyle="bold"
                align="center"
                fill="#1e293b"
              />
              <Text
                x={0}
                y={60}
                width={150}
                text="gpm"
                fontSize={12}
                align="center"
                fill="#64748b"
              />
            </Group>
          </Layer>
        </Stage>
      </div>
    </div>
  )
}
