// File: petra-designer/src/components/hmi/components/ISA101ValveComponent.tsx
// ISA-101 Compliant Valve Component following HMI standards
import { useEffect, useState } from 'react'
import { Group, Line, Shape, Text, Rect, Circle } from 'react-konva'

const ISA101Colors = {
  equipmentOutline: '#000000',
  equipmentFill: '#E6E6E6',
  processLine: '#000000',
  open: '#00FF00',
  closed: '#808080',
  transitioning: '#FFFF00',
  fault: '#FF0000',
  processValue: '#000000',
  containerBackground: '#FFFFFF',
  alarmCritical: '#FF0000',
  alarmHigh: '#FF8C00',
}

interface ISA101ValveProps {
  x: number
  y: number
  width: number
  height: number
  properties: {
    tagName: string
    position: number  // 0-100%
    commandPosition?: number  // Target position
    
    // Valve states
    status: 'open' | 'closed' | 'transitioning' | 'fault'
    valveType: 'gate' | 'ball' | 'butterfly' | 'control' | 'globe'
    actuatorType?: 'manual' | 'pneumatic' | 'electric' | 'hydraulic'
    
    // Fault/Alarm states
    failPosition?: 'open' | 'closed' | 'last' | 'none'
    travelAlarm?: boolean
    positionDeviation?: boolean
    
    // Control
    controlMode?: 'local' | 'remote' | 'cascade' | 'manual'
    interlocked?: boolean
    
    // Display options
    showPosition?: boolean
    showFailPosition?: boolean
    orientation?: 'horizontal' | 'vertical'
  }
  style?: {
    lineWidth?: number
  }
  selected?: boolean
  onContextMenu?: (e: any) => void
  onClick?: () => void
  [key: string]: any
}

