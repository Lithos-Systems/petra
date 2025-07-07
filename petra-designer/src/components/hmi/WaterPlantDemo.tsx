// src/components/hmi/WaterPlantPetraDemo.tsx

import { useState, useEffect, useRef } from 'react'
import { Stage, Layer, Group, Rect, Circle, Line, Text, Path } from 'react-konva'
import { FaPlay, FaPause, FaCog, FaChartLine, FaExclamationTriangle, FaSync } from 'react-icons/fa'
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
  const { signalBus, isConnected } = usePetra()
  
  const [simulation, setSimulation] = useState<SimulationState>({
    running: true,
    timeMultiplier: 10,
    petraConnected: false,
    lastUpdate: null,
  })
  
  const [showControls, setShowControls] = useState(true)
  const [stageSize, setStageSize] = useState({ width: 800, height: 600 })
  
  // PETRA signal values (read from signal bus)
  const [signals, setSignals] = useState({
    // Tank
    tankLevelFeet: 12.5,
    tankLevelPercent: 50,
    
    // Well pump
    wellRunning: false,
    wellFlowRate: 0,
    wellStartLevel: 8,
    wellStopLevel: 20,
    wellSetpointFlow: 2000,
    
    // System
    systemPressure: 60,
    systemDemand: 2500,
    
    // Pumps
    pump1Running: false,
    pump1FlowRate: 0,
    pump1SetpointFlow: 1500,
    pump1Efficiency: 85,
    pump1IsLead: true,
    
    pump2Running: false,
    pump2FlowRate: 0,
    pump2SetpointFlow: 1500,
    pump2Efficiency: 85,
    pump2IsLead: false,
    
    pump3Running: false,
    pump3FlowRate: 0,
    pump3SetpointFlow: 2000,
    pump3Efficiency: 90,
    pump3IsLead: false,
    
    // Pressure setpoints
    leadStartPressure: 55,
    leadStopPressure: 65,
    lagStartPressure: 50,
    lagStopPressure: 70,
    
    // Hydrotanks
    hydrotank1WaterLevel: 50,
    hydrotank1AirBlanket: 50,
    hydrotank1CompressorRunning: false,
    
    hydrotank2WaterLevel: 50,
    hydrotank2AirBlanket: 50,
    hydrotank2CompressorRunning: false,
    
    // Alarms
    alarmTankLow: false,
    alarmTankHigh: false,
    alarmPressureLow: false,
    alarmPressureHigh: false,
    
    // Lead pump rotation
    leadRotationCounter: 1,
  })
  
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
  
  // Read signals from PETRA signal bus
  useEffect(() => {
    if (!signalBus || !isConnected) return
    
    const readSignals = () => {
      try {
        setSignals({
          // Tank
          tankLevelFeet: signalBus.get_float('tank.level_feet') || 12.5,
          tankLevelPercent: signalBus.get_float('tank.level_percent') || 50,
          
          // Well pump
          wellRunning: signalBus.get_bool('well.running') || false,
          wellFlowRate: signalBus.get_float('well.flow_rate') || 0,
          wellStartLevel: signalBus.get_float('well.start_level') || 8,
          wellStopLevel: signalBus.get_float('well.stop_level') || 20,
          wellSetpointFlow: signalBus.get_float('well.setpoint_flow') || 2000,
          
          // System
          systemPressure: signalBus.get_float('system.pressure') || 60,
          systemDemand: signalBus.get_float('system.demand') || 2500,
          
          // Pumps
          pump1Running: signalBus.get_bool('pump1.running') || false,
          pump1FlowRate: signalBus.get_float('pump1.flow_rate') || 0,
          pump1SetpointFlow: signalBus.get_float('pump1.setpoint_flow') || 1500,
          pump1Efficiency: signalBus.get_float('pump1.efficiency') || 85,
          pump1IsLead: signalBus.get_bool('pump1.is_lead') || false,
          
          pump2Running: signalBus.get_bool('pump2.running') || false,
          pump2FlowRate: signalBus.get_float('pump2.flow_rate') || 0,
          pump2SetpointFlow: signalBus.get_float('pump2.setpoint_flow') || 1500,
          pump2Efficiency: signalBus.get_float('pump2.efficiency') || 85,
          pump2IsLead: signalBus.get_bool('pump2.is_lead') || false,
          
          pump3Running: signalBus.get_bool('pump3.running') || false,
          pump3FlowRate: signalBus.get_float('pump3.flow_rate') || 0,
          pump3SetpointFlow: signalBus.get_float('pump3.setpoint_flow') || 2000,
          pump3Efficiency: signalBus.get_float('pump3.efficiency') || 90,
          pump3IsLead: signalBus.get_bool('pump3.is_lead') || false,
          
          // Pressure setpoints
          leadStartPressure: signalBus.get_float('pressure.lead_start') || 55,
          leadStopPressure: signalBus.get_float('pressure.lead_stop') || 65,
          lagStartPressure: signalBus.get_float('pressure.lag_start') || 50,
          lagStopPressure: signalBus.get_float('pressure.lag_stop') || 70,
          
          // Hydrotanks
          hydrotank1WaterLevel: signalBus.get_float('hydrotank1.water_level') || 50,
          hydrotank1AirBlanket: signalBus.get_float('hydrotank1.air_blanket') || 50,
          hydrotank1CompressorRunning: signalBus.get_bool('hydrotank1.compressor_running') || false,
          
          hydrotank2WaterLevel: signalBus.get_float('hydrotank2.water_level') || 50,
          hydrotank2AirBlanket: signalBus.get_float('hydrotank2.air_blanket') || 50,
          hydrotank2CompressorRunning: signalBus.get_bool('hydrotank2.compressor_running') || false,
          
          // Alarms
          alarmTankLow: signalBus.get_bool('alarm.tank_low') || false,
          alarmTankHigh: signalBus.get_bool('alarm.tank_high') || false,
          alarmPressureLow: signalBus.get_bool('alarm.pressure_low') || false,
          alarmPressureHigh: signalBus.get_bool('alarm.pressure_high') || false,
          
          // Lead pump rotation
          leadRotationCounter: signalBus.get_float('lead.rotation_counter') || 1,
        })
        
        setSimulation(prev => ({ ...prev, lastUpdate: new Date() }))
      } catch (error) {
        console.error('Error reading PETRA signals:', error)
      }
    }
    
    // Read signals at 10Hz
    const interval = setInterval(readSignals, 100)
    readSignals() // Initial read
    
    return () => clearInterval(interval)
  }, [signalBus, isConnected])
  
  // Simulation physics (updates PETRA signals)
  useEffect(() => {
    if (!simulation.running || !signalBus || !isConnected) return
    
    const interval = setInterval(() => {
      try {
        const timeStep = 1 / 60 * simulation.timeMultiplier
        
        // Read current values
        const tankLevel = signalBus.get_float('tank.level_feet') || 12.5
        const wellFlow = signalBus.get_float('well.flow_rate') || 0
        const totalPumpFlow = signalBus.get_float('pumps.total_flow') || 0
        const systemDemand = signalBus.get_float('system.demand') || 2500
        
        // Calculate net flow to tank
        const netFlow = wellFlow - totalPumpFlow
        
        // Update tank level (1 foot = 8000 gallons for a 200k gallon, 25ft tank)
        const gallonsPerFoot = 8000
        const levelChange = (netFlow * timeStep) / gallonsPerFoot
        const newLevel = Math.max(0, Math.min(25, tankLevel + levelChange))
        
        // Write updated tank level
        signalBus.set('tank.level_feet', newLevel)
        
        // Calculate system pressure based on pump flow vs demand
        const currentPressure = signalBus.get_float('system.pressure') || 60
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
        
        signalBus.set('system.pressure', Math.round(newPressure))
        
        // Update hydrotank levels based on net system flow
        const netSystemFlow = totalPumpFlow - systemDemand
        const htFlowShare = netSystemFlow * timeStep / 2 // Split between two tanks
        
        // Hydrotank 1
        const ht1WaterLevel = signalBus.get_float('hydrotank1.water_level') || 50
        const ht1NewLevel = Math.max(0, Math.min(100, ht1WaterLevel + htFlowShare / 50)) // 50 gpm = 1% per minute
        signalBus.set('hydrotank1.water_level', ht1NewLevel)
        signalBus.set('hydrotank1.air_blanket', 100 - ht1NewLevel)
        
        // Hydrotank 2
        const ht2WaterLevel = signalBus.get_float('hydrotank2.water_level') || 50
        const ht2NewLevel = Math.max(0, Math.min(100, ht2WaterLevel + htFlowShare / 50))
        signalBus.set('hydrotank2.water_level', ht2NewLevel)
        signalBus.set('hydrotank2.air_blanket', 100 - ht2NewLevel)
        
      } catch (error) {
        console.error('Simulation error:', error)
      }
    }, 100)
    
    return () => clearInterval(interval)
  }, [simulation.running, simulation.timeMultiplier, signalBus, isConnected])
  
  // Write signal to PETRA
  const writeSignal = (signal: string, value: any) => {
    if (!signalBus || !isConnected) return
    
    try {
      if (typeof value === 'boolean') {
        signalBus.set(signal, value)
      } else if (typeof value === 'number') {
        signalBus.set(signal, value)
      } else {
        signalBus.set(signal, value.toString())
      }
    } catch (error) {
      console.error(`Error writing signal ${signal}:`, error)
    }
  }
  
  // Control handlers
  const togglePump = (pumpNum: number) => {
    const signal = `pump${pumpNum}.running`
    const currentValue = signalBus?.get_bool(signal) || false
    writeSignal(signal, !currentValue)
  }
  
  const toggleWellPump = () => {
    const currentValue = signalBus?.get_bool('well.running') || false
    writeSignal('well.running', !currentValue)
  }
  
  const updateSetpoint = (signal: string, value: number) => {
    writeSignal(signal, value)
  }
  
  const currentLeadPump = Math.round(signals.leadRotationCounter)
  
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
                  'text-gray-700'
                }`}>
                  {signals.tankLevelFeet.toFixed(1)} ft ({signals.tankLevelPercent.toFixed(0)}%)
                </span>
              </div>
              <div className="flex justify-between">
                <span>Total Pump Flow:</span>
                <span className="font-medium">
                  {(signals.pump1FlowRate + signals.pump2FlowRate + signals.pump3FlowRate).toFixed(0)} gpm
                </span>
              </div>
            </div>
          </div>
          
          {/* Well Pump Control */}
          <div className="mb-6">
            <h3 className="font-semibold mb-3">Well Pump Control</h3>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between items-center mb-3">
                <span className="font-medium">Manual Control:</span>
                <button
                  onClick={toggleWellPump}
                  className={`px-3 py-1 rounded text-sm ${
                    signals.wellRunning 
                      ? 'bg-green-500 text-white' 
                      : 'bg-gray-300 text-gray-700'
                  }`}
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
                />
              </div>
              <div className="mt-2 p-2 bg-gray-100 rounded">
                <div className="text-xs text-gray-600">
                  Current: {signals.wellRunning ? 'Running' : 'Stopped'} at {signals.wellFlowRate} gpm
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
                  value={signals.leadStartPressure}
                  onChange={(e) => updateSetpoint('pressure.lead_start', parseInt(e.target.value))}
                  className="w-20 px-2 py-1 border rounded text-right"
                />
              </div>
              <div className="flex justify-between items-center">
                <span>Stop Pressure:</span>
                <input
                  type="number"
                  value={signals.leadStopPressure}
                  onChange={(e) => updateSetpoint('pressure.lead_stop', parseInt(e.target.value))}
                  className="w-20 px-2 py-1 border rounded text-right"
                />
              </div>
              
              <div className="font-medium text-gray-700 mt-3">Lag Pumps</div>
              <div className="flex justify-between items-center">
                <span>Start Pressure:</span>
                <input
                  type="number"
                  value={signals.lagStartPressure}
                  onChange={(e) => updateSetpoint('pressure.lag_start', parseInt(e.target.value))}
                  className="w-20 px-2 py-1 border rounded text-right"
                />
              </div>
              <div className="flex justify-between items-center">
                <span>Stop Pressure:</span>
                <input
                  type="number"
                  value={signals.lagStopPressure}
                  onChange={(e) => updateSetpoint('pressure.lag_stop', parseInt(e.target.value))}
                  className="w-20 px-2 py-1 border rounded text-right"
                />
              </div>
            </div>
          </div>
          
          {/* Current Lead Pump Indicator */}
          <div className="mb-6 p-4 bg-green-50 rounded-lg">
            <h3 className="font-semibold mb-2">Lead Pump Rotation</h3>
            <div className="text-sm">
              <span>Current Lead: </span>
              <span className="font-bold text-green-700">Pump {currentLeadPump}</span>
            </div>
            <div className="text-xs text-gray-600 mt-1">
              Controlled by PETRA logic blocks
            </div>
          </div>
          
          {/* Booster Pump Status */}
          <div className="mb-6">
            <h3 className="font-semibold mb-3">Booster Pump Status</h3>
            <div className="space-y-3">
              {[1, 2, 3].map(num => {
                const isRunning = signals[`pump${num}Running`]
                const flowRate = signals[`pump${num}FlowRate`]
                const setpointFlow = signals[`pump${num}SetpointFlow`]
                const efficiency = signals[`pump${num}Efficiency`]
                const isLead = signals[`pump${num}IsLead`]
                
                return (
                  <div 
                    key={num} 
                    className={`p-3 border rounded-lg ${
                      isLead ? 'border-green-500 bg-green-50' : 'border-gray-200'
                    }`}
                  >
                    <div className="flex items-center justify-between mb-2">
                      <span className="font-medium">
                        Pump {num} 
                        {isLead && 
                          <span className="ml-2 text-xs bg-green-600 text-white px-2 py-0.5 rounded">LEAD</span>
                        }
                      </span>
                      <button
                        onClick={() => togglePump(num)}
                        className={`px-3 py-1 rounded text-sm ${
                          isRunning 
                            ? 'bg-green-500 text-white' 
                            : 'bg-gray-300 text-gray-700'
                        }`}
                      >
                        {isRunning ? 'Running' : 'Stopped'}
                      </button>
                    </div>
                    
                    <div className="text-sm text-gray-600 space-y-1">
                      <div>Flow: {flowRate.toFixed(0)} / {setpointFlow} gpm</div>
                      <div className="flex items-center gap-2">
                        <span>Setpoint:</span>
                        <input
                          type="number"
                          value={setpointFlow}
                          onChange={(e) => updateSetpoint(`pump${num}.setpoint_flow`, parseInt(e.target.value))}
                          className="w-16 px-1 py-0.5 border rounded text-right text-xs"
                          min="0"
                          max="3000"
                          step="100"
                        />
                        <span className="text-xs text-gray-500">gpm</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <span>Efficiency:</span>
                        <input
                          type="number"
                          value={efficiency}
                          onChange={(e) => updateSetpoint(`pump${num}.efficiency`, parseInt(e.target.value))}
                          className="w-16 px-1 py-0.5 border rounded text-right text-xs"
                          min="0"
                          max="100"
                          step="5"
                        />
                        <span className="text-xs text-gray-500">%</span>
                      </div>
                    </div>
                  </div>
                )
              })}
            </div>
          </div>
          
          {/* Active Alarms */}
          {(signals.alarmTankLow || signals.alarmTankHigh || signals.alarmPressureLow || signals.alarmPressureHigh) && (
            <div className="mb-6">
              <h3 className="font-semibold mb-3 flex items-center gap-2">
                <FaExclamationTriangle className="text-red-600" />
                Active Alarms
              </h3>
              <div className="space-y-2">
                {signals.alarmTankLow && (
                  <div className="text-sm p-2 bg-red-50 rounded border border-red-200">
                    <div className="font-medium">Tank Low Level</div>
                    <div className="text-red-600">Level below 5 feet</div>
                  </div>
                )}
                {signals.alarmTankHigh && (
                  <div className="text-sm p-2 bg-red-50 rounded border border-red-200">
                    <div className="font-medium">Tank High Level</div>
                    <div className="text-red-600">Level above 22.5 feet</div>
                  </div>
                )}
                {signals.alarmPressureLow && (
                  <div className="text-sm p-2 bg-red-50 rounded border border-red-200">
                    <div className="font-medium">Low System Pressure</div>
                    <div className="text-red-600">Pressure below 40 psi</div>
                  </div>
                )}
                {signals.alarmPressureHigh && (
                  <div className="text-sm p-2 bg-red-50 rounded border border-red-200">
                    <div className="font-medium">High System Pressure</div>
                    <div className="text-red-600">Pressure above 80 psi</div>
                  </div>
                )}
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
              text="Water Treatment Plant - PETRA Logic Control"
              fontSize={24}
              fontStyle="bold"
              fill="#1e293b"
            />
            
            {/* PETRA Status */}
            <Group x={stageSize.width - 200} y={20}>
              <Rect
                x={0}
                y={0}
                width={180}
                height={30}
                fill={isConnected ? '#dcfce7' : '#fee2e2'}
                stroke={isConnected ? '#16a34a' : '#dc2626'}
                strokeWidth={2}
                cornerRadius={5}
              />
              <Text
                x={0}
                y={5}
                width={180}
                height={20}
                text={isConnected ? 'PETRA Connected' : 'PETRA Disconnected'}
                fontSize={14}
                align="center"
                verticalAlign="middle"
                fill={isConnected ? '#16a34a' : '#dc2626'}
                fontStyle="bold"
              />
            </Group>
            
            {/* Ground Storage Tank */}
            <Group x={100} y={200}>
              <TankComponent
                x={0}
                y={0}
                width={200}
                height={250}
                properties={{
                  maxLevel: 100,
                  currentLevel: signals.tankLevelPercent,
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
                text={`Level: ${signals.tankLevelFeet.toFixed(1)} ft`}
                fontSize={12}
                align="center"
                fill="#64748b"
              />
              <Text
                x={0}
                y={275}
                width={200}
                text={`Start: ${signals.wellStartLevel} ft | Stop: ${signals.wellStopLevel} ft`}
                fontSize={10}
                align="center"
                fill="#94a3b8"
              />
            </Group>
          </Layer>
        </Stage>
      </div>
    </div>
  )
}
