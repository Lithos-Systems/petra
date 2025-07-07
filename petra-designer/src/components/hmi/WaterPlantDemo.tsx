// src/components/hmi/WaterPlantDemo.tsx

import { useState, useEffect, useRef } from 'react'
import { Stage, Layer, Group, Rect, Circle, Line, Text, Path } from 'react-konva'
import { FaPlay, FaPause, FaCog, FaChartLine, FaExclamationTriangle } from 'react-icons/fa'
import TankComponent from './components/TankComponent'
import PumpComponent from './components/PumpComponent'
import ValveComponent from './components/ValveComponent'
import GaugeComponent from './components/GaugeComponent'

interface PumpSetpoints {
  startLevel: number // Tank level % to start
  stopLevel: number // Tank level % to stop
  alarmLowPressure: number // Low pressure alarm (psi)
  alarmHighPressure: number // High pressure alarm (psi)
  alarmHighFlow: number // High flow alarm (gpm)
}

interface SimulationState {
  // Tank parameters
  tankCapacity: number // gallons
  tankLevel: number // gallons
  tankLevelPercent: number // %
  
  // Well parameters
  wellFlowRate: number // gpm
  wellRunning: boolean
  wellSetpoints: PumpSetpoints
  
  // Pump parameters
  boosterPumps: Array<{
    id: string
    name: string
    flowRate: number // gpm
    running: boolean
    efficiency: number // %
    pressure: number // psi
    setpoints: PumpSetpoints
    inAlarm: boolean
    alarmMessage?: string
  }>
  
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
    wellFlowRate: 2000,
    wellRunning: true,
    wellSetpoints: {
      startLevel: 30,
      stopLevel: 80,
      alarmLowPressure: 0,
      alarmHighPressure: 100,
      alarmHighFlow: 3000,
    },
    boosterPumps: [
      { 
        id: 'bp1', 
        name: 'Booster 1', 
        flowRate: 1500, 
        running: true, 
        efficiency: 85, 
        pressure: 60,
        setpoints: {
          startLevel: 40,
          stopLevel: 70,
          alarmLowPressure: 40,
          alarmHighPressure: 80,
          alarmHighFlow: 2000,
        },
        inAlarm: false
      },
      { 
        id: 'bp2', 
        name: 'Booster 2', 
        flowRate: 1500, 
        running: false, 
        efficiency: 85, 
        pressure: 60,
        setpoints: {
          startLevel: 35,
          stopLevel: 75,
          alarmLowPressure: 40,
          alarmHighPressure: 80,
          alarmHighFlow: 2000,
        },
        inAlarm: false
      },
      { 
        id: 'bp3', 
        name: 'Booster 3', 
        flowRate: 2000, 
        running: false, 
        efficiency: 90, 
        pressure: 65,
        setpoints: {
          startLevel: 30,
          stopLevel: 80,
          alarmLowPressure: 45,
          alarmHighPressure: 85,
          alarmHighFlow: 2500,
        },
        inAlarm: false
      },
    ],
    demand: 2500,
    systemPressure: 55,
    targetPressure: 60,
    running: true,
    timeMultiplier: 10,
    activeAlarms: [],
  })
  
  const [showControls, setShowControls] = useState(true)
  const [selectedPump, setSelectedPump] = useState<string | null>(null)
  const simulationRef = useRef<SimulationState>(simulation)
  simulationRef.current = simulation
  
  // Check alarms
  const checkAlarms = (state: SimulationState) => {
    const newAlarms: typeof state.activeAlarms = []
    const updatedPumps = state.boosterPumps.map(pump => {
      const pumpAlarms: string[] = []
      
      if (pump.running) {
        if (pump.pressure < pump.setpoints.alarmLowPressure) {
          pumpAlarms.push(`Low pressure: ${pump.pressure} psi`)
        }
        if (pump.pressure > pump.setpoints.alarmHighPressure) {
          pumpAlarms.push(`High pressure: ${pump.pressure} psi`)
        }
        if (pump.flowRate > pump.setpoints.alarmHighFlow) {
          pumpAlarms.push(`High flow: ${pump.flowRate} gpm`)
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
        
        // Automatic well control based on tank level
        let wellRunning = prev.wellRunning
        if (prev.tankLevelPercent <= prev.wellSetpoints.startLevel) {
          wellRunning = true
        } else if (prev.tankLevelPercent >= prev.wellSetpoints.stopLevel) {
          wellRunning = false
        }
        
        // Automatic pump control based on setpoints
        const updatedPumps = prev.boosterPumps.map(pump => {
          let running = pump.running
          if (prev.tankLevelPercent <= pump.setpoints.startLevel && prev.systemPressure < prev.targetPressure) {
            running = true
          } else if (prev.tankLevelPercent >= pump.setpoints.stopLevel || prev.systemPressure > prev.targetPressure * 1.1) {
            running = false
          }
          return { ...pump, running }
        })
        
        // Calculate total pump output
        const totalPumpOutput = updatedPumps
          .filter(p => p.running)
          .reduce((sum, pump) => sum + pump.flowRate * (pump.efficiency / 100), 0)
        
        // Calculate net flow (well + pumps - demand)
        const wellFlow = wellRunning ? prev.wellFlowRate : 0
        const netFlow = wellFlow - prev.demand + totalPumpOutput
        
        // Update tank level
        const newLevel = Math.max(0, Math.min(prev.tankCapacity, prev.tankLevel + (netFlow * timeStep)))
        const newLevelPercent = (newLevel / prev.tankCapacity) * 100
        
        // Calculate system pressure based on running pumps and tank level
        const runningPumps = updatedPumps.filter(p => p.running)
        let newPressure = 0
        
        if (runningPumps.length > 0) {
          // Average pressure from running pumps, adjusted by tank level
          const avgPumpPressure = runningPumps.reduce((sum, p) => sum + p.pressure, 0) / runningPumps.length
          const tankLevelFactor = 0.5 + (newLevelPercent / 100) * 0.5 // 50-100% efficiency based on tank level
          newPressure = avgPumpPressure * tankLevelFactor
          
          // Adjust for demand
          const demandRatio = prev.demand / totalPumpOutput
          if (demandRatio > 1) {
            // Demand exceeds supply, pressure drops
            newPressure *= Math.max(0.5, 1 - (demandRatio - 1) * 0.3)
          }
        }
        
        // Check alarms
        const { updatedPumps: alarmedPumps, newAlarms } = checkAlarms({
          ...prev,
          boosterPumps: updatedPumps,
          wellRunning,
          systemPressure: Math.round(newPressure),
        })
        
        // Keep only last 10 alarms
        const allAlarms = [...prev.activeAlarms, ...newAlarms].slice(-10)
        
        return {
          ...prev,
          wellRunning,
          boosterPumps: alarmedPumps,
          tankLevel: newLevel,
          tankLevelPercent: newLevelPercent,
          systemPressure: Math.round(newPressure),
          activeAlarms: allAlarms,
        }
      })
    }, 100) // Update every 100ms
    
    return () => clearInterval(interval)
  }, [simulation.running, simulation.timeMultiplier])
  
  const togglePump = (pumpId: string) => {
    setSimulation(prev => ({
      ...prev,
      boosterPumps: prev.boosterPumps.map(pump =>
        pump.id === pumpId ? { ...pump, running: !pump.running } : pump
      )
    }))
  }
  
  const updatePumpSetpoint = (pumpId: string, field: keyof PumpSetpoints, value: number) => {
    setSimulation(prev => ({
      ...prev,
      boosterPumps: prev.boosterPumps.map(pump =>
        pump.id === pumpId 
          ? { ...pump, setpoints: { ...pump.setpoints, [field]: value } }
          : pump
      )
    }))
  }
  
  const updateWellSetpoint = (field: keyof PumpSetpoints, value: number) => {
    setSimulation(prev => ({
      ...prev,
      wellSetpoints: { ...prev.wellSetpoints, [field]: value }
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
                <span>Target Pressure:</span>
                <input
                  type="number"
                  value={simulation.targetPressure}
                  onChange={(e) => setSimulation(prev => ({ ...prev, targetPressure: parseInt(e.target.value) }))}
                  className="w-20 px-2 py-1 border rounded text-right"
                />
              </div>
              <div className="flex justify-between">
                <span>Current Demand:</span>
                <span className="font-bold">{simulation.demand.toLocaleString()} gpm</span>
              </div>
              <div className="flex justify-between">
                <span>Tank Level:</span>
                <span className="font-bold">{simulation.tankLevelPercent.toFixed(1)}%</span>
              </div>
            </div>
          </div>
          
          {/* Well Setpoints */}
          <div className="mb-6">
            <h3 className="font-semibold mb-3">Well Pump Setpoints</h3>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between items-center">
                <span>Start Level (%):</span>
                <input
                  type="number"
                  value={simulation.wellSetpoints.startLevel}
                  onChange={(e) => updateWellSetpoint('startLevel', parseInt(e.target.value))}
                  className="w-20 px-2 py-1 border rounded text-right"
                  min="0"
                  max="100"
                />
              </div>
              <div className="flex justify-between items-center">
                <span>Stop Level (%):</span>
                <input
                  type="number"
                  value={simulation.wellSetpoints.stopLevel}
                  onChange={(e) => updateWellSetpoint('stopLevel', parseInt(e.target.value))}
                  className="w-20 px-2 py-1 border rounded text-right"
                  min="0"
                  max="100"
                />
              </div>
              <div className="flex justify-between items-center">
                <span>High Flow Alarm (gpm):</span>
                <input
                  type="number"
                  value={simulation.wellSetpoints.alarmHighFlow}
                  onChange={(e) => updateWellSetpoint('alarmHighFlow', parseInt(e.target.value))}
                  className="w-20 px-2 py-1 border rounded text-right"
                />
              </div>
            </div>
          </div>
          
          {/* Booster Pump Controls */}
          <div className="mb-6">
            <h3 className="font-semibold mb-3">Booster Pumps</h3>
            <div className="space-y-3">
              {simulation.boosterPumps.map(pump => (
                <div 
                  key={pump.id} 
                  className={`p-3 border rounded-lg ${
                    pump.inAlarm ? 'border-red-500 bg-red-50' : 'border-gray-200'
                  }`}
                >
                  <div className="flex items-center justify-between mb-2">
                    <span className="font-medium">{pump.name}</span>
                    <button
                      onClick={() => togglePump(pump.id)}
                      className={`px-3 py-1 rounded text-sm ${
                        pump.running 
                          ? 'bg-green-500 text-white' 
                          : 'bg-gray-300 text-gray-700'
                      }`}
                    >
                      {pump.running ? 'Running' : 'Stopped'}
                    </button>
                  </div>
                  
                  {pump.inAlarm && (
                    <div className="flex items-center gap-2 text-red-600 text-sm mb-2">
                      <FaExclamationTriangle />
                      <span>{pump.alarmMessage}</span>
                    </div>
                  )}
                  
                  <button
                    onClick={() => setSelectedPump(selectedPump === pump.id ? null : pump.id)}
                    className="text-sm text-blue-600 hover:text-blue-800"
                  >
                    {selectedPump === pump.id ? 'Hide' : 'Show'} Setpoints
                  </button>
                  
                  {selectedPump === pump.id && (
                    <div className="mt-2 pt-2 border-t space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span>Start Level (%):</span>
                        <input
                          type="number"
                          value={pump.setpoints.startLevel}
                          onChange={(e) => updatePumpSetpoint(pump.id, 'startLevel', parseInt(e.target.value))}
                          className="w-16 px-1 py-0.5 border rounded text-right"
                          min="0"
                          max="100"
                        />
                      </div>
                      <div className="flex justify-between">
                        <span>Stop Level (%):</span>
                        <input
                          type="number"
                          value={pump.setpoints.stopLevel}
                          onChange={(e) => updatePumpSetpoint(pump.id, 'stopLevel', parseInt(e.target.value))}
                          className="w-16 px-1 py-0.5 border rounded text-right"
                          min="0"
                          max="100"
                        />
                      </div>
                      <div className="flex justify-between">
                        <span>Low Pressure (psi):</span>
                        <input
                          type="number"
                          value={pump.setpoints.alarmLowPressure}
                          onChange={(e) => updatePumpSetpoint(pump.id, 'alarmLowPressure', parseInt(e.target.value))}
                          className="w-16 px-1 py-0.5 border rounded text-right"
                        />
                      </div>
                      <div className="flex justify-between">
                        <span>High Pressure (psi):</span>
                        <input
                          type="number"
                          value={pump.setpoints.alarmHighPressure}
                          onChange={(e) => updatePumpSetpoint(pump.id, 'alarmHighPressure', parseInt(e.target.value))}
                          className="w-16 px-1 py-0.5 border rounded text-right"
                        />
                      </div>
                      <div className="flex justify-between">
                        <span>High Flow (gpm):</span>
                        <input
                          type="number"
                          value={pump.setpoints.alarmHighFlow}
                          onChange={(e) => updatePumpSetpoint(pump.id, 'alarmHighFlow', parseInt(e.target.value))}
                          className="w-16 px-1 py-0.5 border rounded text-right"
                        />
                      </div>
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>
          
          {/* Demand Control */}
          <div className="mb-6">
            <h3 className="font-semibold mb-3">Demand Control</h3>
            <label className="text-sm text-gray-600">Demand (gpm)</label>
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
          
          {/* Active Alarms */}
          {simulation.activeAlarms.length > 0 && (
            <div className="mb-6 p-4 bg-red-50 rounded-lg">
              <h3 className="font-semibold mb-2 text-red-700">Active Alarms</h3>
              <div className="space-y-1 text-sm">
                {simulation.activeAlarms.slice(-5).reverse().map(alarm => (
                  <div key={alarm.id} className="text-red-600">
                    <span className="font-medium">{alarm.source}:</span> {alarm.message}
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
      
      {/* Toggle Control Panel Button */}
      <button
        onClick={() => setShowControls(!showControls)}
        className="absolute left-0 top-1/2 transform -translate-y-1/2 bg-white border border-gray-300 rounded-r-lg px-2 py-4 shadow-lg hover:bg-gray-50 z-10"
      >
        {showControls ? '◀' : '▶'}
      </button>
      
      {/* Main Display */}
      <div className="flex-1 relative">
        <Stage width={window.innerWidth - (showControls ? 384 : 0)} height={window.innerHeight - 60}>
          <Layer>
            {/* Background */}
            <Rect
              x={0}
              y={0}
              width={window.innerWidth - (showControls ? 384 : 0)}
              height={window.innerHeight - 60}
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
                  fill: '#ddd6fe',
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
                    fill: pump.running ? '#dcfce7' : '#f3f4f6',
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
                <Text
                  x={-20}
                  y={85}
                  width={120}
                  text={`${pump.flowRate} gpm`}
                  fontSize={12}
                  align="center"
                  fill="#64748b"
                />
                <Text
                  x={-20}
                  y={100}
                  width={120}
                  text={`${pump.pressure} psi`}
                  fontSize={12}
                  align="center"
                  fill="#64748b"
                />
                
                {/* Inlet pipe */}
                <Line
                  points={[40, 80, 40, 120, -100 + index * 150, 120]}
                  stroke="#6b7280"
                  strokeWidth={4}
                />
                
                {/* Outlet pipe */}
                <Line
                  points={[40, 0, 40, -50]}
                  stroke={pump.running ? '#16a34a' : '#6b7280'}
                  strokeWidth={4}
                />
              </Group>
            ))}
            
            {/* Main distribution pipe */}
            <Line
              points={[300, 300, 850, 300, 850, 200]}
              stroke="#0284c7"
              strokeWidth={8}
            />
            
            {/* Connect tank to distribution */}
            <Line
              points={[300, 325, 350, 325, 350, 300]}
              stroke="#0284c7"
              strokeWidth={6}
            />
            
            {/* System Pressure Gauge */}
            <Group x={900} y={100}>
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
                  showScale: true,
                  majorTicks: 5,
                }}
                bindings={[]}
                style={{
                  fill: '#ffffff',
                  stroke: '#0284c7',
                  strokeWidth: 3,
                }}
              />
              <Text
                x={0}
                y={155}
                width={150}
                text="System Pressure"
                fontSize={14}
                align="center"
                fill="#0284c7"
              />
            </Group>
            
            {/* Demand indicator */}
            <Group x={900} y={300}>
              <Text
                x={0}
                y={0}
                text="Current Demand"
                fontSize={16}
                fontStyle="bold"
                fill="#0284c7"
              />
              <Text
                x={0}
                y={25}
                text={`${simulation.demand.toLocaleString()} gpm`}
                fontSize={24}
                fontStyle="bold"
                fill="#0284c7"
              />
            </Group>
          </Layer>
        </Stage>
      </div>
    </div>
  )
}
