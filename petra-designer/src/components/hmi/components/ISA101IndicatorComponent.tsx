// @ts-nocheck
import { Group, Circle } from 'react-konva'

export default function ISA101IndicatorComponent({ x, y, width, height, properties, ...props }: any) {
  const radius = Math.min(width, height) / 2
  return (
    <Group x={x} y={y} {...props}>
      <Circle
        x={radius}
        y={radius}
        radius={radius}
        fill={properties.on ? properties.onColor || '#00ff00' : properties.offColor || '#cccccc'}
        stroke="#000"
        strokeWidth={2}
      />
    </Group>
  )
}
