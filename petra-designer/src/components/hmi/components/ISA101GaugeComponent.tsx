// @ts-nocheck
import { Group, Circle, Line, Text, Shape, Rect } from 'react-konva'

const ISA101Colors = {
  equipmentOutline: '#000000',
  equipmentFill: '#FFFFFF',
  processValue: '#000000',
  setpoint: '#0000FF',
  alarmHigh: '#FF8C00',
  alarmLow: '#FF8C00',
  scale: '#666666',
}

interface ISA101GaugeProps {
  x: number
  y: number
  width: number
  height: number
  properties: {
    tagName: string
    currentValue: number
    units: string
    minValue: number
    maxValue: number
    alarmHigh?: number
    alarmLow?: number
    setpoint?: number
    showDigitalValue?: boolean
    gaugeType?: 'pressure' | 'temperature' | 'flow' | 'level'
  }
  selected?: boolean
  onClick?: () => void
  draggable?: boolean
  onDragEnd?: (e: any) => void
}

export default function ISA101GaugeComponent({
  x,
  y,
  width,
  height,
  properties,
  selected,
  onClick,
  draggable,
  onDragEnd,
}: ISA101GaugeProps) {
  const radius = Math.min(width, height) / 2 - 10
  const centerX = width / 2
  const centerY = height / 2

  // Calculate angle for value
  const valueRange = properties.maxValue - properties.minValue
  const normalizedValue = (properties.currentValue - properties.minValue) / valueRange
  const startAngle = -135
  const endAngle = 135
  const valueAngle = startAngle + (endAngle - startAngle) * normalizedValue

  const getXY = (angle: number, r: number) => {
    const rad = (angle * Math.PI) / 180
    return {
      x: centerX + r * Math.cos(rad),
      y: centerY + r * Math.sin(rad),
    }
  }

  return (
    <Group
      x={x}
      y={y}
      draggable={draggable}
      onDragEnd={onDragEnd}
      onClick={onClick}
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

      {/* Gauge background */}
      <Circle
        x={centerX}
        y={centerY}
        radius={radius}
        fill={ISA101Colors.equipmentFill}
        stroke={ISA101Colors.equipmentOutline}
        strokeWidth={2}
      />

      {/* Scale arc */}
      <Shape
        sceneFunc={(context, shape) => {
          context.beginPath()
          context.arc(
            centerX,
            centerY,
            radius - 10,
            (startAngle * Math.PI) / 180,
            (endAngle * Math.PI) / 180
          )
          context.strokeStyle = ISA101Colors.scale
          context.lineWidth = 2
          context.stroke()
        }}
      />

      {/* Scale ticks and labels */}
      {[0, 0.25, 0.5, 0.75, 1].map((fraction, i) => {
        const angle = startAngle + (endAngle - startAngle) * fraction
        const inner = getXY(angle, radius - 15)
        const outer = getXY(angle, radius - 5)
        const label = getXY(angle, radius - 30)
        const value = properties.minValue + valueRange * fraction

        return (
          <Group key={i}>
            <Line
              points={[inner.x, inner.y, outer.x, outer.y]}
              stroke={ISA101Colors.scale}
              strokeWidth={2}
            />
            <Text
              x={label.x - 15}
              y={label.y - 6}
              width={30}
              text={value.toFixed(0)}
              fontSize={10}
              fontFamily="Arial"
              fill={ISA101Colors.processValue}
              align="center"
            />
          </Group>
        )
      })}

      {/* Alarm limits */}
      {properties.alarmHigh !== undefined && (
        (() => {
          const angle =
            startAngle +
            ((properties.alarmHigh - properties.minValue) / valueRange) *
              (endAngle - startAngle)
          const inner = getXY(angle, radius - 15)
          const outer = getXY(angle, radius)
          return (
            <Line
              points={[inner.x, inner.y, outer.x, outer.y]}
              stroke={ISA101Colors.alarmHigh}
              strokeWidth={3}
            />
          )
        })()
      )}

      {/* Value pointer */}
      <Line
        points={[centerX, centerY, getXY(valueAngle, radius - 20).x, getXY(valueAngle, radius - 20).y]}
        stroke={ISA101Colors.processValue}
        strokeWidth={3}
        lineCap="round"
      />

      {/* Center dot */}
      <Circle
        x={centerX}
        y={centerY}
        radius={5}
        fill={ISA101Colors.equipmentOutline}
      />

      {/* Tag name */}
      <Text
        x={0}
        y={height - 25}
        width={width}
        text={properties.tagName}
        fontSize={12}
        fontFamily="Arial"
        fontStyle="bold"
        fill={ISA101Colors.processValue}
        align="center"
      />

      {/* Digital value display */}
      {properties.showDigitalValue && (
        <Group y={centerY + 20}>
          <Rect
            x={centerX - 40}
            y={0}
            width={80}
            height={25}
            fill={ISA101Colors.equipmentFill}
            stroke={ISA101Colors.equipmentOutline}
            strokeWidth={1}
          />
          <Text
            x={centerX - 40}
            y={5}
            width={80}
            text={`${properties.currentValue.toFixed(1)} ${properties.units}`}
            fontSize={12}
            fontFamily="Courier New"
            fontStyle="bold"
            fill={ISA101Colors.processValue}
            align="center"
          />
        </Group>
      )}
    </Group>
  )
}
