import { useEffect, useState, useRef } from 'react'
import { Group, Rect, Shape, Text, Line } from 'react-konva'

// ISA-101 Standard Colors
const ISA101Colors = {
  // Equipment and process lines
  equipmentOutline: '#000000',      // Black outlines
  equipmentFill: '#E6E6E6',         // Light gray fill
  
  // Alarms per ISA-101 priority levels
  alarmCritical: '#FF0000',         // Red - Critical/Safety
  alarmHigh: '#FF8C00',             // Orange - High priority
  alarmMedium: '#FFFF00',           // Yellow - Medium priority
  alarmLow: '#00FFFF',              // Cyan - Low priority
  
  // Status indication
  running: '#00FF00',               // Green - Running/Active
  stopped: '#808080',               // Gray - Stopped
  
  // Process values
  processValue: '#000000',          // Black text
  setpoint: '#0000FF',              // Blue for setpoints
  
  // Liquid/Material (subdued per ISA-101)
  liquidNormal: '#87CEEB',          // Sky blue
  liquidHot: '#FFB6C1',             // Light red
  liquidChemical: '#DDA0DD',        // Plum
  
  // Background
  background: '#F0F0F0',            // Light gray
  containerBackground: '#FFFFFF',    // White
}

interface ISA101TankProps {
  x: number
  y: number
  width: number
  height: number
  properties: {
    tagName?: string
    currentLevel: number
    levelUnits?: string
    maxLevel: number
    minLevel: number
    criticalHigh?: number
    alarmHigh?: number
    alarmLow?: number
    criticalLow?: number
    alarmState?: 'none' | 'low' | 'high' | 'criticalLow' | 'criticalHigh'
    alarmAcknowledged?: boolean
    showTrend?: boolean
    showAlarmLimits?: boolean
    showNormalBand?: boolean
    materialType?: 'water' | 'chemical' | 'hot' | 'oil'
    temperature?: number
    inletValveOpen?: boolean
    outletValveOpen?: boolean
    agitatorRunning?: boolean
    [key: string]: any
  }
  style?: {
    lineWidth?: number
    [key: string]: any
  }
  selected?: boolean
  onContextMenu?: (e: any) => void
  onClick?: () => void
  draggable?: boolean
  onDragEnd?: (e: any) => void
  onDragStart?: (e: any) => void
}

