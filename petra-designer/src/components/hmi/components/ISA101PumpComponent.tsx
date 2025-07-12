import { useEffect, useRef, useState } from 'react'
import { Group, Circle, Line, Text, Rect, Shape, Path } from 'react-konva'

// ISA-101 Standard Colors
const ISA101Colors = {
  equipmentOutline: '#000000',
  equipmentFill: '#E6E6E6',
  running: '#00FF00',
  stopped: '#808080',
  fault: '#FF0000',
  interlocked: '#FF8C00',
  manual: '#9370DB',
  processValue: '#000000',
  background: '#FFFFFF',
}

interface ISA101PumpProps {
  x: number
  y: number
  width: number
  height: number
  properties: {
    tagName?: string
    status?: 'running' | 'stopped' | 'fault' | 'transitioning'
    flowRate?: number
    flowUnits?: string
    dischargePressure?: number
    pressureUnits?: string
    speed?: number
    controlMode?: 'auto' | 'manual' | 'cascade'
    interlocked?: boolean
    showDetailedStatus?: boolean
    showFlowDirection?: boolean
    pumpType?: 'centrifugal' | 'positive-displacement'
    [key: string]: any
  }
  style?: {
    lineWidth?: number
    [key: string]: any
  }
  selected?: boolean
  onClick?: () => void
  onContextMenu?: (e: any) => void
  draggable?: boolean
  onDragEnd?: (e: any) => void
  onDragStart?: (e: any) => void
}

