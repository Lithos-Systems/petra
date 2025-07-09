// @ts-nocheck
import { Group, Circle, Text, Rect } from 'react-konva'
import { useRef } from 'react'

const ISA101Colors = {
  equipmentOutline: '#000000',
  equipmentFill: '#E6E6E6',
  running: '#00FF00',
  stopped: '#808080',
  fault: '#FF0000',
  manual: '#9370DB',
  processValue: '#000000',
  background: '#FFFFFF',
}

interface ISA101MotorProps {
  x: number
  y: number
  width: number
  height: number
  properties: {
    tagName: string
    running: boolean
    speed?: number
    fault?: boolean
    controlMode?: 'auto' | 'manual'
    current?: number
    temperature?: number
  }
  style?: any
  selected?: boolean
  draggable?: boolean
  onDragEnd?: (e: any) => void
  onDragStart?: (e: any) => void
  onClick?: () => void
  onContextMenu?: (e: any) => void
}

export default function ISA101MotorComponent({
  x,
  y,
  width,
  height,
  properties,
  style = {},
  selected,
  draggable,
  onDragEnd,
  onDragStart,
  onClick,
  onContextMenu,
  ...restProps
}: ISA101MotorProps) {
  const radius = Math.min(width, height) / 2
  const groupRef = useRef<any>()

  const getMotorColor = () => {
    if (properties.fault) return ISA101Colors.fault
    if (properties.running) return ISA101Colors.running
    return ISA101Colors.stopped
  }

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

      {/* Motor circle */}
      <Circle
        x={radius}
        y={radius}
        radius={radius - 2}
        fill={getMotorColor()}
        stroke={ISA101Colors.equipmentOutline}
        strokeWidth={2}
      />

      {/* Motor symbol 'M' */}
      <Text
        x={0}
        y={0}
        width={width}
        height={height}
        text="M"
        fontSize={radius * 0.8}
        fontStyle="bold"
        fill="#FFFFFF"
        align="center"
        verticalAlign="middle"
      />

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

      {/* Speed indicator */}
      {properties.speed !== undefined && properties.speed !== 100 && (
        <Text
          x={0}
          y={height + 5}
          width={width}
          text={`${properties.speed}%`}
          fontSize={10}
          fill={ISA101Colors.processValue}
          align="center"
        />
      )}

      {/* Control mode indicator */}
      {properties.controlMode === 'manual' && (
        <Group x={width - 15} y={-5}>
          <Circle
            x={0}
            y={0}
            radius={6}
            fill={ISA101Colors.manual}
            stroke={ISA101Colors.equipmentOutline}
            strokeWidth={1}
          />
          <Text
            x={-3}
            y={-4}
            text="M"
            fontSize={8}
            fontStyle="bold"
            fill="#FFFFFF"
          />
        </Group>
      )}

      {/* Fault indicator */}
      {properties.fault && (
        <Group x={-10} y={-5}>
          <Circle
            x={0}
            y={0}
            radius={6}
            fill={ISA101Colors.fault}
            stroke={ISA101Colors.equipmentOutline}
            strokeWidth={1}
          />
          <Text
            x={-2}
            y={-4}
            text="!"
            fontSize={8}
            fontStyle="bold"
            fill="#FFFFFF"
          />
        </Group>
      )}
    </Group>
  )
}
