// src/components/hmi/WaterPlantPetraDemo.tsx

import { useState, useEffect } from 'react'
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
    tankLevelFeet: petraSignals.get('tank.level_feet') ?? 12.5,
    tankLevelPercent: petraSignals.get('tank.level_percent') ?? 50,

    // Well pump
    wellRunning: petraSignals.get('well.running') ?? false,
    wellFlowRate: petraSignals.get('well.flow_rate') ?? 0,
    wellStartLevel: petraSignals.get('well.start_level') ?? 8,
    wellStopLevel: petraSignals.get('well.stop_level') ?? 20,
    wellSetpointFlow: petraSignals.get('well.setpoint_flow') ?? 2000,

    // System
    systemPressure: petraSignals.get('system.pressure') ?? 60,
    systemDemand: petraSignals.get('system.demand') ?? 2500,

    // Pumps
    pump1Running: petraSignals.get('pump1.running') ?? false,
    pump1FlowRate: petraSignals.get('pump1.flow_rate') ?? 0,
    pump1SetpointFlow: petraSignals.get('pump1.setpoint_flow') ?? 1500,
    pump1Efficiency: petraSignals.get('pump1.efficiency') ?? 85,
    pump1IsLead: petraSignals.get('pump1.is_lead') ?? false,

    pump2Running: petraSignals.get('pump2.running') ?? false,
    pump2FlowRate: petraSignals.get('pump2.flow_rate') ?? 0,
    pump2SetpointFlow: petraSignals.get('pump2.setpoint_flow') ?? 1500,
    pump2Efficiency: petraSignals.get('pump2.efficiency') ?? 85,
    pump2IsLead: petraSignals.get('pump2.is_lead') ?? false,

    pump3Running: petraSignals.get('pump3.running') ?? false,
    pump3FlowRate: petraSignals.get('pump3.flow_rate') ?? 0,
    pump3SetpointFlow: petraSignals.get('pump3.setpoint_flow') ?? 2000,
    pump3Efficiency: petraSignals.get('pump3.efficiency') ?? 90,
    pump3IsLead: petraSignals.get('pump3.is_lead') ?? false,

    // Pressure setpoints
    leadStartPressure: petraSignals.get('pressure.lead_start') ?? 55,
    leadStopPressure: petraSignals.get('pressure.lead_stop') ?? 65,
    lagStartPressure: petraSignals.get('pressure.lag_start') ?? 50,
    lagStopPressure: petraSignals.get('pressure.lag_stop') ?? 70,

    // Hydrotanks
    hydrotank1WaterLevel: petraSignals.get('hydrotank1.water_level') ?? 50,
    hydrotank1AirBlanket: petraSignals.get('hydrotank1.air_blanket') ?? 50,
    compressor1Running: petraSignals.get('compressor1.running') ?? false,

    hydrotank2WaterLevel: petraSignals.get('hydrotank2.water_level') ?? 50,
    hydrotank2AirBlanket: petraSignals.get('hydrotank2.air_blanket') ?? 50,
    compressor2Running: petraSignals.get('compressor2.running') ?? false,

    // Hydrotank setpoints
    hydrotankAirBlanketMin: petraSignals.get('hydrotank.air_blanket_min') ?? 45,
    hydrotankAirBlanketMax: petraSignals.get('hydrotank.air_blanket_max') ?? 55,

    // Alarms
    alarmTankLow: petraSignals.get('alarm.tank_low') ?? false,
    alarmTankHigh: petraSignals.get('alarm.tank_high') ?? false,
    alarmPressureLow: petraSignals.get('alarm.pressure_low') ?? false,
    alarmPressureHigh: petraSignals.get('alarm.pressure_high') ?? false,

    // Lead pump rotation
    currentLeadPump: Math.round(petraSignals.get('lead.rotation_counter') ?? 1),
    
    // Total pump flow
    totalPumpFlow: petraSignals.get('pumps.total_flow') ?? 0,
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
  
  // Update connection status
  useEffect(() => {
    setSimulation(prev => ({
      ...prev,
      petraConnected: isConnected,
      lastUpdate: isConnected ? new Date() : prev.lastUpdate
    }))
  }, [isConnected])
  
  // Simulation physics (updates PETRA signals)
  useEffect(() => {
    if (!simulation.running || !isConnected) return
    
    const interval = setInterval(() => {
      try {
        const timeStep = 1 / 60 * simulation.timeMultiplier
        
        // Read current values
        const tankLevel = Number(petraSignals.get('tank.level_feet') ?? 12.5)
        const wellFlow = Number(petraSignals.get('well.flow_rate') ?? 0)
        const totalPumpFlow = Number(petraSignals.get('pumps.total_flow') ?? 0)
        const systemDemand = Number(petraSignals.get('system.demand') ?? 2500)
        
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
        const currentPressure = Number(petraSignals.get('system.pressure') ?? 60)
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
        const ht1WaterLevel = Number(petraSignals.get('hydrotank1.water_level') ?? 50)
        const ht1NewLevel = Math.max(0, Math.min(100, ht1WaterLevel + htFlowShare / 50)) // 50 gpm = 1% per minute
        setSignalValue('hydrotank1.water_level', ht1NewLevel)
        setSignalValue('hydrotank1.air_blanket', 100 - ht1NewLevel)
        
        // Hydrotank 2
        const ht2WaterLevel = Number(petraSignals.get('hydrotank2.water_level') ?? 50)
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
  }, [simulation.running, simulation.timeMultiplier, isConnected, petraSignals, setSignalValue])
  
  // Write signal to PETRA
  const writeSignal = (signal: string, value: any) => {
    if (!isConnected) return
    
    try {
      setSignalValue(signal, value)
    } catch (error) {
      console.error(`Error writing signal ${signal}:`, error)
    }
  }
  
  // Control handlers
  const togglePump = (pumpNum: number) => {
    const signal = `pump${pumpNum}.running`
    const currentValue = Boolean(petraSignals.get(signal)) || false
    writeSignal(signal, !currentValue)
  }
  
  const toggleWellPump = () => {
    const currentValue = Boolean(petraSignals.get('well.running')) || false
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
                  'text-gray-700'
                }`}>
                  {signals.tankLevelFeet.toFixed(1)} ft ({signals.tankLevelPercent.toFixed(0)}%)
                </span>
              </div>
              <div className="flex justify-between">
                <span>Total Pump Flow:</span>
                <span className="font-medium">
                  {signals.totalPumpFlow.toFixed(0)} gpm
                </span>
              </div>
              <div className="flex justify-between">
                <span>Current Demand:</span>
                <span className="font-medium">{signals.systemDemand.toLocaleString()} gpm</span>
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
                  disabled={!isConnected}
                />
              </div>
              <div className="flex justify-between items-center">
                <span>Stop Pressure:</span>
                <input
                  type="number"
                  value={signals.leadStopPressure}
                  onChange={(e) => updateSetpoint('pressure.lead_stop', parseInt(e.target.value))}
                  className="w-20 px-2 py-1 border rounded text-right"
                  disabled={!isConnected}
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
                  disabled={!isConnected}
                />
              </div>
              <div className="flex justify-between items-center">
                <span>Stop Pressure:</span>
                <input
                  type="number"
                  value={signals.lagStopPressure}
                  onChange={(e) => updateSetpoint('pressure.lag_stop', parseInt(e.target.value))}
                  className="w-20 px-2 py-1 border rounded text-right"
                  disabled={!isConnected}
                />
              </div>
            </div>
          </div>
          
          {/* Hydrotank Setpoints */}
          <div className="mb-6">
            <h3 className="font-semibold mb-3">Hydrotank Air Blanket Control</h3>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between items-center">
                <span>Compressor Start (%):</span>
                <input
                  type="number"
                  value={signals.hydrotankAirBlanketMin}
                  onChange={(e) => updateSetpoint('hydrotank.air_blanket_min', parseInt(e.target.value))}
                  className="w-20 px-2 py-1 border rounded text-right"
                  min="20"
                  max="80"
                  disabled={!isConnected}
                />
              </div>
              <div className="flex justify-between items-center">
                <span>Compressor Stop (%):</span>
                <input
                  type="number"
                  value={signals.hydrotankAirBlanketMax}
                  onChange={(e) => updateSetpoint('hydrotank.air_blanket_max', parseInt(e.target.value))}
                  className="w-20 px-2 py-1 border rounded text-right"
                  min="20"
                  max="80"
                  disabled={!isConnected}
                />
              </div>
            </div>
          </div>
          
          {/* Current Lead Pump Indicator */}
          <div className="mb-6 p-4 bg-green-50 rounded-lg">
            <h3 className="font-semibold mb-2">Lead Pump Rotation</h3>
            <div className="text-sm">
              <span>Current Lead: </span>
              <span className="font-bold text-green-700">Pump {signals.currentLeadPump}</span>
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
                const isRunning = (signals as any)[`pump${num}Running`]
                const flowRate = (signals as any)[`pump${num}FlowRate`]
                const setpointFlow = (signals as any)[`pump${num}SetpointFlow`]
                const efficiency = (signals as any)[`pump${num}Efficiency`]
                const isLead = num === signals.currentLeadPump
                
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
                        disabled={!isConnected}
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
                          disabled={!isConnected}
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
                          disabled={!isConnected}
                        />
                        <span className="text-xs text-gray-500">%</span>
                      </div>
                    </div>
                  </div>
                )
              })}
            </div>
          </div>
          
          {/* Hydrotank Status */}
          <div className="mb-6">
            <h3 className="font-semibold mb-3">Hydrotank Status</h3>
            <div className="space-y-3">
              {[1, 2].map(num => {
                const waterLevel = num === 1 ? signals.hydrotank1WaterLevel : signals.hydrotank2WaterLevel
                const airBlanket = num === 1 ? signals.hydrotank1AirBlanket : signals.hydrotank2AirBlanket
                const compressorRunning = num === 1 ? signals.compressor1Running : signals.compressor2Running
                
                return (
                  <div key={num} className="p-3 border rounded-lg">
                    <div className="font-medium mb-2">Hydrotank {num}</div>
                    <div className="text-sm text-gray-600 space-y-1">
                      <div>Air Blanket: {airBlanket.toFixed(1)}%</div>
                      <div>Water Level: {waterLevel.toFixed(1)}%</div>
                      <div>Pressure: {signals.systemPressure} psi</div>
                      <div className="flex items-center gap-2">
                        <span>Compressor:</span>
                        <span className={`font-medium ${compressorRunning ? 'text-green-600' : 'text-gray-600'}`}>
                          {compressorRunning ? 'Running' : 'Off'}
                        </span>
                      </div>
                    </div>
                  </div>
                )
              })}
            </div>
          </div>
          
          {/* Active Alarms */}
          {(signals.alarmTankLow || signals.alarmTankHigh || signals.alarmPressureLow || signals.alarmPressureHigh) && (
            <div className="mb-6 p-4 bg-red-50 rounded-lg">
              <h3 className="font-semibold mb-2 text-red-700 flex items-center gap-2">
                <FaExclamationTriangle />
                Active Alarms
              </h3>
              <div className="space-y-1 text-sm">
                {signals.alarmTankLow && (
                  <div className="text-red-600">
                    <span className="font-medium">Tank Low:</span> Level below 5 ft
                  </div>
                )}
                {signals.alarmTankHigh && (
                  <div className="text-red-600">
                    <span className="font-medium">Tank High:</span> Level above 22.5 ft
                  </div>
                )}
                {signals.alarmPressureLow && (
                  <div className="text-red-600">
                    <span className="font-medium">Low Pressure:</span> Below 40 psi
                  </div>
                )}
                {signals.alarmPressureHigh && (
                  <div className="text-red-600">
                    <span className="font-medium">High Pressure:</span> Above 80 psi
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
        className="absolute left-0 top-1/2 transform -translate-y-1/2 bg-white border border-gray-300 rounded-r-lg px-2 py-4 shadow-lg hover:bg-gray-50 z-10"
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
                y={8}
                width={180}
                text={isConnected ? 'PETRA Connected' : 'PETRA Disconnected'}
                fontSize={14}
                align="center"
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
                  stroke: signals.alarmTankHigh || signals.alarmTankLow ? '#dc2626' : '#0284c7',
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
                text={`200k gal capacity`}
                fontSize={12}
                align="center"
                fill="#64748b"
              />
              <Text
                x={0}
                y={275}
                width={200}
                text={`Level: ${signals.tankLevelFeet.toFixed(1)} ft`}
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
                  running: signals.wellRunning,
                  fault: false,
                  speed: signals.wellRunning ? 100 : 0,
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
                text={`${signals.wellFlowRate} gpm`}
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
            {[1, 2, 3].map((num, index) => {
              const isRunning = (signals as any)[`pump${num}Running`]
              const flowRate = (signals as any)[`pump${num}FlowRate`]
              const isLead = num === signals.currentLeadPump
              
              return (
                <Group key={num} x={400 + index * 150} y={350}>
                  <PumpComponent
                    x={0}
                    y={0}
                    width={80}
                    height={80}
                    properties={{
                      running: isRunning,
                      fault: false,
                      speed: isRunning ? 85 : 0,
                      showStatus: true,
                      runAnimation: true,
                    }}
                    bindings={[]}
                    style={{
                      fill: '#ffffff',
                      stroke: isRunning ? '#16a34a' : '#6b7280',
                      strokeWidth: 2,
                    }}
                  />
                  <Text
                    x={-20}
                    y={-25}
                    width={120}
                    text={`Pump ${num}`}
                    fontSize={14}
                    align="center"
                    fontStyle="bold"
                    fill={isRunning ? '#16a34a' : '#6b7280'}
                  />
                  {isLead && (
                    <Rect
                      x={20}
                      y={-40}
                      width={40}
                      height={20}
                      fill="#16a34a"
                      cornerRadius={3}
                    />
                  )}
                  {isLead && (
                    <Text
                      x={20}
                      y={-36}
                      width={40}
                      text="LEAD"
                      fontSize={10}
                      align="center"
                      fill="#ffffff"
                      fontStyle="bold"
                    />
                  )}
                  <Text
                    x={-20}
                    y={85}
                    width={120}
                    text={`${flowRate.toFixed(0)} gpm`}
                    fontSize={12}
                    align="center"
                    fill="#64748b"
                  />
                  <Text
                    x={-20}
                    y={100}
                    width={120}
                    text={`${signals.systemPressure} psi`}
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
                    stroke={isRunning ? '#16a34a' : '#6b7280'}
                    strokeWidth={4}
                  />
                </Group>
              )
            })}
            
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
            {[1, 2].map((num, index) => {
              const waterLevel = num === 1 ? signals.hydrotank1WaterLevel : signals.hydrotank2WaterLevel
              const airBlanket = num === 1 ? signals.hydrotank1AirBlanket : signals.hydrotank2AirBlanket
              const compressorRunning = num === 1 ? signals.compressor1Running : signals.compressor2Running
              
              return (
                <Group key={num} x={900 + index * 180} y={320}>
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
                    y={180 - (180 * waterLevel / 100) - 3}
                    width={114}
                    height={180 * waterLevel / 100}
                    fill="#3b82f6"
                    opacity={0.3}
                    cornerRadius={8}
                  />
                  
                  {/* Air blanket indicator */}
                  <Rect
                    x={3}
                    y={3}
                    width={114}
                    height={180 * airBlanket / 100 - 6}
                    fill="#e0f2fe"
                    opacity={0.5}
                    cornerRadius={8}
                  />
                  
                  {/* Compressor */}
                  <Circle
                    x={60}
                    y={-30}
                    radius={15}
                    fill={compressorRunning ? '#10b981' : '#e5e7eb'}
                    stroke="#374151"
                    strokeWidth={2}
                  />
                  <Text
                    x={55}
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
                    text={`Hydrotank ${num}`}
                    fontSize={14}
                    align="center"
                    fontStyle="bold"
                    fill="#0284c7"
                  />
                  
                  <Text
                    x={0}
                    y={190}
                    width={120}
                    text={`Air: ${airBlanket.toFixed(0)}%`}
                    fontSize={11}
                    align="center"
                    fill="#374151"
                  />
                  
                  <Text
                    x={0}
                    y={205}
                    width={120}
                    text={`${signals.systemPressure} psi`}
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
            <Group x={750} y={70}>
              <GaugeComponent
                x={0}
                y={0}
                width={150}
                height={150}
                properties={{
                  min: 0,
                  max: 100,
                  value: signals.systemPressure,
                  units: 'psi',
                  showScale: true,
                  majorTicks: 5,
                }}
                bindings={[]}
                style={{
                  fill: '#ffffff',
                  stroke: signals.alarmPressureLow || signals.alarmPressureHigh ? '#dc2626' : '#0284c7',
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
            <Group x={950} y={70}>
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
                text={`${signals.systemDemand.toLocaleString()} gpm`}
                fontSize={24}
                fontStyle="bold"
                fill="#0284c7"
              />
              <Text
                x={0}
                y={55}
                text="Total Pump Flow"
                fontSize={14}
                fill="#64748b"
              />
              <Text
                x={0}
                y={75}
                text={`${signals.totalPumpFlow.toFixed(0)} gpm`}
                fontSize={20}
                fontStyle="bold"
                fill={signals.totalPumpFlow >= signals.systemDemand ? '#16a34a' : '#dc2626'}
              />
            </Group>
          </Layer>
        </Stage>
      </div>
    </div>
  )
}
