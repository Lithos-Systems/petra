import { useEffect, useRef } from 'react'
import { Group, Circle, Line, Text, Wedge } from 'react-konva'
import type { PumpProperties } from '@/types/hmi'

interface PumpComponentProps {
  id: string
  x: number
  y: number
  width: number
  height: number
  properties: PumpProperties
  bindings: Array<{
    property: string
    signal: string
    transform?: string
  }>
  style: any
  isSelected?: boolean
  draggable?: boolean
  onDragEnd?: (e: any) => void
  onClick?: () => void
}

export default function PumpComponent({
  id,
  x,
  y,
  width,
  height,
  properties,
  bindings,
  style,
  isSelected,
  draggable = true,
  onDragEnd,
  onClick,
}: PumpComponentProps) {
  const rotationRef = useRef(0)
  const animationRef = useRef<any>()

  useEffect(() => {
    if (properties.running && properties.runAnimation !== false) {
      const animate = () => {
        rotationRef.current = (rotationRef.current + 5) % 360
        animationRef.current = requestAnimationFrame(animate)
      }
      animationRef.current = requestAnimationFrame(animate)
    }

    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current)
      }
    }
  }, [properties.running, properties.runAnimation])

  const centerX = width / 2
  const centerY = height / 2
  const radius = Math.min(width, height) / 2 - 5

  // Determine pump color based on status
  const getPumpColor = () => {
    if (properties.fault) return '#ff4444'
    if (properties.running) return '#10b981'
    return '#6b7280'
  }

  return (
    <Group
      x={x}
      y={y}
      draggable={draggable}
      onDragEnd={onDragEnd}
      onClick={onClick}
      onTap={onClick}
    >
      {/* Selection indicator */}
      {isSelected && (
        <Circle
          x={centerX}
          y={centerY}
          radius={radius + 8}
          stroke="#3b82f6"
          strokeWidth={2}
          dash={[5, 5]}
          fill="transparent"
        />
      )}

      {/* Pump housing */}
      <Circle
        x={centerX}
        y={centerY}
        radius={radius}
        fill={style.fill || '#e5e7eb'}
        stroke={style.stroke || '#333333'}
        strokeWidth={style.strokeWidth || 2}
      />

      {/* Impeller blades */}
      <Group x={centerX} y={centerY} rotation={rotationRef.current}>
        {[0, 120, 240].map((angle) => (
          <Wedge
            key={angle}
            x={0}
            y={0}
            radius={radius * 0.7}
            angle={60}
            rotation={angle}
            fill={getPumpColor()}
            stroke={style.stroke || '#333333'}
            strokeWidth={1}
          />
        ))}
      </Group>

      {/* Center hub */}
      <Circle
        x={centerX}
        y={centerY}
        radius={radius * 0.2}
        fill={style.stroke || '#333333'}
      />

      {/* Inlet pipe */}
      <Line
        points={[0, centerY, -20, centerY]}
        stroke="#666666"
        strokeWidth={6}
      />

      {/* Outlet pipe */}
      <Line
        points={[centerX, 0, centerX, -20]}
        stroke="#666666"
        strokeWidth={6}
      />

      {/* Status indicator */}
      {properties.showStatus && (
        <Group x={centerX - 15} y={height + 5}>
          <Circle
            x={7}
            y={7}
            radius={5}
            fill={properties.running ? '#10b981' : '#6b7280'}
            shadowBlur={properties.running ? 5 : 0}
            shadowColor="#10b981"
          />
          <Text
            x={20}
            y={0}
            text={properties.running ? 'RUN' : 'STOP'}
            fontSize={12}
            fill={properties.running ? '#10b981' : '#6b7280'}
            fontStyle="bold"
          />
        </Group>
      )}

      {/* Fault indicator */}
      {properties.fault && (
        <Group x={width - 30} y={5}>
          <Circle
            x={0}
            y={0}
            radius={10}
            fill="#ff0000"
          />
          <Text
            x={-5}
            y={-5}
            text="!"
            fontSize={14}
            fill="#ffffff"
            fontStyle="bold"
          />
        </Group>
      )}

      {/* Speed indicator */}
      {properties.running && properties.speed > 0 && (
        <Text
          x={0}
          y={height + 20}
          width={width}
          text={`${properties.speed}%`}
          fontSize={11}
          fill="#666666"
          align="center"
        />
      )}
    </Group>
  )
}
