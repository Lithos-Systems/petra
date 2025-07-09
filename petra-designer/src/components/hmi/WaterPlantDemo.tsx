// src/components/hmi/WaterPlantDemo.tsx

import { useState, useEffect, useCallback } from 'react'
import { Stage, Layer, Group, Rect, Circle, Line, Text, Path } from 'react-konva'
import { FaPlay, FaPause, FaCog, FaExclamationTriangle, FaSync } from 'react-icons/fa'
import TankComponent from './components/TankComponent'
import PumpComponent from './components/PumpComponent'
import ValveComponent from './components/ValveComponent'
import GaugeComponent from './components/GaugeComponent'
import { usePetra } from '../../contexts/PetraContext'

interface SimulationState {
  running: boolean
  timeMultiplier: number
  petraConnected: boolean
  lastUpdate: Date | null
}

export default function WaterPlantPetraDemo() {
  const { connected, signals: petraSignals, setSignalValue } = usePetra()
  const isConnected = connected
  
  // Helper function to get signal value with fallback
  const getSignalValue = useCallback((signalName: string, defaultValue: any) => {
    const value = petraSignals.get(signalName)
    return value !== undefined ? value : defaultValue
  }, [petraSignals])
  
  // Debug logging
  useEffect(() => {
    console.log('PETRA Connection Status:', connected)
    console.log('PETRA Signals Map size:', petraSignals.size)
    console.log('Sample signals:', {
      tankLevel: getSignalValue('tank.level_feet', 12.5),
      pressure: getSignalValue('system.pressure', 60),
      pump1Running: getSignalValue('pump1.running', false)
    })
  }, [connected, petraSignals, getSignalValue])
  
  const [simulation, setSimulation] = useState<SimulationState>({
    running: true,
    timeMultiplier: 10,
    petraConnected: false,
    lastUpdate: null,
  })
  
  const [showControls, setShowControls] = useState(true)
  const [stageSize, setStageSize] = useState({ width: 800, height: 600 })
  
  // PETRA signal values (read from signal bus)
  const signals = {
    // Tank
    tankLevelFeet: Number(getSignalValue('tank.level_feet', 12.5)),
    tankLevelPercent: Number(getSignalValue('tank.level_percent', 50)),

    // Well pump
    wellRunning: Boolean(getSignalValue('well.running', false)),
    wellFlowRate: Number(getSignalValue('well.flow_rate', 0)),
    wellStartLevel: Number(getSignalValue('well.start_level', 8)),
    wellStopLevel: Number(getSignalValue('well.stop_level', 20)),
    wellSetpointFlow: Number(getSignalValue('well.setpoint_flow', 2000)),

    // System
    systemPressure: Number(getSignalValue('system.pressure', 60)),
    systemDemand: Number(getSignalValue('system.demand', 2500)),

    // Pumps
    pump1Running: Boolean(getSignalValue('pump1.running', false)),
    pump1FlowRate: Number(getSignalValue('pump1.flow_rate', 0)),
    pump1SetpointFlow: Number(getSignalValue('pump1.setpoint_flow', 1500)),
    pump1Efficiency: Number(getSignalValue('pump1.efficiency', 85)),
    pump1IsLead: Boolean(getSignalValue('pump1.is_lead', true)),

    pump2Running: Boolean(getSignalValue('pump2.running', false)),
    pump2FlowRate: Number(getSignalValue('pump2.flow_rate', 0)),
    pump2SetpointFlow: Number(getSignalValue('pump2.setpoint_flow', 1500)),
    pump2Efficiency: Number(getSignalValue('pump2.efficiency', 85)),
    pump2IsLead: Boolean(getSignalValue('pump2.is_lead', false)),

    pump3Running: Boolean(getSignalValue('pump3.running', false)),
    pump3FlowRate: Number(getSignalValue('pump3.flow_rate', 0)),
    pump3SetpointFlow: Number(getSignalValue('pump3.setpoint_flow', 1500)),
    pump3Efficiency: Number(getSignalValue('pump3.efficiency', 85)),
    pump3IsLead: Boolean(getSignalValue('pump3.is_lead', false)),

    // Hydrotanks
    hydrotank1WaterLevel: Number(getSignalValue('hydrotank1.water_level', 50)),
    hydrotank1AirBlanket: Number(getSignalValue('hydrotank1.air_blanket', 50)),

    hydrotank2WaterLevel: Number(getSignalValue('hydrotank2.water_level', 50)),
    hydrotank2AirBlanket: Number(getSignalValue('hydrotank2.air_blanket', 50)),

    // Valves
    dischargeValve1Open: Boolean(getSignalValue('valve.discharge1_open', true)),
    dischargeValve2Open: Boolean(getSignalValue('valve.discharge2_open', true)),
    dischargeValve3Open: Boolean(getSignalValue('valve.discharge3_open', true)),
    dischargeHeaderValve1: Boolean(getSignalValue('valve.header1_open', true)),
    dischargeHeaderValve2: Boolean(getSignalValue('valve.header2_open', true)),
    wellValveOpen: Boolean(getSignalValue('valve.well_open', true)),

    // Pressures
    dischargePressure1: Number(getSignalValue('pressure.discharge1', 62)),
    dischargePressure2: Number(getSignalValue('pressure.discharge2', 61)),
    dischargePressure3: Number(getSignalValue('pressure.discharge3', 60)),
    headerPressure1: Number(getSignalValue('pressure.header1', 59)),
    headerPressure2: Number(getSignalValue('pressure.header2', 58)),
    systemHeaderPressure: Number(getSignalValue('pressure.system_header', 55)),

    // Alarms
    alarmTankLow: Boolean(getSignalValue('alarm.tank_low', false)),
    alarmTankHigh: Boolean(getSignalValue('alarm.tank_high', false)),
    alarmPressureLow: Boolean(getSignalValue('alarm.pressure_low', false)),
    alarmPressureHigh: Boolean(getSignalValue('alarm.pressure_high', false)),

    // Lead pump rotation
    currentLeadPump: Math.round(Number(getSignalValue('lead.rotation_counter', 1))),
    
    // Total pump flow
    totalPumpFlow: Number(getSignalValue('pumps.total_flow', 0)),
  }
  
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
  
  // Simulation physics (updates PETRA signals)
  useEffect(() => {
    if (!simulation.running || !isConnected) return
    
    const interval = setInterval(() => {
      try {
        const timeStep = 1 / 60 * simulation.timeMultiplier
        
        // Read current values
        const tankLevel = Number(getSignalValue('tank.level_feet', 12.5))
        const wellFlow = Number(getSignalValue('well.flow_rate', 0))
        const totalPumpFlow = Number(getSignalValue('pumps.total_flow', 0))
        const systemDemand = Number(getSignalValue('system.demand', 2500))
        
        // Calculate net flow to tank
        const netFlow = wellFlow - totalPumpFlow
        
        // Update tank level (1 foot = 8000 gallons for a 200k gallon, 25ft tank)
        const gallonsPerFoot = 8000
        const levelChange = (netFlow * timeStep) / gallonsPerFoot
        const newLevel = Math.max(0, Math.min(25, tankLevel + levelChange))
        
        // Write updated tank level
        setSignalValue('tank.level_feet', newLevel)
        setSignalValue('tank.level_percent', (newLevel / 25) * 100)
        
        // Calculate system pressure based on pump flow vs demand
        const currentPressure = Number(getSignalValue('system.pressure', 60))
        let targetPressure = 0
        
        if (totalPumpFlow > 0) {
          const basePressure = 60
          const supplyDemandRatio = totalPumpFlow / Math.max(1, systemDemand)
          
          if (supplyDemandRatio >= 1) {
            targetPressure = Math.min(100, basePressure + (supplyDemandRatio - 1) * 20)
          } else {
            targetPressure = basePressure * supplyDemandRatio
          }
        }
        
        // Apply pressure change with hydrotank dampening
        const pressureChangeRate = 0.1
        let newPressure = currentPressure
        
        if (targetPressure > currentPressure) {
          newPressure = Math.min(targetPressure, currentPressure + (targetPressure - currentPressure) * pressureChangeRate)
        } else {
          newPressure = Math.max(targetPressure, currentPressure - (currentPressure - targetPressure) * pressureChangeRate)
        }
        
        setSignalValue('system.pressure', Math.round(newPressure))
        
        // Update hydrotank levels based on net system flow
        const netSystemFlow = totalPumpFlow - systemDemand
        const htFlowShare = netSystemFlow * timeStep / 2 // Split between two tanks
        
        // Hydrotank 1
        const ht1WaterLevel = Number(getSignalValue('hydrotank1.water_level', 50))
        const ht1NewLevel = Math.max(0, Math.min(100, ht1WaterLevel + htFlowShare / 50))
        setSignalValue('hydrotank1.water_level', ht1NewLevel)
        setSignalValue('hydrotank1.air_blanket', 100 - ht1NewLevel)
        
        // Hydrotank 2
        const ht2WaterLevel = Number(getSignalValue('hydrotank2.water_level', 50))
        const ht2NewLevel = Math.max(0, Math.min(100, ht2WaterLevel + htFlowShare / 50))
        setSignalValue('hydrotank2.water_level', ht2NewLevel)
        setSignalValue('hydrotank2.air_blanket', 100 - ht2NewLevel)
        
        // Update timestamp
        setSimulation(prev => ({ ...prev, lastUpdate: new Date() }))
        
      } catch (error) {
        console.error('Simulation error:', error)
      }
    }, 100)
    
    return () => clearInterval(interval)
  }, [simulation.running, simulation.timeMultiplier, isConnected, getSignalValue, setSignalValue])
  
  // Write signal to PETRA
  const writeSignal = (signal: string, value: any) => {
    if (!isConnected) {
      console.warn('Cannot write signal - not connected to PETRA')
      return
    }
    
    console.log(`Writing signal ${signal} = ${value}`)
    
    try {
      setSignalValue(signal, value)
      
      // Force a re-read to verify the write
      setTimeout(() => {
        const newValue = getSignalValue(signal, null)
        console.log(`Verified ${signal} = ${newValue}`)
      }, 100)
    } catch (error) {
      console.error(`Error writing signal ${signal}:`, error)
    }
  }
  
  // Control handlers
  const togglePump = (pumpNum: number) => {
    const signal = `pump${pumpNum}.running`
    const currentValue = Boolean(getSignalValue(signal, false))
    writeSignal(signal, !currentValue)
  }
  
  const toggleWellPump = () => {
    const currentValue = Boolean(getSignalValue('well.running', false))
    writeSignal('well.running', !currentValue)
  }
  
  const updateSetpoint = (signal: string, value: number) => {
    writeSignal(signal, value)
  }
  
  return (
    <div className="flex h-full bg-gray-50">
      {/* Control Panel */}
      {showControls && (
        <div className="w-96 bg-white border-r border-gray-200 p-4 overflow-y-auto">
          <h2 className="text-xl font-bold mb-4 flex items-center gap-2">
            <FaCog className="text-petra-600" />
            Water Plant Controls - PETRA Connected
          </h2>
          
          {/* Connection Status */}
          <div className={`mb-4 p-3 rounded-lg ${isConnected ? 'bg-green-50 border border-green-300' : 'bg-red-50 border border-red-300'}`}>
            <div className="flex items-center justify-between">
              <span className="font-semibold flex items-center gap-2">
                <FaSync className={isConnected ? 'text-green-600' : 'text-red-600'} />
                PETRA Connection
              </span>
              <span className={`text-sm ${isConnected ? 'text-green-600' : 'text-red-600'}`}>
                {isConnected ? 'Connected' : 'Disconnected'}
              </span>
            </div>
            {simulation.lastUpdate && (
              <div className="text-xs text-gray-600 mt-1">
                Last update: {simulation.lastUpdate.toLocaleTimeString()}
              </div>
            )}
          </div>
          
          {/* Simulation Control */}
          <div className="mb-6 p-4 bg-gray-50 rounded-lg">
            <div className="flex items-center justify-between mb-3">
              <h3 className="font-semibold">Simulation</h3>
              <button
                onClick={() => setSimulation(prev => ({ ...prev, running: !prev.running }))}
                className={`px-3 py-1 rounded flex items-center gap-2 text-white transition-colors ${
                  simulation.running ? 'bg-red-500 hover:bg-red-600' : 'bg-green-500 hover:bg-green-600'
                }`}
                disabled={!isConnected}
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
                value={signals.systemDemand}
                onChange={(e) => updateSetpoint('system.demand', parseInt(e.target.value))}
                className="w-full"
                disabled={!isConnected}
              />
              <div className="text-sm text-gray-500 mt-1">{signals.systemDemand.toLocaleString()} gpm</div>
            </div>
          </div>
          
          {/* System Status */}
          <div className="mb-6 p-4 bg-blue-50 rounded-lg">
            <h3 className="font-semibold mb-3">System Status</h3>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span>System Pressure:</span>
                <span className={`font-bold ${
                  signals.alarmPressureLow ? 'text-red-600' :
                  signals.alarmPressureHigh ? 'text-orange-600' :
                  'text-green-600'
                }`}>
                  {signals.systemPressure} psi
                </span>
              </div>
              <div className="flex justify-between">
                <span>Tank Level:</span>
                <span className={`font-medium ${
                  signals.alarmTankLow ? 'text-red-600' :
                  signals.alarmTankHigh ? 'text-orange-600' :
                  'text-green-600'
                }`}>
                  {signals.tankLevelFeet.toFixed(1)} ft ({signals.tankLevelPercent.toFixed(0)}%)
                </span>
              </div>
              <div className="flex justify-between">
                <span>Total Flow:</span>
                <span className="font-medium">{signals.totalPumpFlow.toLocaleString()} gpm</span>
              </div>
              <div className="flex justify-between">
                <span>Lead Pump:</span>
                <span className="font-medium">Pump {signals.currentLeadPump}</span>
              </div>
            </div>
          </div>
          
          {/* Pump Controls */}
          <div className="mb-6">
            <h3 className="font-semibold mb-3">Distribution Pumps</h3>
            
            {[1, 2, 3].map(num => (
              <div key={num} className="mb-4 p-3 bg-gray-50 rounded-lg">
                <div className="flex justify-between items-center mb-2">
                  <span className="font-medium flex items-center gap-2">
                    Pump {num}
                    {signals[`pump${num}IsLead` as keyof typeof signals] && (
                      <span className="text-xs bg-petra-600 text-white px-2 py-1 rounded">LEAD</span>
                    )}
                  </span>
                  <button
                    onClick={() => togglePump(num)}
                    className={`px-3 py-1 rounded text-sm font-medium transition-colors ${
                      signals[`pump${num}Running` as keyof typeof signals]
                        ? 'bg-green-500 text-white' 
                        : 'bg-gray-300 text-gray-700'
                    }`}
                    disabled={!isConnected}
                  >
                    {signals[`pump${num}Running` as keyof typeof signals] ? 'Running' : 'Stopped'}
                  </button>
                </div>
                
                <div className="flex justify-between items-center text-sm">
                  <span>Flow:</span>
                  <span className="font-medium">
                    {signals[`pump${num}FlowRate` as keyof typeof signals].toLocaleString()} gpm
                  </span>
                </div>
                <div className="flex justify-between items-center text-sm">
                  <span>Setpoint:</span>
                  <input
                    type="number"
                    value={signals[`pump${num}SetpointFlow` as keyof typeof signals]}
                    onChange={(e) => updateSetpoint(`pump${num}.setpoint_flow`, parseInt(e.target.value))}
                    className="w-20 px-2 py-1 border rounded text-right"
                    min="0"
                    max="3000"
                    step="100"
                    disabled={!isConnected}
                  />
                </div>
                <div className="flex justify-between items-center text-sm">
                  <span>Efficiency:</span>
                  <span className="font-medium">{signals[`pump${num}Efficiency` as keyof typeof signals]}%</span>
                </div>
              </div>
            ))}
          </div>
          
          {/* Well Pump Control */}
          <div className="mb-6 p-4 bg-blue-50 rounded-lg">
            <h3 className="font-semibold mb-3">Well Pump</h3>
            
            <div className="space-y-3">
              <div className="flex justify-between items-center">
                <span>Status:</span>
                <button
                  onClick={toggleWellPump}
                  className={`px-3 py-1 rounded text-sm font-medium transition-colors ${
                    signals.wellRunning
                      ? 'bg-green-500 text-white' 
                      : 'bg-gray-300 text-gray-700'
                  }`}
                  disabled={!isConnected}
                >
                  {signals.wellRunning ? 'Running' : 'Stopped'}
                </button>
              </div>
              
              <div className="flex justify-between items-center">
                <span>Start Level (ft):</span>
                <input
                  type="number"
                  value={signals.wellStartLevel}
                  onChange={(e) => updateSetpoint('well.start_level', parseFloat(e.target.value))}
                  className="w-20 px-2 py-1 border rounded text-right"
                  min="0"
                  max="25"
                  step="0.5"
                  disabled={!isConnected}
                />
              </div>
              <div className="flex justify-between items-center">
                <span>Stop Level (ft):</span>
                <input
                  type="number"
                  value={signals.wellStopLevel}
                  onChange={(e) => updateSetpoint('well.stop_level', parseFloat(e.target.value))}
                  className="w-20 px-2 py-1 border rounded text-right"
                  min="0"
                  max="25"
                  step="0.5"
                  disabled={!isConnected}
                />
              </div>
              <div className="flex justify-between items-center">
                <span>Flow Rate (gpm):</span>
                <input
                  type="number"
                  value={signals.wellSetpointFlow}
                  onChange={(e) => updateSetpoint('well.setpoint_flow', parseInt(e.target.value))}
                  className="w-20 px-2 py-1 border rounded text-right"
                  min="0"
                  max="5000"
                  step="100"
                  disabled={!isConnected}
                />
              </div>
              <div className="mt-2 p-2 bg-gray-100 rounded">
                <div className="text-xs text-gray-600">
                  Current: {signals.wellRunning ? `${signals.wellFlowRate.toLocaleString()} gpm` : 'Off'}
                </div>
              </div>
            </div>
          </div>
          
          {/* Alarms */}
          {(signals.alarmTankLow || signals.alarmTankHigh || signals.alarmPressureLow || signals.alarmPressureHigh) && (
            <div className="mb-6 p-4 bg-red-50 border border-red-300 rounded-lg">
              <h3 className="font-semibold text-red-700 mb-2 flex items-center gap-2">
                <FaExclamationTriangle />
                Active Alarms
              </h3>
              <div className="space-y-1 text-sm text-red-600">
                {signals.alarmTankLow && <div>• Tank Level Low</div>}
                {signals.alarmTankHigh && <div>• Tank Level High</div>}
                {signals.alarmPressureLow && <div>• System Pressure Low</div>}
                {signals.alarmPressureHigh && <div>• System Pressure High</div>}
              </div>
            </div>
          )}
        </div>
      )}
      
      {/* Main Stage */}
      <div className="flex-1 relative">
        {/* Control Toggle Button */}
        <button
          onClick={() => setShowControls(!showControls)}
          className="absolute top-4 left-4 z-10 bg-white rounded-lg px-3 py-2 shadow-md flex items-center gap-2 hover:bg-gray-50"
        >
          <FaCog />
          {showControls ? 'Hide' : 'Show'} Controls
        </button>
        
        <Stage width={stageSize.width} height={stageSize.height}>
          <Layer>
            {/* Background */}
            <Rect
              x={0}
              y={0}
              width={stageSize.width}
              height={stageSize.height}
              fill="#f9fafb"
            />
            
            {/* Ground Storage Tank */}
            <Group x={100} y={100}>
              <TankComponent
                x={0}
                y={0}
                width={200}
                height={300}
                properties={{
                  currentLevel: signals.tankLevelPercent,
                  maxLevel: 100,
                  minLevel: 0,
                  units: '%',
                  fillColor: '#3b82f6',
                  showLabel: true,
                  label: 'Ground Storage Tank',
                  alarmHigh: 90,
                  alarmLow: 10,
                  isMetric: false
                }}
                style={{
                  fill: '#f3f4f6',
                  stroke: '#374151',
                  strokeWidth: 2
                }}
                bindings={[]}
              />
              {/* Tank annotations */}
              <Text
                x={0}
                y={-30}
                width={200}
                text="Ground Storage Tank"
                fontSize={16}
                fontStyle="bold"
                align="center"
                fill="#374151"
              />
              <Text
                x={0}
                y={310}
                width={200}
                text={`${signals.tankLevelFeet.toFixed(1)} ft (${signals.tankLevelPercent.toFixed(0)}%)`}
                fontSize={14}
                align="center"
                fill="#6b7280"
              />
            </Group>
            
            {/* Well connection */}
            <Group>
              <Line
                points={[300, 250, 380, 250]}
                stroke="#4B5563"
                strokeWidth={6}
                lineCap="round"
              />
              <ValveComponent
                x={380}
                y={230}
                width={40}
                height={40}
                properties={{
                  open: signals.wellValveOpen,
                  position: signals.wellValveOpen ? 100 : 0,
                  showPosition: false,
                  fault: false
                }}
                style={{
                  fill: '#6b7280',
                  stroke: '#374151',
                  strokeWidth: 2
                }}
              />
              <Line
                points={[420, 250, 500, 250]}
                stroke="#4B5563"
                strokeWidth={6}
                lineCap="round"
              />
              {/* Well pump */}
              <PumpComponent
                x={500}
                y={210}
                width={80}
                height={80}
                properties={{
                  running: signals.wellRunning,
                  fault: false,
                  speed: signals.wellRunning ? 100 : 0,
                  showStatus: true,
                  runAnimation: true
                }}
                style={{
                  fill: signals.wellRunning ? '#10b981' : '#e5e7eb',
                  stroke: '#374151',
                  strokeWidth: 2
                }}
                bindings={[]}
              />
              <Text
                x={500}
                y={295}
                width={80}
                text="Well Pump"
                fontSize={12}
                align="center"
                fill="#374151"
              />
              <Text
                x={500}
                y={310}
                width={80}
                text={`${signals.wellFlowRate} gpm`}
                fontSize={11}
                align="center"
                fill="#6b7280"
              />
              <Line
                points={[580, 250, 660, 250, 660, 350, 200, 350, 200, 400]}
                stroke="#4B5563"
                strokeWidth={6}
                lineCap="round"
              />
            </Group>
            
            {/* Distribution pumps */}
            {[1, 2, 3].map((num, index) => (
              <Group key={num}>
                <Line
                  points={[200, 250 - index * 60, 500, 250 - index * 60]}
                  stroke="#4B5563"
                  strokeWidth={6}
                  lineCap="round"
                />
                <PumpComponent
                  x={500}
                  y={230 - index * 60}
                  width={80}
                  height={80}
                  properties={{
                    running: signals[`pump${num}Running` as keyof typeof signals] as boolean,
                    fault: false,
                    speed: signals[`pump${num}Running` as keyof typeof signals] ? 100 : 0,
                    showStatus: true,
                    runAnimation: true
                  }}
                  style={{
                    fill: signals[`pump${num}Running` as keyof typeof signals] ? '#10b981' : '#e5e7eb',
                    stroke: signals[`pump${num}IsLead` as keyof typeof signals] ? '#f59e0b' : '#374151',
                    strokeWidth: signals[`pump${num}IsLead` as keyof typeof signals] ? 3 : 2
                  }}
                  bindings={[]}
                />
                <Text
                  x={500}
                  y={315 - index * 60}
                  width={80}
                  text={`Pump ${num}`}
                  fontSize={12}
                  align="center"
                  fill="#374151"
                  fontStyle={signals[`pump${num}IsLead` as keyof typeof signals] ? 'bold' : 'normal'}
                />
                <Text
                  x={500}
                  y={330 - index * 60}
                  width={80}
                  text={`${signals[`pump${num}FlowRate` as keyof typeof signals]} gpm`}
                  fontSize={11}
                  align="center"
                  fill="#6b7280"
                />
                <Line
                  points={[580, 250 - index * 60, 620, 250 - index * 60]}
                  stroke="#4B5563"
                  strokeWidth={6}
                  lineCap="round"
                />
                <ValveComponent
                  x={620}
                  y={230 - index * 60}
                  width={40}
                  height={40}
                  properties={{
                    open: signals[`dischargeValve${num}Open` as keyof typeof signals] as boolean,
                    position: 100,
                    showPosition: false,
                    fault: false
                  }}
                  style={{
                    fill: '#6b7280',
                    stroke: '#374151',
                    strokeWidth: 2
                  }}
                />
                <Line
                  points={[660, 250 - index * 60, 700, 250 - index * 60]}
                  stroke="#4B5563"
                  strokeWidth={6}
                  lineCap="round"
                />
                <GaugeComponent
                  x={720}
                  y={230 - index * 60}
                  width={40}
                  height={40}
                  properties={{
                    value: signals[`dischargePressure${num}` as keyof typeof signals] as number,
                    min: 0,
                    max: 100,
                    units: 'psi',
                    label: `P${num}`,
                    showDigital: true,
                    alarmLow: 30,
                    alarmHigh: 80,
                    warningLow: 40,
                    warningHigh: 70
                  }}
                  style={{
                    fill: '#ffffff',
                    stroke: '#374151',
                    strokeWidth: 2,
                    needleColor: '#ef4444',
                    textColor: '#374151'
                  }}
                />
                <Line
                  points={[780, 250 - index * 60, 820, 250 - index * 60]}
                  stroke="#4B5563"
                  strokeWidth={6}
                  lineCap="round"
                />
              </Group>
            ))}
            
            {/* Discharge headers */}
            <Group>
              {/* Header 1 - Pumps 1&2 */}
              <Line
                points={[820, 250, 820, 200, 820, 190, 900, 190]}
                stroke="#4B5563"
                strokeWidth={6}
                lineCap="round"
              />
              <ValveComponent
                x={900}
                y={170}
                width={40}
                height={40}
                properties={{
                  open: signals.dischargeHeaderValve1,
                  position: 100,
                  showPosition: false,
                  fault: false
                }}
                style={{
                  fill: '#6b7280',
                  stroke: '#374151',
                  strokeWidth: 2
                }}
              />
              <Line
                points={[940, 190, 980, 190]}
                stroke="#4B5563"
                strokeWidth={6}
                lineCap="round"
              />
              <GaugeComponent
                x={1000}
                y={170}
                width={40}
                height={40}
                properties={{
                  value: signals.headerPressure1,
                  min: 0,
                  max: 100,
                  units: 'psi',
                  label: 'HP1',
                  showDigital: true,
                  alarmLow: 30,
                  alarmHigh: 80,
                  warningLow: 40,
                  warningHigh: 70
                }}
                style={{
                  fill: '#ffffff',
                  stroke: '#374151',
                  strokeWidth: 2,
                  needleColor: '#ef4444',
                  textColor: '#374151'
                }}
              />
              
              {/* Header 2 - All pumps */}
              <Line
                points={[820, 130, 820, 100, 900, 100]}
                stroke="#4B5563"
                strokeWidth={6}
                lineCap="round"
              />
              <ValveComponent
                x={900}
                y={80}
                width={40}
                height={40}
                properties={{
                  open: signals.dischargeHeaderValve2,
                  position: 100,
                  showPosition: false,
                  fault: false
                }}
                style={{
                  fill: '#6b7280',
                  stroke: '#374151',
                  strokeWidth: 2
                }}
              />
              <Line
                points={[940, 100, 980, 100]}
                stroke="#4B5563"
                strokeWidth={6}
                lineCap="round"
              />
              <GaugeComponent
                x={1000}
                y={80}
                width={40}
                height={40}
                properties={{
                  value: signals.headerPressure2,
                  min: 0,
                  max: 100,
                  units: 'psi',
                  label: 'HP2',
                  showDigital: true,
                  alarmLow: 30,
                  alarmHigh: 80,
                  warningLow: 40,
                  warningHigh: 70
                }}
                style={{
                  fill: '#ffffff',
                  stroke: '#374151',
                  strokeWidth: 2,
                  needleColor: '#ef4444',
                  textColor: '#374151'
                }}
              />
            </Group>
            
            {/* Hydrotanks */}
            <Group x={900} y={300}>
              <TankComponent
                x={0}
                y={0}
                width={80}
                height={150}
                properties={{
                  currentLevel: signals.hydrotank1WaterLevel,
                  maxLevel: 100,
                  minLevel: 0,
                  units: '%',
                  fillColor: '#60a5fa',
                  showLabel: true,
                  label: 'HT-1',
                  alarmHigh: 80,
                  alarmLow: 20,
                  isMetric: false
                }}
                style={{
                  fill: '#e0e7ff',
                  stroke: '#4338ca',
                  strokeWidth: 2
                }}
                bindings={[]}
              />
              <TankComponent
                x={100}
                y={0}
                width={80}
                height={150}
                properties={{
                  currentLevel: signals.hydrotank2WaterLevel,
                  maxLevel: 100,
                  minLevel: 0,
                  units: '%',
                  fillColor: '#60a5fa',
                  showLabel: true,
                  label: 'HT-2',
                  alarmHigh: 80,
                  alarmLow: 20,
                  isMetric: false
                }}
                style={{
                  fill: '#e0e7ff',
                  stroke: '#4338ca',
                  strokeWidth: 2
                }}
                bindings={[]}
              />
            </Group>
            
            {/* Main system header */}
            <Group>
              <Line
                points={[1060, 190, 1100, 190, 1100, 300, 1100, 375, 980, 375]}
                stroke="#4B5563"
                strokeWidth={8}
                lineCap="round"
              />
              <Line
                points={[1060, 100, 1100, 100, 1100, 190]}
                stroke="#4B5563"
                strokeWidth={8}
                lineCap="round"
              />
              
              {/* System pressure gauge */}
              <GaugeComponent
                x={1150}
                y={230}
                width={80}
                height={80}
                properties={{
                  value: signals.systemPressure,
                  min: 0,
                  max: 100,
                  units: 'psi',
                  label: 'System',
                  showDigital: true,
                  alarmLow: 30,
                  alarmHigh: 80,
                  warningLow: 40,
                  warningHigh: 70
                }}
                style={{
                  fill: '#ffffff',
                  stroke: signals.alarmPressureLow || signals.alarmPressureHigh ? '#ef4444' : '#374151',
                  strokeWidth: 3,
                  needleColor: '#ef4444',
                  textColor: '#374151'
                }}
              />
              
              {/* Distribution arrow */}
              <Group x={1100} y={450}>
                <Line
                  points={[0, 0, 0, 50]}
                  stroke="#4B5563"
                  strokeWidth={8}
                  lineCap="round"
                />
                <Path
                  data="M -10 50 L 10 50 L 0 70 Z"
                  fill="#4B5563"
                />
                <Text
                  x={-50}
                  y={80}
                  text="To Distribution"
                  fontSize={14}
                  fill="#374151"
                  align="center"
                  width={100}
                />
                <Text
                  x={-50}
                  y={100}
                  text={`${signals.systemDemand.toLocaleString()} gpm`}
                  fontSize={12}
                  fill="#6B7280"
                  align="center"
                  width={100}
                />
              </Group>
            </Group>
          </Layer>
        </Stage>
      </div>
    </div>
  )
}