export default function ISA101TankComponent({
  x,
  y,
  width,
  height,
  properties,
  style = {},
  selected = false,
  onContextMenu,
  onClick,
  draggable = true,
  onDragEnd,
  onDragStart,
  ...restProps
}: ISA101TankProps) {
  const [isBlinking, setIsBlinking] = useState(false)
  const groupRef = useRef<any>()
  const liquidAnimationRef = useRef(0)

  // Calculate liquid fill height
  const levelPercent = ((properties.currentLevel - properties.minLevel) / 
                       (properties.maxLevel - properties.minLevel)) * 100
  const liquidHeight = (height * 0.85) * (levelPercent / 100)

  // Determine liquid color based on material type
  const getLiquidColor = () => {
    switch (properties.materialType) {
      case 'hot': return ISA101Colors.liquidHot
      case 'chemical': return ISA101Colors.liquidChemical
      case 'oil': return '#8B7355'
      default: return ISA101Colors.liquidNormal
    }
  }

  // Determine alarm color
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

  // Handle alarm blinking
  useEffect(() => {
    if (properties.alarmState !== 'none' && !properties.alarmAcknowledged) {
      const interval = setInterval(() => {
        setIsBlinking(prev => !prev)
      }, 500)
      return () => clearInterval(interval)
    } else {
      setIsBlinking(false)
    }
  }, [properties.alarmState, properties.alarmAcknowledged])

  // Animate liquid surface
  useEffect(() => {
    if (properties.agitatorRunning || properties.inletValveOpen) {
      const animate = () => {
        liquidAnimationRef.current = (liquidAnimationRef.current + 0.1) % (Math.PI * 2)
        if (groupRef.current) {
          groupRef.current.getLayer()?.batchDraw()
        }
      }
      const animationId = setInterval(animate, 50)
      return () => clearInterval(animationId)
    }
  }, [properties.agitatorRunning, properties.inletValveOpen])

  const tankBottomY = height * 0.85
  const alarmColor = getAlarmColor()
  const showAlarm = alarmColor && (!properties.alarmAcknowledged || isBlinking)

  return (
    <Group
      ref={groupRef}
      x={x}
      y={y}
      draggable={draggable}
      onDragEnd={onDragEnd}
      onDragStart={(e) => {
        e.target.moveToTop()
        onDragStart?.(e)
      }}
      onClick={onClick}
      onContextMenu={onContextMenu}
      {...restProps}
    >
      {/* Selection indicator */}
      {selected && (
        <Rect
          x={-5}
          y={-5}
          width={width + 10}
          height={height + 10}
          stroke="#0080FF"
          strokeWidth={2}
          dash={[5, 5]}
          fill="transparent"
        />
      )}

      {/* Tank shell - ISA-101 style (simple, no gradients) */}
      <Shape
        sceneFunc={(ctx, shape) => {
          // Tank body
          ctx.beginPath()
          ctx.moveTo(5, 20)
          ctx.lineTo(5, tankBottomY - 10)
          ctx.quadraticCurveTo(5, tankBottomY, 15, tankBottomY)
          ctx.lineTo(width - 15, tankBottomY)
          ctx.quadraticCurveTo(width - 5, tankBottomY, width - 5, tankBottomY - 10)
          ctx.lineTo(width - 5, 20)
          
          // Tank top (elliptical)
          ctx.ellipse(width / 2, 20, (width - 10) / 2, 10, 0, 0, Math.PI, true)
          
          ctx.closePath()
          ctx.fillStrokeShape(shape)
        }}
        fill={ISA101Colors.equipmentFill}
        stroke={showAlarm ? alarmColor : ISA101Colors.equipmentOutline}
        strokeWidth={showAlarm ? 3 : (style.lineWidth || 2)}
      />

      {/* Liquid fill */}
      {liquidHeight > 0 && (
        <Shape
          sceneFunc={(ctx, shape) => {
            const liquidTop = tankBottomY - liquidHeight
            const waveHeight = properties.agitatorRunning ? 3 : 
                             properties.inletValveOpen ? 2 : 0
            
            ctx.beginPath()
            ctx.moveTo(6, tankBottomY - 1)
            ctx.lineTo(6, liquidTop)
            
            // Wavy surface if agitated
            if (waveHeight > 0) {
              for (let i = 0; i <= width - 12; i += 5) {
                const y = liquidTop + Math.sin(liquidAnimationRef.current + i * 0.2) * waveHeight
                ctx.lineTo(6 + i, y)
              }
            } else {
              ctx.lineTo(width - 6, liquidTop)
            }
            
            ctx.lineTo(width - 6, tankBottomY - 1)
            ctx.quadraticCurveTo(width - 6, tankBottomY - 1, width - 16, tankBottomY - 1)
            ctx.lineTo(16, tankBottomY - 1)
            ctx.quadraticCurveTo(6, tankBottomY - 1, 6, tankBottomY - 1)
            ctx.closePath()
            ctx.fillStrokeShape(shape)
          }}
          fill={getLiquidColor()}
          opacity={0.8}
        />
      )}

      {/* Level scale (0-100%) */}
      <Group x={width - 25} y={20}>
        {[0, 25, 50, 75, 100].map((mark) => {
          const markY = (tankBottomY - 20) * (1 - mark / 100)
          return (
            <Group key={mark}>
              <Line
                points={[20, markY, 25, markY]}
                stroke={ISA101Colors.equipmentOutline}
                strokeWidth={1}
              />
              <Text
                x={28}
                y={markY - 5}
                text={`${mark}`}
                fontSize={8}
                fill={ISA101Colors.processValue}
              />
            </Group>
          )
        })}
      </Group>

      {/* Alarm limit indicators */}
      {properties.showAlarmLimits && (
        <>
          {properties.criticalHigh && (
            <Line
              points={[10, 20 + (tankBottomY - 20) * (1 - properties.criticalHigh / 100), 
                      width - 35, 20 + (tankBottomY - 20) * (1 - properties.criticalHigh / 100)]}
              stroke={ISA101Colors.alarmCritical}
              strokeWidth={2}
              dash={[5, 3]}
            />
          )}
          {properties.alarmHigh && (
            <Line
              points={[10, 20 + (tankBottomY - 20) * (1 - properties.alarmHigh / 100), 
                      width - 35, 20 + (tankBottomY - 20) * (1 - properties.alarmHigh / 100)]}
              stroke={ISA101Colors.alarmHigh}
              strokeWidth={1}
              dash={[5, 3]}
            />
          )}
          {properties.alarmLow && (
            <Line
              points={[10, 20 + (tankBottomY - 20) * (1 - properties.alarmLow / 100), 
                      width - 35, 20 + (tankBottomY - 20) * (1 - properties.alarmLow / 100)]}
              stroke={ISA101Colors.alarmHigh}
              strokeWidth={1}
              dash={[5, 3]}
            />
          )}
          {properties.criticalLow && (
            <Line
              points={[10, 20 + (tankBottomY - 20) * (1 - properties.criticalLow / 100), 
                      width - 35, 20 + (tankBottomY - 20) * (1 - properties.criticalLow / 100)]}
              stroke={ISA101Colors.alarmCritical}
              strokeWidth={2}
              dash={[5, 3]}
            />
          )}
        </>
      )}

      {/* Tag name (top) */}
      <Text
        x={0}
        y={-15}
        width={width}
        text={properties.tagName}
        fontSize={12}
        fontStyle="bold"
        fill={ISA101Colors.processValue}
        align="center"
      />

      {/* Current value display (primary information) */}
      <Rect
        x={width / 2 - 40}
        y={height - 25}
        width={80}
        height={20}
        fill={ISA101Colors.containerBackground}
        stroke={ISA101Colors.equipmentOutline}
        strokeWidth={1}
      />
      <Text
        x={width / 2 - 40}
        y={height - 22}
        width={80}
        text={`${properties.currentLevel.toFixed(1)} ${properties.levelUnits}`}
        fontSize={14}
        fontStyle="bold"
        fill={ISA101Colors.processValue}
        align="center"
      />

      {/* Trend sparkline (if enabled) */}
      {properties.showTrend && (
        <Group x={10} y={height - 45}>
          <Rect
            x={0}
            y={0}
            width={60}
            height={20}
            fill={ISA101Colors.containerBackground}
            stroke={ISA101Colors.equipmentOutline}
            strokeWidth={1}
          />
          <Text
            x={2}
            y={2}
            text="TREND"
            fontSize={8}
            fill={ISA101Colors.processValue}
          />
          {/* Placeholder for actual trend - would connect to historical data */}
          <Line
            points={[5, 15, 15, 12, 25, 14, 35, 10, 45, 13, 55, 11]}
            stroke={ISA101Colors.setpoint}
            strokeWidth={1}
          />
        </Group>
      )}

      {/* Inlet/Outlet indicators */}
      {properties.inletValveOpen && (
        <Shape
          x={width / 2 - 10}
          y={5}
          sceneFunc={(ctx, shape) => {
            ctx.beginPath()
            ctx.moveTo(10, 0)
            ctx.lineTo(5, 8)
            ctx.lineTo(15, 8)
            ctx.closePath()
            ctx.fillStrokeShape(shape)
          }}
          fill={ISA101Colors.running}
        />
      )}
      
      {properties.outletValveOpen && (
        <Shape
          x={width / 2 - 10}
          y={tankBottomY + 5}
          sceneFunc={(ctx, shape) => {
            ctx.beginPath()
            ctx.moveTo(10, 8)
            ctx.lineTo(5, 0)
            ctx.lineTo(15, 0)
            ctx.closePath()
            ctx.fillStrokeShape(shape)
          }}
          fill={ISA101Colors.running}
        />
      )}
    </Group>
  )
}
