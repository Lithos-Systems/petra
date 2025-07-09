// File: petra-designer/src/components/hmi/components/ISA101PumpComponent.tsx
// ISA-101 Compliant Pump Component following HMI standards
import { useEffect, useState } from 'react'
import { Group, Circle, Line, Text, Rect, Shape, Wedge } from 'react-konva'

// ISA-101 Standard Colors
const ISA101Colors = {
  processLine: '#000000',
  equipmentOutline: '#000000',
  equipmentFill: '#E6E6E6',
  running: '#00FF00',
  stopped: '#808080',
  fault: '#FF0000',
  processValue: '#000000',
  background: '#F0F0F0',
  containerBackground: '#FFFFFF',
  // Alarm colors
  alarmCritical: '#FF0000',
  alarmHigh: '#FF8C00',
  alarmMedium: '#FFFF00',
  alarmLow: '#00FFFF',
}

interface ISA101PumpProps {
  x: number
  y: number
  width: number
  height: number
  properties: {
    tagName: string                    // Equipment tag (e.g., "P-101A")
    status: 'running' | 'stopped' | 'fault'
    
    // Process values
    flowRate?: number                  // Current flow rate
    flowUnits?: string                 // Flow units (e.g., "GPM")
    dischargePressure?: number         // Discharge pressure
    pressureUnits?: string             // Pressure units (e.g., "PSI")
    speed?: number                     // Speed percentage (0-100)
    current?: number                   // Motor current
    
    // Alarm states
    alarmState?: 'none' | 'warning' | 'fault' | 'trip'
    alarmAcknowledged?: boolean
    
    // Interlock status
    interlocked?: boolean
    interlockReason?: string
    
    // Control mode
    controlMode?: 'local' | 'remote' | 'cascade' | 'manual'
    runPermissive?: boolean
    
    // Additional status
    sealFlushPressure?: number
    bearingTemp?: number
    vibration?: number
    
    // Display options
    showFlowDirection?: boolean
    showDetailedStatus?: boolean
    pumpType?: 'centrifugal' | 'positive-displacement'
  }
  style?: {
    lineWidth?: number
  }
  selected?: boolean
  onContextMenu?: (e: any) => void
  onClick?: () => void
}