export default function ISA101PumpComponent({
  x,
  y,
  width,
  height,
  properties,
  style = {},
  selected = false,
  onClick,
  onContextMenu,
  draggable = true,
  onDragEnd,
  onDragStart,
  ...restProps
}: ISA101PumpProps) {
  const rotationRef = useRef(0)
  const animationRef = useRef<any>()
  const groupRef = useRef<any>()
  const [isAnimating, setIsAnimating] = useState(false)

  // Determine pump color based on status
  const getPumpColor = () => {
    if (properties.interlocked) return ISA101Colors.interlocked
    switch (properties.status) {
      case 'running': return ISA101Colors.running
      case 'fault': return ISA101Colors.fault
      case 'transitioning': return '#FFFF00'
      default: return ISA101Colors.stopped
    }
  }

  // Animate pump when running
  useEffect(() => {
    if (properties.status === 'running') {
      setIsAnimating(true)
      const animate = () => {
        rotationRef.current = (rotationRef.current + (properties.speed || 100) / 20) % 360
        if (groupRef.current) {
          groupRef.current.getLayer()?.batchDraw()
        }
        animationRef.current = requestAnimationFrame(animate)
      }
      animationRef.current = requestAnimationFrame(animate)
    } else {
      setIsAnimating(false)
    }

    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current)
      }
    }
  }, [properties.status, properties.speed])

  const centerX = width / 2
  const centerY = height / 2
  const radius = Math.min(width, height) * 0.35

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

      {/* Pump casing (circle) */}
      <Circle
        x={centerX}
        y={centerY}
        radius={radius}
        fill={ISA101Colors.equipmentFill}
        stroke={properties.status === 'fault' ? ISA101Colors.fault : ISA101Colors.equipmentOutline}
        strokeWidth={properties.status === 'fault' ? 3 : (style.lineWidth || 2)}
      />

      {/* Pump impeller (simplified for ISA-101) */}
      <Group x={centerX} y={centerY} rotation={rotationRef.current}>
        <Shape
          sceneFunc={(ctx, shape) => {
            // Draw simple impeller vanes
            ctx.beginPath()
            for (let i = 0; i < 4; i++) {
              const angle = (i * Math.PI) / 2
              ctx.moveTo(0, 0)
              ctx.lineTo(
                Math.cos(angle) * radius * 0.7,
                Math.sin(angle) * radius * 0.7
              )
            }
            ctx.fillStrokeShape(shape)
          }}
          stroke={ISA101Colors.equipmentOutline}
          strokeWidth={2}
        />
        
        {/* Center hub */}
        <Circle
          x={0}
          y={0}
          radius={radius * 0.2}
          fill={getPumpColor()}
          stroke={ISA101Colors.equipmentOutline}
          strokeWidth={1}
        />
      </Group>

      {/* Suction and discharge flanges */}
      <Line
        points={[0, centerY, centerX - radius, centerY]}
        stroke={ISA101Colors.equipmentOutline}
        strokeWidth={4}
      />
      <Line
        points={[centerX, centerY - radius, centerX, 0]}
        stroke={ISA101Colors.equipmentOutline}
        strokeWidth={4}
      />

      {/* Flow direction arrows (if enabled) */}
      {properties.showFlowDirection && properties.status === 'running' && (
        <>
          {/* Suction arrow */}
          <Shape
            x={centerX - radius - 15}
            y={centerY}
            sceneFunc={(ctx, shape) => {
              ctx.beginPath()
              ctx.moveTo(-10, -5)
              ctx.lineTo(0, 0)
              ctx.lineTo(-10, 5)
              ctx.fillStrokeShape(shape)
            }}
            fill={ISA101Colors.running}
          />
          
          {/* Discharge arrow */}
          <Shape
            x={centerX}
            y={centerY - radius - 15}
            sceneFunc={(ctx, shape) => {
              ctx.beginPath()
              ctx.moveTo(-5, 10)
              ctx.lineTo(0, 0)
              ctx.lineTo(5, 10)
              ctx.fillStrokeShape(shape)
            }}
            fill={ISA101Colors.running}
          />
        </>
      )}

      {/* Tag name */}
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

      {/* Status indicator */}
      <Group x={centerX + radius + 10} y={centerY - 10}>
        <Circle
          x={0}
          y={0}
          radius={8}
          fill={getPumpColor()}
          stroke={ISA101Colors.equipmentOutline}
          strokeWidth={1}
        />
        {properties.controlMode === 'manual' && (
          <Text
            x={-5}
            y={-5}
            text="M"
            fontSize={10}
            fontStyle="bold"
            fill="#FFFFFF"
          />
        )}
      </Group>

      {/* Flow rate display (primary information) */}
      {properties.flowRate !== undefined && (
        <Group x={0} y={height + 5}>
          <Rect
            x={width / 2 - 40}
            y={0}
            width={80}
            height={20}
            fill={ISA101Colors.background}
            stroke={ISA101Colors.equipmentOutline}
            strokeWidth={1}
          />
          <Text
            x={width / 2 - 40}
            y={3}
            width={80}
            text={`${properties.flowRate.toFixed(1)} ${properties.flowUnits || 'GPM'}`}
            fontSize={12}
            fontStyle="bold"
            fill={ISA101Colors.processValue}
            align="center"
          />
        </Group>
      )}

      {/* Discharge pressure (secondary information) */}
      {properties.showDetailedStatus && properties.dischargePressure !== undefined && (
        <Group x={0} y={height + 28}>
          <Rect
            x={width / 2 - 30}
            y={0}
            width={60}
            height={16}
            fill={ISA101Colors.background}
            stroke={ISA101Colors.equipmentOutline}
            strokeWidth={1}
          />
          <Text
            x={width / 2 - 30}
            y={2}
            width={60}
            text={`${properties.dischargePressure} ${properties.pressureUnits || 'PSI'}`}
            fontSize={10}
            fill={ISA101Colors.processValue}
            align="center"
          />
        </Group>
      )}

      {/* Speed indicator (if variable speed) */}
      {properties.speed !== undefined && properties.speed !== 100 && (
        <Text
          x={0}
          y={height + 48}
          width={width}
          text={`${properties.speed}%`}
          fontSize={10}
          fill={ISA101Colors.processValue}
          align="center"
        />
      )}

      {/* Interlock indicator */}
      {properties.interlocked && (
        <Group x={centerX - 20} y={centerY + radius + 5}>
          <Rect
            x={0}
            y={0}
            width={40}
            height={16}
            fill={ISA101Colors.interlocked}
            stroke={ISA101Colors.equipmentOutline}
            strokeWidth={1}
          />
          <Text
            x={0}
            y={2}
            width={40}
            text="INTLK"
            fontSize={10}
            fontStyle="bold"
            fill="#FFFFFF"
            align="center"
          />
        </Group>
      )}

      {/* Fault indicator */}
      {properties.status === 'fault' && (
        <Group x={centerX - radius - 30} y={centerY - 10}>
          <Shape
            sceneFunc={(ctx, shape) => {
              // Draw warning triangle
              ctx.beginPath()
              ctx.moveTo(10, 0)
              ctx.lineTo(0, 17)
              ctx.lineTo(20, 17)
              ctx.closePath()
              ctx.fillStrokeShape(shape)
            }}
            fill={ISA101Colors.fault}
            stroke={ISA101Colors.equipmentOutline}
            strokeWidth={1}
          />
          <Text
            x={7}
            y={8}
            text="!"
            fontSize={12}
            fontStyle="bold"
            fill="#FFFFFF"
          />
        </Group>
      )}
    </Group>
  )
}
