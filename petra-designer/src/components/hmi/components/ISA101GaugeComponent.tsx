import { Group, Circle, Line, Text, Shape, Rect } from 'react-konva'
import { useRef } from 'react'

// ISA-101 Standard Colors
const ISA101Colors = {
  equipmentOutline: '#000000',
  equipmentFill: '#FFFFFF',
  processValue: '#000000',
  setpoint: '#0000FF',
  alarmHigh: '#FF8C00',
  alarmLow: '#FF8C00',
  scale: '#666666',
  needleNormal: '#000000',
  needleAlarm: '#FF0000',
  background: '#FFFFFF',
}

interface ISA101GaugeProps {
  x: number
  y: number
  width: number
  height: number
  properties: {
    tagName?: string
    currentValue?: number
    units?: string
    minValue?: number
    maxValue?: number
    alarmHigh?: number
    alarmLow?: number
    setpoint?: number
    showDigitalValue?: boolean
    gaugeType?: 'pressure' | 'temperature' | 'flow' | 'level'
    showTrend?: boolean
    [key: string]: any
  }
  style?: {
    lineWidth?: number
    [key: string]: any
  }
  selected?: boolean
  onClick?: () => void
  onContextMenu?: (e: any) => void
  draggable?: boolean
  onDragEnd?: (e: any) => void
  onDragStart?: (e: any) => void
}

