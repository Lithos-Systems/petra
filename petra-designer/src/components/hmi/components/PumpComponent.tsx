// src/components/hmi/components/PumpComponent.tsx

import { useEffect, useRef } from 'react'
import { Group, Circle, Line, Text, Path, Rect } from 'react-konva'
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

  // IEC 60617 compliant pump symbol path
  // This creates a circle with internal triangle (simplified centrifugal pump symbol)
  const pumpPath = `
    M ${centerX - 25 * scale} ${centerY}
    A ${25 * scale} ${25 * scale} 0 1 1 ${centerX + 25 * scale} ${centerY}
    A ${25 * scale} ${25 * scale} 0 1 1 ${centerX - 25 * scale} ${centerY}
    M ${centerX - 15 * scale} ${centerY + 10 * scale}
    L ${centerX} ${centerY - 15 * scale}
    L ${centerX + 15 * scale} ${centerY + 10 * scale}
    Z
  `

  // Motor representation (small rectangle on top)
  const motorWidth = 30 * scale
  const motorHeight = 15 * scale
  const motorX = centerX - motorWidth / 2
  const motorY = centerY - 25 * scale - motorHeight - 5 * scale

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
          x={-5}
          y={motorY - 5}
          width={width + 10}
          height={height + 10}
          stroke="#3b82f6"
          strokeWidth={2}
          dash={[5, 5]}
          fill="transparent"
        />
      )}

      {/* Motor */}
      <Rect
        x={motorX}
        y={motorY}
        width={motorWidth}
        height={motorHeight}
        fill={fillColor}
        stroke={strokeColor}
        strokeWidth={2 * scale}
      />

      {/* Motor status indicator */}
      <Rect
        x={motorX + 2}
        y={motorY + 2}
        width={motorWidth - 4}
        height={motorHeight - 4}
        fill={properties.running ? statusColor : 'transparent'}
        opacity={0.3}
      />

      {/* Pump body */}
      <Path
        data={pumpPath}
        fill={fillColor}
        stroke={strokeColor}
        strokeWidth={2 * scale}
      />

      {/* Running indicator (fill the triangle) */}
      {properties.running && (
        <Path
          data={`
            M ${centerX - 15 * scale} ${centerY + 10 * scale}
            L ${centerX} ${centerY - 15 * scale}
            L ${centerX + 15 * scale} ${centerY + 10 * scale}
            Z
          `}
          fill={statusColor}
          opacity={0.5}
        />
      )}

      {/* Inlet pipe */}
      <Line
        points={[
          centerX - 25 * scale - 15 * scale, centerY,
          centerX - 25 * scale, centerY
        ]}
        stroke={strokeColor}
        strokeWidth={6 * scale}
        lineCap="round"
      />

      {/* Outlet pipe */}
      <Line
        points={[
          centerX, centerY - 25 * scale,
          centerX, centerY - 25 * scale - 15 * scale
        ]}
        stroke={strokeColor}
        strokeWidth={6 * scale}
        lineCap="round"
      />

      {/* Flow direction arrow on outlet */}
      <Path
        data={`
          M ${centerX - 4 * scale} ${centerY - 35 * scale}
          L ${centerX} ${centerY - 42 * scale}
          L ${centerX + 4 * scale} ${centerY - 35 * scale}
        `}
        fill={strokeColor}
      />

      {/* Status indicator dot */}
      {properties.showStatus && (
        <Circle
          x={centerX + 25 * scale + 10 * scale}
          y={centerY}
          radius={5 * scale}
          fill={statusColor}
          stroke={strokeColor}
          strokeWidth={1}
        />
      )}

      {/* Speed indicator (if variable speed) */}
      {properties.speed !== undefined && properties.speed !== 100 && (
        <Text
          x={centerX - 20 * scale}
          y={centerY + 35 * scale}
          width={40 * scale}
          text={`${Math.round(properties.speed)}%`}
          fontSize={10 * scale}
          align="center"
          fill={strokeColor}
        />
      )}

      {/* Fault indicator */}
      {properties.fault && (
        <Text
          x={centerX - 10 * scale}
          y={motorY - 15 * scale}
          text="!"
          fontSize={16 * scale}
          fontStyle="bold"
          fill="#ef4444"
          align="center"
        />
      )}
    </Group>
  )
}
