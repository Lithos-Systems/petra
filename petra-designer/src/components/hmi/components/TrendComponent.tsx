import { Group, Rect, Line, Text } from 'react-konva'
import type { TrendProperties } from '@/types/hmi'

interface TrendComponentProps {
  x: number
  y: number
  width: number
  height: number
  properties: TrendProperties
  style: any
  [key: string]: any
}

export default function TrendComponent({
  x, y, width, height, properties, style, ...rest
}: TrendComponentProps) {
  return (
    <Group x={x} y={y} {...rest}>
      {/* Background */}
      <Rect
        x={0}
        y={0}
        width={width}
        height={height}
        fill={style.fill || '#1a1a1a'}
        stroke={style.stroke || '#333333'}
        strokeWidth={style.strokeWidth || 1}
      />
      
      {/* Grid */}
      {properties.showGrid && (
        <>
          {/* Horizontal grid lines */}
          {[0.25, 0.5, 0.75].map((ratio) => (
            <Line
              key={`h-${ratio}`}
              points={[0, height * ratio, width, height * ratio]}
              stroke="#333333"
              strokeWidth={1}
              dash={[5, 5]}
            />
          ))}
          
          {/* Vertical grid lines */}
          {[0.25, 0.5, 0.75].map((ratio) => (
            <Line
              key={`v-${ratio}`}
              points={[width * ratio, 0, width * ratio, height]}
              stroke="#333333"
              strokeWidth={1}
              dash={[5, 5]}
            />
          ))}
        </>
      )}
      
      {/* Placeholder trend line */}
      <Line
        points={[
          10, height * 0.8,
          width * 0.3, height * 0.6,
          width * 0.5, height * 0.7,
          width * 0.7, height * 0.4,
          width - 10, height * 0.5
        ]}
        stroke="#00ff00"
        strokeWidth={2}
        tension={0.3}
      />
      
      {/* Y-axis labels */}
      <Text
        x={5}
        y={5}
        text={String(properties.yMax)}
        fontSize={10}
        fill="#ffffff"
      />
      <Text
        x={5}
        y={height - 15}
        text={String(properties.yMin)}
        fontSize={10}
        fill="#ffffff"
      />
      
      {/* Time range label */}
      <Text
        x={width - 50}
        y={height - 15}
        text={properties.timeRange}
        fontSize={10}
        fill="#ffffff"
      />
    </Group>
  )
}
