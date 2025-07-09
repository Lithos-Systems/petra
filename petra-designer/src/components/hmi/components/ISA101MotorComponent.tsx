// @ts-nocheck
import { Group, Circle, Text } from 'react-konva'
import { useRef } from 'react'
import ISA101Colors from '../constants/isa101Colors'

export default function ISA101MotorComponent({
  x,
  y,
  width,
  height,
  properties,
  selected,
  draggable,
  onDragEnd,
  ...props
}: any) {
  const radius = Math.min(width, height) / 2
  const shapeRef = useRef<any>()

  return (
    <Group
      ref={shapeRef}
      x={x}
      y={y}
      draggable={draggable}
      onDragEnd={onDragEnd}
      onDragStart={(e) => e.target.moveToTop()}
      {...props}
    >
      <Circle
        x={radius}
        y={radius}
        radius={radius}
        fill={properties.fault
          ? ISA101Colors.fault
          : properties.running
            ? ISA101Colors.running
            : ISA101Colors.stopped}
        stroke={ISA101Colors.equipmentOutline}
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