export default function ISA101GaugeComponent({
  x,
  y,
  width,
  height,
  properties,
  style = {},
  selected = false,
  onClick,
  onContextMenu,
  draggable = true,
  onDragEnd,
  onDragStart,
  ...restProps
}: ISA101GaugeProps) {
  const groupRef = useRef<any>()
  const radius = Math.min(width, height) / 2 - 10
  const centerX = width / 2
  const centerY = height / 2

  const {
    minValue = 0,
    maxValue = 100,
    currentValue = 0,
    units = '',
  } = properties

  // Calculate angles
  const startAngle = -135 // Start at 225 degrees (lower left)
  const endAngle = 135   // End at 315 degrees (lower right)
  const angleRange = endAngle - startAngle

  // Calculate value position
  const valueRange = maxValue - minValue
  const normalizedValue = Math.max(0, Math.min(1, 
    (currentValue - minValue) / valueRange
  ))
  const valueAngle = startAngle + (angleRange * normalizedValue)

  // Helper function to convert angle to x,y coordinates
  const getXY = (angle: number, r: number) => {
    const rad = (angle * Math.PI) / 180
    return {
      x: centerX + r * Math.cos(rad),
      y: centerY + r * Math.sin(rad),
    }
  }

  // Determine if value is in alarm
  const isInAlarm = () => {
    if (properties.alarmHigh !== undefined && currentValue >= properties.alarmHigh) return true
    if (properties.alarmLow !== undefined && currentValue <= properties.alarmLow) return true
    return false
  }

  // Generate scale marks
  const generateScaleMarks = () => {
    const marks = []
    const majorTicks = 5 // Number of major tick marks
    const minorTicks = 4 // Minor ticks between major ticks

    for (let i = 0; i <= majorTicks; i++) {
      const angle = startAngle + (angleRange * i / majorTicks)
      const value = minValue + (valueRange * i / majorTicks)
      const outer = getXY(angle, radius)
      const inner = getXY(angle, radius - 10)
      const label = getXY(angle, radius - 20)

      // Major tick
      marks.push(
        <Line
          key={`major-${i}`}
          points={[outer.x, outer.y, inner.x, inner.y]}
          stroke={ISA101Colors.scale}
          strokeWidth={2}
        />
      )

      // Scale label
      marks.push(
        <Text
          key={`label-${i}`}
          x={label.x - 15}
          y={label.y - 5}
          width={30}
          text={value.toFixed(0)}
          fontSize={10}
          fill={ISA101Colors.scale}
          align="center"
        />
      )

      // Minor ticks
      if (i < majorTicks) {
        for (let j = 1; j <= minorTicks; j++) {
          const minorAngle = angle + (angleRange / majorTicks * j / (minorTicks + 1))
          const minorOuter = getXY(minorAngle, radius)
          const minorInner = getXY(minorAngle, radius - 5)
          
          marks.push(
            <Line
              key={`minor-${i}-${j}`}
              points={[minorOuter.x, minorOuter.y, minorInner.x, minorInner.y]}
              stroke={ISA101Colors.scale}
              strokeWidth={1}
            />
          )
        }
      }
    }

    return marks
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

      {/* Gauge background circle */}
      <Circle
        x={centerX}
        y={centerY}
        radius={radius + 5}
        fill={ISA101Colors.equipmentFill}
        stroke={ISA101Colors.equipmentOutline}
        strokeWidth={style.lineWidth || 2}
      />

      {/* Scale arc */}
      <Shape
        sceneFunc={(ctx, shape) => {
          ctx.beginPath()
          ctx.arc(
            centerX,
            centerY,
            radius,
            (startAngle * Math.PI) / 180,
            (endAngle * Math.PI) / 180,
            false
          )
          ctx.fillStrokeShape(shape)
        }}
        stroke={ISA101Colors.scale}
        strokeWidth={2}
      />

      {/* Alarm zones */}
      {properties.alarmHigh !== undefined && (
        <Shape
          sceneFunc={(ctx, shape) => {
            const alarmNormalized = (properties.alarmHigh! - minValue) / valueRange
            const alarmStartAngle = startAngle + (angleRange * alarmNormalized)
            ctx.beginPath()
            ctx.arc(
              centerX,
              centerY,
              radius - 3,
              (alarmStartAngle * Math.PI) / 180,
              (endAngle * Math.PI) / 180,
              false
            )
            ctx.fillStrokeShape(shape)
          }}
          stroke={ISA101Colors.alarmHigh}
          strokeWidth={4}
          opacity={0.6}
        />
      )}

      {properties.alarmLow !== undefined && (
        <Shape
          sceneFunc={(ctx, shape) => {
            const alarmNormalized = (properties.alarmLow! - minValue) / valueRange
            const alarmEndAngle = startAngle + (angleRange * alarmNormalized)
            ctx.beginPath()
            ctx.arc(
              centerX,
              centerY,
              radius - 3,
              (startAngle * Math.PI) / 180,
              (alarmEndAngle * Math.PI) / 180,
              false
            )
            ctx.fillStrokeShape(shape)
          }}
          stroke={ISA101Colors.alarmLow}
          strokeWidth={4}
          opacity={0.6}
        />
      )}

      {/* Scale marks and labels */}
      {generateScaleMarks()}

      {/* Setpoint indicator */}
      {properties.setpoint !== undefined && (
        (() => {
          const setpointNormalized = (properties.setpoint! - minValue) / valueRange
          const setpointAngle = startAngle + (angleRange * setpointNormalized)
          const sp = getXY(setpointAngle, radius + 8)
          return (
            <Shape
              sceneFunc={(ctx, shape) => {
                ctx.beginPath()
                ctx.moveTo(sp.x - 4, sp.y - 4)
                ctx.lineTo(sp.x + 4, sp.y - 4)
                ctx.lineTo(sp.x, sp.y + 4)
                ctx.closePath()
                ctx.fillStrokeShape(shape)
              }}
              fill={ISA101Colors.setpoint}
              stroke={ISA101Colors.setpoint}
              strokeWidth={1}
            />
          )
        })()
      )}

      {/* Gauge needle */}
      <Group rotation={valueAngle} x={centerX} y={centerY}>
        <Line
          points={[0, 0, radius - 15, 0]}
          stroke={isInAlarm() ? ISA101Colors.needleAlarm : ISA101Colors.needleNormal}
          strokeWidth={3}
          lineCap="round"
        />
        {/* Needle hub */}
        <Circle
          x={0}
          y={0}
          radius={6}
          fill={ISA101Colors.equipmentOutline}
        />
      </Group>

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

      {/* Digital value display */}
      {properties.showDigitalValue !== false && (
        <Group x={centerX - 40} y={centerY + radius - 20}>
          <Rect
            x={0}
            y={0}
            width={80}
            height={24}
            fill={ISA101Colors.background}
            stroke={ISA101Colors.equipmentOutline}
            strokeWidth={1}
          />
          <Text
            x={0}
            y={4}
            width={80}
            text={`${currentValue.toFixed(1)} ${units}`}
            fontSize={14}
            fontStyle="bold"
            fill={isInAlarm() ? ISA101Colors.needleAlarm : ISA101Colors.processValue}
            align="center"
          />
        </Group>
      )}

      {/* Gauge type indicator */}
      <Text
        x={0}
        y={height + 5}
        width={width}
        text={properties.gaugeType?.toUpperCase() || 'GAUGE'}
        fontSize={10}
        fill={ISA101Colors.scale}
        align="center"
      />

      {/* Trend indicator (simplified) */}
      {properties.showTrend && (
        <Group x={width - 50} y={5}>
          <Rect
            x={0}
            y={0}
            width={45}
            height={20}
            fill={ISA101Colors.background}
            stroke={ISA101Colors.equipmentOutline}
            strokeWidth={1}
          />
          <Text
            x={2}
            y={2}
            text="TREND"
            fontSize={8}
            fill={ISA101Colors.scale}
          />
          <Line
            points={[5, 15, 15, 12, 25, 14, 35, 10, 40, 13]}
            stroke={ISA101Colors.setpoint}
            strokeWidth={1}
          />
        </Group>
      )}
    </Group>
  )
}