export default function ISA101PumpComponent({
  x,
  y,
  width,
  height,
  properties,
  style = {},
  selected = false,
  onContextMenu,
  onClick
}: ISA101PumpProps) {
  const [blinkState, setBlinkState] = useState(true)
  const [rotation, setRotation] = useState(0)
  
  // Blink for unacknowledged alarms
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
  
  // Rotation animation for running pumps
  useEffect(() => {
    if (properties.status === 'running') {
      const timer = setInterval(() => {
        setRotation(prev => (prev + 5) % 360)
      }, 50)
      return () => clearInterval(timer)
    }
  }, [properties.status])
  
  const lineWidth = style.lineWidth || 2
  const centerX = width / 2
  const centerY = height / 2
  const radius = Math.min(width, height) / 2 - 10
  
  // Determine fill color based on status
  const getFillColor = () => {
    if (properties.status === 'fault') return ISA101Colors.fault
    if (properties.status === 'running') return ISA101Colors.running
    return ISA101Colors.stopped
  }
  
  // Get status text
  const getStatusText = () => {
    if (properties.interlocked) return 'INTLK'
    if (properties.status === 'fault') return 'FAULT'
    if (properties.status === 'running') return 'RUN'
    return 'STOP'
  }
  
  const fillColor = getFillColor()
  const showAlarm = properties.alarmState !== 'none' && blinkState
  
  return (
    <Group x={x} y={y} onClick={onClick} onContextMenu={onContextMenu}>
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
      
      {/* Pump casing - ISA-101 simplified representation */}
      <Circle
        x={centerX}
        y={centerY}
        radius={radius}
        fill={ISA101Colors.equipmentFill}
        stroke={ISA101Colors.equipmentOutline}
        strokeWidth={lineWidth}
      />
      
      {/* Pump impeller (simplified for ISA-101) */}
      {properties.pumpType === 'centrifugal' ? (
        <Group x={centerX} y={centerY} rotation={rotation}>
          {/* Centrifugal impeller vanes */}
          {[0, 120, 240].map(angle => (
            <Wedge
              key={angle}
              x={0}
              y={0}
              radius={radius * 0.7}
              angle={60}
              rotation={angle}
              fill={ISA101Colors.containerBackground}
              stroke={ISA101Colors.equipmentOutline}
              strokeWidth={1}
            />
          ))}
        </Group>
      ) : (
        /* Positive displacement representation */
        <Group x={centerX} y={centerY}>
          <Rect
            x={-radius * 0.5}
            y={-radius * 0.3}
            width={radius}
            height={radius * 0.6}
            fill={ISA101Colors.containerBackground}
            stroke={ISA101Colors.equipmentOutline}
            strokeWidth={1}
            rotation={rotation}
          />
        </Group>
      )}
      
      {/* Center hub with status color */}
      <Circle
        x={centerX}
        y={centerY}
        radius={radius * 0.3}
        fill={fillColor}
        stroke={ISA101Colors.equipmentOutline}
        strokeWidth={lineWidth}
      />
      
      {/* Flow direction arrows (ISA-101 style) */}
      {properties.showFlowDirection && properties.status === 'running' && (
        <Group>
          {/* Inlet arrow */}
          <Shape
            sceneFunc={(context, shape) => {
              context.beginPath()
              context.moveTo(-radius - 15, centerY)
              context.lineTo(-radius - 5, centerY)
              context.moveTo(-radius - 10, centerY - 5)
              context.lineTo(-radius - 5, centerY)
              context.lineTo(-radius - 10, centerY + 5)
              context.strokeStyle = ISA101Colors.processValue
              context.lineWidth = 2
              context.stroke()
            }}
          />
          
          {/* Outlet arrow */}
          <Shape
            sceneFunc={(context, shape) => {
              context.beginPath()
              context.moveTo(width + radius + 5, centerY)
              context.lineTo(width + radius + 15, centerY)
              context.moveTo(width + radius + 10, centerY - 5)
              context.lineTo(width + radius + 15, centerY)
              context.lineTo(width + radius + 10, centerY + 5)
              context.strokeStyle = ISA101Colors.processValue
              context.lineWidth = 2
              context.stroke()
            }}
          />
        </Group>
      )}
      
      {/* Equipment tag */}
      <Text
        x={0}
        y={-radius - 25}
        width={width}
        text={properties.tagName}
        fontSize={12}
        fontFamily="Arial"
        fontStyle="bold"
        fill={ISA101Colors.processValue}
        align="center"
      />
      
      {/* Status display box */}
      <Group y={radius + 10}>
        <Rect
          x={centerX - 30}
          y={0}
          width={60}
          height={20}
          fill={showAlarm ? 
            (properties.alarmState === 'fault' || properties.alarmState === 'trip' ? 
              ISA101Colors.alarmCritical : ISA101Colors.alarmHigh) : 
            ISA101Colors.containerBackground}
          stroke={ISA101Colors.equipmentOutline}
          strokeWidth={1}
        />
        <Text
          x={centerX - 30}
          y={3}
          width={60}
          text={getStatusText()}
          fontSize={11}
          fontFamily="Arial"
          fontStyle="bold"
          fill={showAlarm ? '#FFFFFF' : ISA101Colors.processValue}
          align="center"
        />
      </Group>
      
      {/* Control mode indicator */}
      {properties.controlMode && (
        <Group y={radius + 35}>
          <Text
            x={0}
            y={0}
            width={width}
            text={properties.controlMode.toUpperCase()}
            fontSize={10}
            fontFamily="Arial"
            fill={ISA101Colors.processValue}
            align="center"
          />
        </Group>
      )}
      
      {/* Process values display */}
      {properties.showDetailedStatus && (
        <Group y={radius + 50}>
          {/* Flow rate */}
          {properties.flowRate !== undefined && (
            <Group y={0}>
              <Text
                x={0}
                y={0}
                width={width}
                text={`${properties.flowRate.toFixed(1)} ${properties.flowUnits || 'GPM'}`}
                fontSize={10}
                fontFamily="Arial"
                fill={ISA101Colors.processValue}
                align="center"
              />
            </Group>
          )}
          
          {/* Discharge pressure */}
          {properties.dischargePressure !== undefined && (
            <Group y={12}>
              <Text
                x={0}
                y={0}
                width={width}
                text={`${properties.dischargePressure.toFixed(0)} ${properties.pressureUnits || 'PSI'}`}
                fontSize={10}
                fontFamily="Arial"
                fill={ISA101Colors.processValue}
                align="center"
              />
            </Group>
          )}
          
          {/* Speed */}
          {properties.speed !== undefined && (
            <Group y={24}>
              <Text
                x={0}
                y={0}
                width={width}
                text={`${properties.speed.toFixed(0)}% SPD`}
                fontSize={10}
                fontFamily="Arial"
                fill={ISA101Colors.processValue}
                align="center"
              />
            </Group>
          )}
        </Group>
      )}
      
      {/* Interlock indicator */}
      {properties.interlocked && (
        <Group x={centerX + radius - 10} y={centerY - radius - 10}>
          <Circle
            x={0}
            y={0}
            radius={8}
            fill={ISA101Colors.alarmMedium}
            stroke={ISA101Colors.equipmentOutline}
            strokeWidth={1}
          />
          <Text
            x={-3}
            y={-4}
            text="I"
            fontSize={10}
            fontFamily="Arial"
            fontStyle="bold"
            fill={ISA101Colors.equipmentOutline}
          />
        </Group>
      )}
      
      {/* Permissive indicator */}
      {properties.runPermissive === false && (
        <Group x={centerX - radius + 10} y={centerY - radius - 10}>
          <Circle
            x={0}
            y={0}
            radius={8}
            fill={ISA101Colors.alarmHigh}
            stroke={ISA101Colors.equipmentOutline}
            strokeWidth={1}
          />
          <Text
            x={-3}
            y={-4}
            text="P"
            fontSize={10}
            fontFamily="Arial"
            fontStyle="bold"
            fill="#FFFFFF"
          />
        </Group>
      )}
      
      {/* Process connection lines */}
      <Group>
        {/* Suction line */}
        <Line
          points={[-radius - 5, centerY, 0, centerY]}
          stroke={ISA101Colors.processLine}
          strokeWidth={3}
        />
        
        {/* Discharge line */}
        <Line
          points={[width, centerY, width + radius + 5, centerY]}
          stroke={ISA101Colors.processLine}
          strokeWidth={3}
        />
      </Group>
    </Group>
  )
}
