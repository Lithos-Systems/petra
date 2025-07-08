// src/components/hmi/components/PumpComponent.tsx

import { useRef } from 'react'
import { Group, Circle, Line, Text, Path, Rect } from 'react-konva'
import type { PumpProperties } from '@/types/hmi'

interface PumpComponentProps {
  x: number
  y: number
  width: number
  height: number
  properties: PumpProperties
  style: any
  isSelected?: boolean
  draggable?: boolean
  onDragEnd?: (e: any) => void
  onClick?: () => void
}

export default function PumpComponent({
  x,
  y,
  width,
  height,
  properties,
  style,
  isSelected,
  draggable = true,
  onDragEnd,
  onClick,
}: PumpComponentProps) {
  const centerX = width / 2
  const centerY = height / 2
  const scale = Math.min(width, height) / 80 // Base size is 80x80

  // Determine pump color based on status
  const getPumpColor = () => {
    if (properties.fault) return '#ef4444' // red-500
    if (properties.running) return '#10b981' // emerald-500
    return '#9ca3af' // gray-400
  }

  const fillColor = style.fill || '#ffffff'
  const strokeColor = style.stroke || '#374151'
  const statusColor = getPumpColor()

  // IEC 60617 compliant pump symbol
  const radius = 25 * scale

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
        <Rect
          x={centerX - radius - 10}
          y={centerY - radius - 10}
          width={radius * 2 + 20}
          height={radius * 2 + 20}
          stroke="#3b82f6"
          strokeWidth={2}
          dash={[5, 5]}
          fill="transparent"
        />
      )}

      {/* Pump circle */}
      <Circle
        x={centerX}
        y={centerY}
        radius={radius}
        fill={fillColor}
        stroke={strokeColor}
        strokeWidth={2 * scale}
      />

      {/* Interior triangle (IEC pump symbol) */}
      <Path
        data={`
          M ${centerX} ${centerY - radius * 0.7}
          L ${centerX - radius * 0.6} ${centerY + radius * 0.35}
          L ${centerX + radius * 0.6} ${centerY + radius * 0.35}
          Z
        `}
        fill={properties.running ? statusColor : strokeColor}
        opacity={properties.running ? 0.8 : 0.3}
      />

      {/* Inlet pipe (left side) */}
      <Line
        points={[
          centerX - radius - 20 * scale, centerY,
          centerX - radius, centerY
        ]}
        stroke={strokeColor}
        strokeWidth={4 * scale}
        lineCap="round"
      />

      {/* Outlet pipe (top) */}
      <Line
        points={[
          centerX, centerY - radius,
          centerX, centerY - radius - 20 * scale
        ]}
        stroke={strokeColor}
        strokeWidth={4 * scale}
        lineCap="round"
      />

      {/* Flow direction arrow on outlet */}
      <Path
        data={`
          M ${centerX - 3 * scale} ${centerY - radius - 15 * scale}
          L ${centerX} ${centerY - radius - 20 * scale}
          L ${centerX + 3 * scale} ${centerY - radius - 15 * scale}
        `}
        fill={strokeColor}
      />

      {/* Status indicator dot */}
      {properties.showStatus && (
        <Circle
          x={centerX + radius + 8 * scale}
          y={centerY - radius + 5 * scale}
          radius={4 * scale}
          fill={statusColor}
          stroke={strokeColor}
          strokeWidth={1}
        />
      )}

      {/* Speed indicator (if variable speed) */}
      {properties.speed !== undefined && properties.speed !== 100 && properties.running && (
        <Text
          x={centerX - 20 * scale}
          y={centerY + radius + 5 * scale}
          width={40 * scale}
          text={`${Math.round(properties.speed)}%`}
          fontSize={10 * scale}
          align="center"
          fill={strokeColor}
        />
      )}

      {/* Fault indicator */}
      {properties.fault && (
        <Group>
          <Circle
            x={centerX}
            y={centerY - radius - 35 * scale}
            radius={8 * scale}
            fill="#ef4444"
          />
          <Text
            x={centerX - 4 * scale}
            y={centerY - radius - 39 * scale}
            text="!"
            fontSize={12 * scale}
            fontStyle="bold"
            fill="#ffffff"
          />
        </Group>
      )}
    </Group>
  )
}
