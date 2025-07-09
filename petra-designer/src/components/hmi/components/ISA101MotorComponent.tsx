// @ts-nocheck
import { Group, Circle, Text } from 'react-konva'

export default function ISA101MotorComponent({ x, y, width, height, properties, ...props }: any) {
  const radius = Math.min(width, height) / 2
  return (
    <Group x={x} y={y} {...props}>
      <Circle
        x={radius}
        y={radius}
        radius={radius}
        fill={properties.running ? '#10b981' : '#6b7280'}
        stroke="#000"
        strokeWidth={2}
      />
      <Text
        x={0}
        y={0}
        width={width}
        height={height}
        text="M"
        fontSize={radius}
        fontStyle="bold"
        fill="#fff"
        align="center"
        verticalAlign="middle"
      />
    </Group>
  )
}