export default function ISA101ValveComponent({
  x,
  y,
  width,
  height,
  properties,
  style = {},
  selected = false,
  onContextMenu,
  onClick,
  ...rest
}: ISA101ValveProps) {
  const [blinkState, setBlinkState] = useState(true)
  
  // Blink for alarms
  useEffect(() => {
    if (properties.travelAlarm || properties.positionDeviation) {
      const timer = setInterval(() => {
        setBlinkState(prev => !prev)
      }, 500)
      return () => clearInterval(timer)
    } else {
      setBlinkState(true)
    }
  }, [properties.travelAlarm, properties.positionDeviation])
  
  const lineWidth = style.lineWidth || 2
  const centerX = width / 2
  const centerY = height / 2
  const isVertical = properties.orientation === 'vertical'
  
  // Get valve body color based on position
  const getValveColor = () => {
    if (properties.status === 'fault') return ISA101Colors.fault
    if (properties.status === 'transitioning') return ISA101Colors.transitioning
    if (properties.position > 95) return ISA101Colors.open
    if (properties.position < 5) return ISA101Colors.closed
    // Partially open - blend between open and closed
    return ISA101Colors.transitioning
  }
  
  // Draw valve symbol based on type
  const drawValveSymbol = (context: any, shape: any) => {
    context.strokeStyle = ISA101Colors.equipmentOutline
    context.lineWidth = lineWidth
    context.fillStyle = getValveColor()
    
    switch (properties.valveType) {
      case 'gate':
        // Gate valve symbol
        if (isVertical) {
          context.beginPath()
          context.moveTo(centerX - 15, centerY - 10)
          context.lineTo(centerX - 15, centerY + 10)
          context.lineTo(centerX + 15, centerY + 10)
          context.lineTo(centerX + 15, centerY - 10)
          context.closePath()
          context.fill()
          context.stroke()
          
          // Stem
          context.beginPath()
          context.moveTo(centerX, centerY - 10)
          context.lineTo(centerX, centerY - 20)
          context.stroke()
        } else {
          context.beginPath()
          context.moveTo(centerX - 10, centerY - 15)
          context.lineTo(centerX - 10, centerY + 15)
          context.lineTo(centerX + 10, centerY + 15)
          context.lineTo(centerX + 10, centerY - 15)
          context.closePath()
          context.fill()
          context.stroke()
          
          // Stem
          context.beginPath()
          context.moveTo(centerX, centerY - 15)
          context.lineTo(centerX, centerY - 25)
          context.stroke()
        }
        break
        
      case 'ball':
        // Ball valve symbol
        context.beginPath()
        context.arc(centerX, centerY, 12, 0, Math.PI * 2)
        context.fill()
        context.stroke()
        
        // Ball position indicator
        context.save()
        context.translate(centerX, centerY)
        context.rotate((properties.position / 100) * Math.PI / 2)
        context.beginPath()
        context.moveTo(-8, 0)
        context.lineTo(8, 0)
        context.strokeStyle = ISA101Colors.equipmentOutline
        context.lineWidth = 3
        context.stroke()
        context.restore()
        break
        
      case 'butterfly':
        // Butterfly valve symbol
        context.beginPath()
        context.arc(centerX, centerY, 12, 0, Math.PI * 2)
        context.stroke()
        
        // Disc
        context.save()
        context.translate(centerX, centerY)
        context.rotate((properties.position / 100) * Math.PI / 2)
        context.beginPath()
        context.ellipse(0, 0, 10, 3, 0, 0, Math.PI * 2)
        context.fillStyle = getValveColor()
        context.fill()
        context.stroke()
        context.restore()
        break
        
      case 'control':
      case 'globe':
        // Control/Globe valve symbol
        context.beginPath()
        context.moveTo(centerX - 12, centerY - 12)
        context.lineTo(centerX, centerY)
        context.lineTo(centerX + 12, centerY - 12)
        context.lineTo(centerX + 12, centerY + 12)
        context.lineTo(centerX - 12, centerY + 12)
        context.closePath()
        context.fill()
        context.stroke()
        
        // Stem with position indicator
        const stemY = centerY - 12 - (properties.position / 100) * 10
        context.beginPath()
        context.moveTo(centerX, centerY - 12)
        context.lineTo(centerX, stemY - 10)
        context.stroke()
        
        // Position indicator
        context.beginPath()
        context.moveTo(centerX - 5, stemY)
        context.lineTo(centerX + 5, stemY)
        context.lineWidth = 3
        context.stroke()
        break
    }
  }
  
  // Draw actuator based on type
  const drawActuator = (context: any) => {
    if (!properties.actuatorType || properties.actuatorType === 'manual') return
    
    context.strokeStyle = ISA101Colors.equipmentOutline
    context.lineWidth = lineWidth
    context.fillStyle = ISA101Colors.equipmentFill
    
    const actuatorY = isVertical ? centerY - 35 : centerY - 40
    
    switch (properties.actuatorType) {
      case 'pneumatic':
        // Diaphragm actuator
        context.beginPath()
        context.arc(centerX, actuatorY, 15, Math.PI, 0, true)
        context.closePath()
        context.fill()
        context.stroke()
        break
        
      case 'electric':
        // Motor actuator
        context.beginPath()
        context.rect(centerX - 12, actuatorY - 12, 24, 24)
        context.fill()
        context.stroke()
        
        // M symbol
        context.fillStyle = ISA101Colors.processValue
        context.font = 'bold 10px Arial'
        context.textAlign = 'center'
        context.textBaseline = 'middle'
        context.fillText('M', centerX, actuatorY)
        break
        
      case 'hydraulic':
        // Hydraulic cylinder
        context.beginPath()
        context.rect(centerX - 10, actuatorY - 15, 20, 20)
        context.fill()
        context.stroke()
        
        // H symbol
        context.fillStyle = ISA101Colors.processValue
        context.font = 'bold 10px Arial'
        context.textAlign = 'center'
        context.textBaseline = 'middle'
        context.fillText('H', centerX, actuatorY - 5)
        break
    }
  }
  
  const showAlarm = (properties.travelAlarm || properties.positionDeviation) && blinkState
  
  return (
    <Group x={x} y={y} onClick={onClick} onContextMenu={onContextMenu} {...rest}>
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
      
      {/* Process lines */}
      <Group>
        {isVertical ? (
          <>
            <Line
              points={[centerX, 0, centerX, centerY - 15]}
              stroke={ISA101Colors.processLine}
              strokeWidth={3}
            />
            <Line
              points={[centerX, centerY + 15, centerX, height]}
              stroke={ISA101Colors.processLine}
              strokeWidth={3}
            />
          </>
        ) : (
          <>
            <Line
              points={[0, centerY, centerX - 15, centerY]}
              stroke={ISA101Colors.processLine}
              strokeWidth={3}
            />
            <Line
              points={[centerX + 15, centerY, width, centerY]}
              stroke={ISA101Colors.processLine}
              strokeWidth={3}
            />
          </>
        )}
      </Group>
      
      {/* Valve body */}
      <Shape sceneFunc={drawValveSymbol} />
      
      {/* Actuator */}
      <Shape sceneFunc={drawActuator} />
      
      {/* Equipment tag */}
      <Text
        x={0}
        y={-45}
        width={width}
        text={properties.tagName}
        fontSize={12}
        fontFamily="Arial"
        fontStyle="bold"
        fill={ISA101Colors.processValue}
        align="center"
      />
      
      {/* Position display */}
      {properties.showPosition && (
        <Group y={height + 5}>
          <Rect
            x={centerX - 25}
            y={0}
            width={50}
            height={18}
            fill={showAlarm ? ISA101Colors.alarmHigh : ISA101Colors.containerBackground}
            stroke={ISA101Colors.equipmentOutline}
            strokeWidth={1}
          />
          <Text
            x={centerX - 25}
            y={2}
            width={50}
            text={`${properties.position.toFixed(0)}%`}
            fontSize={11}
            fontFamily="Arial"
            fontStyle="bold"
            fill={showAlarm ? '#FFFFFF' : ISA101Colors.processValue}
            align="center"
          />
        </Group>
      )}
      
      {/* Command position indicator */}
      {properties.commandPosition !== undefined && 
       Math.abs(properties.commandPosition - properties.position) > 2 && (
        <Group y={height + 25}>
          <Text
            x={0}
            y={0}
            width={width}
            text={`CMD: ${properties.commandPosition.toFixed(0)}%`}
            fontSize={10}
            fontFamily="Arial"
            fill={ISA101Colors.transitioning}
            align="center"
          />
        </Group>
      )}
      
      {/* Fail position indicator */}
      {properties.showFailPosition && properties.failPosition && (
        <Group x={centerX + 20} y={centerY - 20}>
          <Circle
            x={0}
            y={0}
            radius={10}
            fill={ISA101Colors.containerBackground}
            stroke={ISA101Colors.equipmentOutline}
            strokeWidth={1}
          />
          <Text
            x={-8}
            y={-6}
            width={16}
            text={properties.failPosition === 'open' ? 'FO' : 
                  properties.failPosition === 'closed' ? 'FC' : 
                  properties.failPosition === 'last' ? 'FL' : ''}
            fontSize={8}
            fontFamily="Arial"
            fill={ISA101Colors.processValue}
            align="center"
          />
        </Group>
      )}
      
      {/* Interlock indicator */}
      {properties.interlocked && (
        <Group x={centerX - 30} y={centerY - 20}>
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
            text="I"
            fontSize={10}
            fontFamily="Arial"
            fontStyle="bold"
            fill="#FFFFFF"
          />
        </Group>
      )}
    </Group>
  )
}
