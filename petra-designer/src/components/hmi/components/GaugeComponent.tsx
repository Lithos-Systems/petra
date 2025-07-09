// src/components/hmi/components/GaugeComponent.tsx

import { Group, Circle, Line, Text } from 'react-konva'
import type { GaugeProperties } from '@/types/hmi'

interface GaugeComponentProps {
  x: number
  y: number
  width: number
  height: number
  properties: GaugeProperties
  style?: any
  [key: string]: any
}

export default function GaugeComponent({
  x, 
  y, 
  width, 
  height, 
  properties = {
    min: 0,
    max: 100,
    value: 0,
    units: '',
    showScale: true,
    majorTicks: 5
  }, 
  style = {}, 
  ...rest
}: GaugeComponentProps) {
  const radius = Math.min(width, height) / 2 - 10
  const centerX = width / 2
  const centerY = height / 2
  const { min = 0, max = 100, value = 0, units = '', showScale = true, majorTicks = 5 } = properties
  
  // Calculate angle for value
  const angleRange = 240 // degrees
  const startAngle = -210
  const valueRatio = Math.max(0, Math.min(1, (value - min) / (max - min)))
  const valueAngle = startAngle + (angleRange * valueRatio)
  
  // Get colors
  const fillColor = style.fill || '#ffffff'
  const strokeColor = style.stroke || '#333333'
  const strokeWidth = style.strokeWidth || 2
  const needleColor = style.needleColor || '#ef4444'
  const textColor = style.textColor || '#333333'
  
  return (
    <Group x={x} y={y} {...rest}>
      {/* Background circle */}
      <Circle
        x={centerX}
        y={centerY}
        radius={radius}
        fill={fillColor}
        stroke={strokeColor}
        strokeWidth={strokeWidth}
      />
      
      {/* Scale marks */}
      {showScale && Array.from({ length: majorTicks + 1 }).map((_, i) => {
        const angle = startAngle + (angleRange * i / majorTicks)
        const radian = (angle * Math.PI) / 180
        const x1 = centerX + Math.cos(radian) * (radius - 10)
        const y1 = centerY + Math.sin(radian) * (radius - 10)
        const x2 = centerX + Math.cos(radian) * radius
        const y2 = centerY + Math.sin(radian) * radius
        
        return (
          <Line
            key={i}
            points={[x1, y1, x2, y2]}
            stroke="#666666"
            strokeWidth={1}
          />
        )
      })}
      
      {/* Needle */}
      <Line
        points={[
          centerX, centerY,
          centerX + Math.cos((valueAngle * Math.PI) / 180) * (radius - 15),
          centerY + Math.sin((valueAngle * Math.PI) / 180) * (radius - 15)
        ]}
        stroke={needleColor}
        strokeWidth={3}
        lineCap="round"
      />
      
      {/* Center dot */}
      <Circle
        x={centerX}
        y={centerY}
        radius={5}
        fill={strokeColor}
      />
      
      {/* Label */}
      {properties.label && (
        <Text
          x={0}
          y={centerY - radius - 15}
          width={width}
          text={properties.label}
          fontSize={12}
          fill={textColor}
          align="center"
        />
      )}
      
      {/* Value display */}
      {properties.showDigital !== false && (
        <Text
          x={0}
          y={centerY + radius / 2}
          width={width}
          text={`${value.toFixed(1)} ${units}`}
          fontSize={14}
          fill={textColor}
          align="center"
          fontStyle="bold"
        />
      )}
    </Group>
  )
}
