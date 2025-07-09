// File: petra-designer/src/components/hmi/components/ISA101TankComponent.tsx
// ISA-101 Compliant Tank Component following HMI standards
import { useEffect, useState, useRef } from 'react'
import Konva from 'konva'
import { Group, Rect, Shape, Text, Line, Circle } from 'react-konva'

// ISA-101 Standard Colors
const ISA101Colors = {
  // Process lines and equipment
  processLine: '#000000',           // Black for process lines
  equipmentOutline: '#000000',      // Black outlines
  equipmentFill: '#E6E6E6',        // Light gray for equipment
  
  // Alarms per ISA-101 priority levels
  alarmCritical: '#FF0000',        // Red - Critical/Safety
  alarmHigh: '#FF8C00',            // Orange - High priority
  alarmMedium: '#FFFF00',          // Yellow - Medium priority
  alarmLow: '#00FFFF',             // Cyan - Low priority
  alarmMessage: '#C0C0C0',         // Gray - Message/Event
  
  // Status indication
  running: '#00FF00',              // Green - Running/Active
  stopped: '#808080',              // Gray - Stopped
  
  // Process values
  processValue: '#000000',          // Black text for values
  setpoint: '#0000FF',             // Blue for setpoints
  
  // Liquid/Material (subdued colors per ISA-101)
  liquidNormal: '#87CEEB',         // Sky blue - normal liquid
  liquidHot: '#FFB6C1',            // Light red - hot liquid
  liquidChemical: '#DDA0DD',       // Plum - chemical
  
  // Background
  background: '#F0F0F0',           // Light gray background
  containerBackground: '#FFFFFF',   // White for containers
}

interface ISA101TankProps {
  x: number
  y: number
  width: number
  height: number
  properties: {
    tagName: string              // Equipment tag (e.g., "TK-101")
    currentLevel: number         // Current level value
    levelUnits: string          // Engineering units
    maxLevel: number            // Max scale value
    minLevel: number            // Min scale value
    
    // Alarm limits per ISA-101
    criticalHigh?: number       // HH - Critical high
    alarmHigh?: number          // H - High alarm
    alarmLow?: number           // L - Low alarm
    criticalLow?: number        // LL - Critical low
    
    // Status
    alarmState?: 'none' | 'low' | 'high' | 'criticalLow' | 'criticalHigh'
    alarmAcknowledged?: boolean
    
    // Display options
    showTrend?: boolean         // Show sparkline trend
    showAlarmLimits?: boolean   // Show alarm setpoints
    showNormalBand?: boolean    // Show normal operating range
    
    // Material properties
    materialType?: 'water' | 'chemical' | 'hot' | 'oil'
    temperature?: number
    
    // Additional process data
    inletValveOpen?: boolean
    outletValveOpen?: boolean
    agitatorRunning?: boolean
  }
  style?: {
    lineWidth?: number
  }
  selected?: boolean
  onContextMenu?: (e: any) => void
}

