import { Group, Circle, Line, Text, Arc } from 'react-konva'
import type { GaugeProperties } from '@/types/hmi'

interface GaugeComponentProps {
  x: number
  y: number
  width: number
  height: number
  properties: GaugeProperties
  style: any
  [key: string]: any
}

export default function GaugeComponent({
  x, y, width, height, properties, style, ...rest
}: GaugeComponentProps) {
  const radius = Math.min(width, height) / 2 - 10
  const centerX = width / 2
  const centerY = height / 2
  const { min, max, value } = properties
  
  // Calculate angle for value
  const angleRange = 240 // degrees
  const startAngle = -210
  const valueRatio = (value - min) / (max - min)
  const valueAngle = startAngle + (angleRange * valueRatio)
  
  return (
    <Group x={x} y={y} {...rest}>
      {/* Background circle */}
      <Circle
        x={centerX}
        y={centerY}
        radius={radius}
        fill={style.fill || '#ffffff'}
        stroke={style.stroke || '#333333'}
        strokeWidth={style.strokeWidth || 2}
      />
      
      {/* Scale marks */}
      {properties.showScale && Array.from({ length: properties.majorTicks + 1 }).map((_, i) => {
        const angle = startAngle + (angleRange * i / properties.majorTicks)
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
            strokeWidth={2}
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
        stroke="#ff0000"
        strokeWidth={3}
        lineCap="round"
      />
      
      {/* Center dot */}
      <Circle
        x={centerX}
        y={centerY}
        radius={5}
        fill="#333333"
      />
      
      {/* Value display */}
      <Text
        x={0}
        y={centerY + radius / 2}
        width={width}
        text={`${value.toFixed(1)} ${properties.units}`}
        fontSize={14}
        fill="#333333"
        align="center"
        fontStyle="bold"
      />
    </Group>
  )
}
