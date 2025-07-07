// src/components/hmi/WaterPlantDemo.tsx

import { useState, useEffect, useRef } from 'react'
import { Stage, Layer, Group, Rect, Circle, Line, Text, Path } from 'react-konva'
import { FaPlay, FaPause, FaCog, FaChartLine } from 'react-icons/fa'
import TankComponent from './components/TankComponent'
import PumpComponent from './components/PumpComponent'
import ValveComponent from './components/ValveComponent'
import GaugeComponent from './components/GaugeComponent'

interface SimulationState {
  // Tank parameters
  tankCapacity: number // gallons
  tankLevel: number // gallons
  tankLevelPercent: number // %
  
  // Well parameters
  wellFlowRate: number // gpm
  wellRunning: boolean
  
  // Pump parameters
  boosterPumps: Array<{
    id: string
    name: string
    flowRate: number // gpm
    running: boolean
    efficiency: number // %
    pressure: number // psi
  }>
  
  // System parameters
  demand: number // gpm
  systemPressure: number // psi
  targetPressure: number // psi
  
  // Simulation
  running: boolean
  timeMultiplier: number
}

export default function WaterPlantDemo() {
  const [simulation, setSimulation] = useState<SimulationState>({
    tankCapacity: 200000,
    tankLevel: 100000,
    tankLevelPercent: 50,
    wellFlowRate: 2000,
    wellRunning: true,
    boosterPumps: [
      { id: 'bp1', name: 'Booster 1', flowRate: 1500, running: true, efficiency: 85, pressure: 60 },
      { id: 'bp2', name: 'Booster 2', flowRate: 1500, running: false, efficiency: 85, pressure: 60 },
      { id: 'bp3', name: 'Booster 3', flowRate: 2000, running: false, efficiency: 90, pressure: 65 },
    ],
    demand: 2500,
    systemPressure: 55,
    targetPressure: 60,
    running: true,
    timeMultiplier: 10,
  })
  
  const [showControls, setShowControls] = useState(true)
  const simulationRef = useRef<SimulationState>(simulation)
  simulationRef.current = simulation
  
  // Simulation loop
  useEffect(() => {
    if (!simulation.running) return
    
    const interval = setInterval(() => {
      setSimulation(prev => {
        const timeStep = 1 / 60 * prev.timeMultiplier // 1 second of simulation time
        
        // Calculate total pump output
        const totalPumpOutput = prev.boosterPumps
          .filter(p => p.running)
          .reduce((sum, pump) => sum + pump.flowRate * (pump.efficiency / 100), 0)
        
        // Calculate net flow (well + pumps - demand)
        const wellFlow = prev.wellRunning ? prev.wellFlowRate : 0
        const netFlow = wellFlow - prev.demand + totalPumpOutput
        
        // Update tank level
        const newLevel = Math.max(0, Math.min(prev.tankCapacity, prev.tankLevel + (netFlow * timeStep)))
        const newLevelPercent = (newLevel / prev.tankCapacity) * 100
        
        // Calculate system pressure based on running pumps and tank level
        const runningPumps = prev.boosterPumps.filter(p => p.running)
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
        
        return {
          ...prev,
          tankLevel: newLevel,
          tankLevelPercent: newLevelPercent,
          systemPressure: Math.round(newPressure),
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
  
  const updatePumpFlowRate = (pumpId: string, flowRate: number) => {
    setSimulation(prev => ({
      ...prev,
      boosterPumps: prev.boosterPumps.map(pump =>
        pump.id === pumpId ? { ...pump, flowRate } : pump
      )
    }))
  }
  
  return (
    <div className="flex h-full bg-gray-50">
      {/* Control Panel */}
      {showControls && (
        <div className="w-80 bg-white border-r border-gray-200 p-4 overflow-y-auto">
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
            <div className="flex justify-between text-sm text-gray-500">
              <span>0</span>
              <span>{(simulation.demand / 1000).toFixed(1)}k gpm</span>
              <span>10k</span>
            </div>
          </div>
          
          {/* Tank Configuration */}
          <div className="mb-6">
            <h3 className="font-semibold mb-3">Ground Storage Tank</h3>
            <div className="space-y-2">
              <div>
                <label className="text-sm text-gray-600">Capacity (gallons)</label>
                <input
                  type="number"
                  value={simulation.tankCapacity}
                  onChange={(e) => setSimulation(prev => ({ ...prev, tankCapacity: parseInt(e.target.value) }))}
                  className="w-full px-2 py-1 border rounded"
                  step="10000"
                />
              </div>
              <div>
                <label className="text-sm text-gray-600">Current Level</label>
                <div className="text-sm font-medium">
                  {simulation.tankLevel.toLocaleString()} gal ({simulation.tankLevelPercent.toFixed(1)}%)
                </div>
              </div>
            </div>
          </div>
          
          {/* Well Control */}
          <div className="mb-6">
            <h3 className="font-semibold mb-3">Well Pump</h3>
            <div className="space-y-2">
              <label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={simulation.wellRunning}
                  onChange={(e) => setSimulation(prev => ({ ...prev, wellRunning: e.target.checked }))}
                  className="w-4 h-4"
                />
                <span className="text-sm">Well Pump Running</span>
              </label>
              <div>
                <label className="text-sm text-gray-600">Flow Rate (gpm)</label>
                <input
                  type="number"
                  value={simulation.wellFlowRate}
                  onChange={(e) => setSimulation(prev => ({ ...prev, wellFlowRate: parseInt(e.target.value) }))}
                  className="w-full px-2 py-1 border rounded"
                  step="100"
                />
              </div>
            </div>
          </div>
          
          {/* Booster Pumps */}
          <div className="mb-6">
            <h3 className="font-semibold mb-3">Booster Pumps</h3>
            <div className="space-y-3">
              {simulation.boosterPumps.map(pump => (
                <div key={pump.id} className="p-3 border rounded-lg">
                  <div className="flex items-center justify-between mb-2">
                    <span className="font-medium">{pump.name}</span>
                    <button
                      onClick={() => togglePump(pump.id)}
                      className={`px-3 py-1 rounded text-sm text-white transition-colors ${
                        pump.running ? 'bg-green-500 hover:bg-green-600' : 'bg-gray-400 hover:bg-gray-500'
                      }`}
                    >
                      {pump.running ? 'Running' : 'Stopped'}
                    </button>
                  </div>
                  <div className="space-y-1 text-sm">
                    <div className="flex justify-between">
                      <span className="text-gray-600">Flow Rate:</span>
                      <input
                        type="number"
                        value={pump.flowRate}
                        onChange={(e) => updatePumpFlowRate(pump.id, parseInt(e.target.value))}
                        className="w-20 px-1 py-0.5 border rounded text-right"
                        step="100"
                        disabled={pump.running}
                      />
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-600">Pressure:</span>
                      <span>{pump.pressure} psi</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-600">Efficiency:</span>
                      <span>{pump.efficiency}%</span>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}
      
      {/* Toggle Controls Button */}
      <button
        onClick={() => setShowControls(!showControls)}
        className="absolute left-0 top-1/2 transform -translate-y-1/2 bg-white p-2 rounded-r shadow-lg z-10"
      >
        {showControls ? '◀' : '▶'}
      </button>
      
      {/* Main Display */}
      <div className="flex-1 relative">
        <Stage width={window.innerWidth - (showControls ? 320 : 0)} height={window.innerHeight - 60}>
          <Layer>
            {/* Background */}
            <Rect
              x={0}
              y={0}
              width={window.innerWidth - (showControls ? 320 : 0)}
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
                  fault: false,
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
                    fault: false,
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
                y={-30}
                width={150}
                text="System Pressure"
                fontSize={16}
                fontStyle="bold"
                align="center"
                fill="#0284c7"
              />
            </Group>
            
            {/* Demand Indicator */}
            <Group x={850} y={300}>
              <Rect
                x={0}
                y={0}
                width={150}
                height={80}
                fill="#fee2e2"
                stroke="#dc2626"
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
                fill="#dc2626"
              />
              <Text
                x={0}
                y={35}
                width={150}
                text={`${simulation.demand.toLocaleString()}`}
                fontSize={24}
                fontStyle="bold"
                align="center"
                fill="#dc2626"
              />
              <Text
                x={0}
                y={60}
                width={150}
                text="gpm"
                fontSize={14}
                align="center"
                fill="#dc2626"
              />
              
              {/* Demand arrow */}
              <Line
                points={[75, 80, 75, 120, 85, 110, 75, 120, 65, 110]}
                stroke="#dc2626"
                strokeWidth={3}
              />
            </Group>
            
            {/* Flow Summary */}
            <Group x={400} y={100}>
              <Rect
                x={0}
                y={0}
                width={300}
                height={120}
                fill="#ffffff"
                stroke="#64748b"
                strokeWidth={1}
                cornerRadius={5}
              />
              <Text
                x={10}
                y={10}
                text="Flow Summary"
                fontSize={14}
                fontStyle="bold"
                fill="#1e293b"
              />
              <Text
                x={10}
                y={35}
                text={`Well Input: ${simulation.wellRunning ? simulation.wellFlowRate : 0} gpm`}
                fontSize={12}
                fill="#64748b"
              />
              <Text
                x={10}
                y={55}
                text={`Booster Output: ${simulation.boosterPumps
                  .filter(p => p.running)
                  .reduce((sum, p) => sum + p.flowRate * (p.efficiency / 100), 0)
                  .toFixed(0)} gpm`}
                fontSize={12}
                fill="#64748b"
              />
              <Text
                x={10}
                y={75}
                text={`System Demand: ${simulation.demand} gpm`}
                fontSize={12}
                fill="#64748b"
              />
              <Text
                x={10}
                y={95}
                text={`Net Flow: ${(
                  (simulation.wellRunning ? simulation.wellFlowRate : 0) +
                  simulation.boosterPumps
                    .filter(p => p.running)
                    .reduce((sum, p) => sum + p.flowRate * (p.efficiency / 100), 0) -
                  simulation.demand
                ).toFixed(0)} gpm`}
                fontSize={12}
                fontStyle="bold"
                fill={
                  (simulation.wellRunning ? simulation.wellFlowRate : 0) +
                  simulation.boosterPumps
                    .filter(p => p.running)
                    .reduce((sum, p) => sum + p.flowRate * (p.efficiency / 100), 0) -
                  simulation.demand >= 0
                    ? '#16a34a'
                    : '#dc2626'
                }
              />
            </Group>
          </Layer>
        </Stage>
      </div>
    </div>
  )
}