export default function ISA101TankComponent({
  x,
  y,
  width,
  height,
  properties,
  style = {},
  selected = false,
  onContextMenu
}: ISA101TankProps) {
  const [animatedLevel, setAnimatedLevel] = useState(properties.currentLevel)
  const [trendData, setTrendData] = useState<number[]>([])
  const [blinkState, setBlinkState] = useState(true)
  
  // Animate level changes smoothly
  useEffect(() => {
    const timer = setInterval(() => {
      setAnimatedLevel(prev => {
        const diff = properties.currentLevel - prev
        if (Math.abs(diff) < 0.1) return properties.currentLevel
        return prev + diff * 0.1
      })
    }, 50)
    
    return () => clearInterval(timer)
  }, [properties.currentLevel])
  
  // Update trend data
  useEffect(() => {
    setTrendData(prev => {
      const updated = [...prev, properties.currentLevel]
      return updated.slice(-20) // Keep last 20 points
    })
  }, [properties.currentLevel])
  
  // Blink animation for unacknowledged alarms
  useEffect(() => {
    if (properties.alarmState !== 'none' && !properties.alarmAcknowledged) {
      const timer = setInterval(() => {
        setBlinkState(prev => !prev)
      }, 500)
      return () => clearInterval(timer)
    } else {
      setBlinkState(true)
    }
  }, [properties.alarmState, properties.alarmAcknowledged])
  
  // Calculate level percentage and pixel position
  const levelRange = properties.maxLevel - properties.minLevel
  const levelPercent = ((animatedLevel - properties.minLevel) / levelRange) * 100
  const tankBodyHeight = height * 0.7
  const tankBodyY = height * 0.2
  const levelHeight = (tankBodyHeight * levelPercent) / 100
  const levelY = tankBodyY + tankBodyHeight - levelHeight
  
  // Determine liquid color based on material type and temperature
  const getLiquidColor = () => {
    if (properties.materialType === 'hot' || (properties.temperature && properties.temperature > 60)) {
      return ISA101Colors.liquidHot
    }
    if (properties.materialType === 'chemical') {
      return ISA101Colors.liquidChemical
    }
    return ISA101Colors.liquidNormal
  }
  
  // Get alarm color based on state
  const getAlarmColor = () => {
    switch (properties.alarmState) {
      case 'criticalHigh':
      case 'criticalLow':
        return ISA101Colors.alarmCritical
      case 'high':
      case 'low':
        return ISA101Colors.alarmHigh
      default:
        return null
    }
  }
  
  const lineWidth = style.lineWidth || 2
  const alarmColor = getAlarmColor()
  const showAlarm = alarmColor && blinkState
  
  return (
    <Group x={x} y={y} onContextMenu={onContextMenu}>
      {/* Selection indicator */}
      {selected && (
        <Rect
          x={-10}
          y={-10}
          width={width + 20}
          height={height + 20}
          stroke="#0080FF"
          strokeWidth={2}
          dash={[5, 5]}
          fill="transparent"
        />
      )}
      
      {/* Tank body background */}
      <Rect
        x={0}
        y={tankBodyY}
        width={width}
        height={tankBodyHeight}
        fill={ISA101Colors.containerBackground}
        stroke={ISA101Colors.equipmentOutline}
        strokeWidth={lineWidth}
      />
      
      {/* Normal operating band (if enabled) */}
      {properties.showNormalBand && properties.alarmHigh && properties.alarmLow && (
        <Rect
          x={2}
          y={tankBodyY + tankBodyHeight * (1 - (properties.alarmHigh - properties.minLevel) / levelRange)}
          width={width - 4}
          height={tankBodyHeight * ((properties.alarmHigh - properties.alarmLow) / levelRange)}
          fill="#E8F5E9"
          opacity={0.3}
        />
      )}
      
      {/* Liquid level */}
      <Rect
        x={2}
        y={levelY}
        width={width - 4}
        height={levelHeight - 2}
        fill={getLiquidColor()}
        opacity={0.7}
      />
      
      {/* Tank top */}
      <Shape
        sceneFunc={(context, shape) => {
          context.beginPath()
          context.moveTo(0, tankBodyY)
          context.lineTo(width * 0.2, 0)
          context.lineTo(width * 0.8, 0)
          context.lineTo(width, tankBodyY)
          context.closePath()
          context.fillStrokeShape(shape)
        }}
        fill={ISA101Colors.equipmentFill}
        stroke={ISA101Colors.equipmentOutline}
        strokeWidth={lineWidth}
      />
      
      {/* Tank bottom */}
      <Shape
        sceneFunc={(context, shape) => {
          context.beginPath()
          context.moveTo(0, tankBodyY + tankBodyHeight)
          context.lineTo(width * 0.2, height)
          context.lineTo(width * 0.8, height)
          context.lineTo(width, tankBodyY + tankBodyHeight)
          context.closePath()
          context.fillStrokeShape(shape)
        }}
        fill={ISA101Colors.equipmentFill}
        stroke={ISA101Colors.equipmentOutline}
        strokeWidth={lineWidth}
      />
      
      {/* Alarm limit indicators */}
      {properties.showAlarmLimits && (
        <Group>
          {/* Critical High (HH) */}
          {properties.criticalHigh && (
            <Group y={tankBodyY + tankBodyHeight * (1 - (properties.criticalHigh - properties.minLevel) / levelRange)}>
              <Line
                points={[0, 0, width, 0]}
                stroke={ISA101Colors.alarmCritical}
                strokeWidth={2}
                dash={[4, 4]}
              />
              <Text
                x={width + 5}
                y={-6}
                text="HH"
                fontSize={10}
                fill={ISA101Colors.alarmCritical}
                fontFamily="Arial"
              />
            </Group>
          )}
          
          {/* High (H) */}
          {properties.alarmHigh && (
            <Group y={tankBodyY + tankBodyHeight * (1 - (properties.alarmHigh - properties.minLevel) / levelRange)}>
              <Line
                points={[0, 0, width, 0]}
                stroke={ISA101Colors.alarmHigh}
                strokeWidth={1}
                dash={[4, 4]}
              />
              <Text
                x={width + 5}
                y={-6}
                text="H"
                fontSize={10}
                fill={ISA101Colors.alarmHigh}
                fontFamily="Arial"
              />
            </Group>
          )}
          
          {/* Low (L) */}
          {properties.alarmLow && (
            <Group y={tankBodyY + tankBodyHeight * (1 - (properties.alarmLow - properties.minLevel) / levelRange)}>
              <Line
                points={[0, 0, width, 0]}
                stroke={ISA101Colors.alarmHigh}
                strokeWidth={1}
                dash={[4, 4]}
              />
              <Text
                x={width + 5}
                y={-6}
                text="L"
                fontSize={10}
                fill={ISA101Colors.alarmHigh}
                fontFamily="Arial"
              />
            </Group>
          )}
          
          {/* Critical Low (LL) */}
          {properties.criticalLow && (
            <Group y={tankBodyY + tankBodyHeight * (1 - (properties.criticalLow - properties.minLevel) / levelRange)}>
              <Line
                points={[0, 0, width, 0]}
                stroke={ISA101Colors.alarmCritical}
                strokeWidth={2}
                dash={[4, 4]}
              />
              <Text
                x={width + 5}
                y={-6}
                text="LL"
                fontSize={10}
                fill={ISA101Colors.alarmCritical}
                fontFamily="Arial"
              />
            </Group>
          )}
        </Group>
      )}
      
      {/* Equipment tag */}
      <Text
        x={0}
        y={-20}
        width={width}
        text={properties.tagName}
        fontSize={12}
        fontFamily="Arial"
        fontStyle="bold"
        fill={ISA101Colors.processValue}
        align="center"
      />
      
      {/* Process value display */}
      <Group y={height + 10}>
        <Rect
          x={0}
          y={0}
          width={width}
          height={25}
          fill={showAlarm ? alarmColor : ISA101Colors.containerBackground}
          stroke={ISA101Colors.equipmentOutline}
          strokeWidth={1}
        />
        <Text
          x={0}
          y={5}
          width={width}
          text={`${animatedLevel.toFixed(1)} ${properties.levelUnits}`}
          fontSize={14}
          fontFamily="Arial"
          fontStyle="bold"
          fill={showAlarm ? '#FFFFFF' : ISA101Colors.processValue}
          align="center"
        />
      </Group>
      
      {/* Trend sparkline (if enabled) */}
      {properties.showTrend && trendData.length > 1 && (
        <Group x={5} y={height + 40}>
          <Shape
            sceneFunc={(context, shape) => {
              const trendWidth = width - 10
              const trendHeight = 20
              
              context.beginPath()
              context.strokeStyle = '#666666'
              context.lineWidth = 1
              
              trendData.forEach((value, index) => {
                const x = (index / (trendData.length - 1)) * trendWidth
                const y = trendHeight - ((value - properties.minLevel) / levelRange) * trendHeight
                
                if (index === 0) {
                  context.moveTo(x, y)
                } else {
                  context.lineTo(x, y)
                }
              })
              
              context.stroke()
            }}
          />
        </Group>
      )}
      
      {/* Inlet/Outlet indicators */}
      <Group>
        {/* Inlet */}
        <Line
          points={[width, tankBodyY + 20, width + 20, tankBodyY + 20]}
          stroke={properties.inletValveOpen ? ISA101Colors.running : ISA101Colors.stopped}
          strokeWidth={3}
        />
        
        {/* Outlet */}
        <Line
          points={[width / 2, height, width / 2, height + 20]}
          stroke={properties.outletValveOpen ? ISA101Colors.running : ISA101Colors.stopped}
          strokeWidth={3}
        />
      </Group>
      
      {/* Agitator indicator (if present) */}
      {properties.agitatorRunning !== undefined && (
        <Group x={width / 2} y={tankBodyY - 10}>
          <Circle
            x={0}
            y={0}
            radius={8}
            fill={properties.agitatorRunning ? ISA101Colors.running : ISA101Colors.stopped}
            stroke={ISA101Colors.equipmentOutline}
            strokeWidth={1}
          />
          <Text
            x={-4}
            y={-4}
            text="M"
            fontSize={8}
            fill={ISA101Colors.equipmentOutline}
            fontFamily="Arial"
          />
        </Group>
      )}
    </Group>
  )
}
