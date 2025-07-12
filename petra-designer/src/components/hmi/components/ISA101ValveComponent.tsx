import { useEffect, useState, useRef } from 'react'
import { Group, Line, Shape, Text, Rect, Circle } from 'react-konva'

// ISA-101 Standard Colors
const ISA101Colors = {
  equipmentOutline: '#000000',
  equipmentFill: '#E6E6E6',
  open: '#00FF00',
  closed: '#808080',
  transitioning: '#FFFF00',
  fault: '#FF0000',
  manual: '#9370DB',
  processValue: '#000000',
  background: '#FFFFFF',
  interlocked: '#FF8C00',
}

interface ISA101ValveProps {
  x: number
  y: number
  width: number
  height: number
  properties: {
    tagName?: string
    position?: number // 0-100%
    status?: 'open' | 'closed' | 'transitioning' | 'fault'
    valveType?: 'gate' | 'ball' | 'butterfly' | 'control'
    controlMode?: 'auto' | 'manual' | 'cascade'
    interlocked?: boolean
    showPosition?: boolean
    orientation?: 'horizontal' | 'vertical'
    failPosition?: 'open' | 'closed' | 'last'
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

export default function ISA101ValveComponent({
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
}: ISA101ValveProps) {
  const [isAnimating, setIsAnimating] = useState(false)
  const animationRef = useRef<any>()
  const groupRef = useRef<any>()
  const position = properties.position ?? 0

  // Determine valve color based on status
  const getValveColor = () => {
    if (properties.interlocked) return ISA101Colors.interlocked
    switch (properties.status) {
      case 'open': return ISA101Colors.open
      case 'closed': return ISA101Colors.closed
      case 'transitioning': return ISA101Colors.transitioning
      case 'fault': return ISA101Colors.fault
      default: return ISA101Colors.equipmentFill
    }
  }

  // Animate valve during transition
  useEffect(() => {
    if (properties.status === 'transitioning') {
      setIsAnimating(true)
      let blink = true
      const animate = () => {
        blink = !blink
        if (groupRef.current) {
          groupRef.current.opacity(blink ? 1 : 0.5)
          groupRef.current.getLayer()?.batchDraw()
        }
        animationRef.current = setTimeout(animate, 500)
      }
      animationRef.current = setTimeout(animate, 500)
    } else {
      setIsAnimating(false)
      if (groupRef.current) {
        groupRef.current.opacity(1)
        groupRef.current.getLayer()?.batchDraw()
      }
    }

    return () => {
      if (animationRef.current) {
        clearTimeout(animationRef.current)
      }
    }
  }, [properties.status])

  const centerX = width / 2
  const centerY = height / 2
  const isVertical = properties.orientation === 'vertical'

  // Draw valve based on type
  const drawValve = () => {
    const valveSize = Math.min(width, height) * 0.6

    switch (properties.valveType) {
      case 'ball':
        return (
          <Group x={centerX} y={centerY}>
            {/* Ball valve body */}
            <Circle
              x={0}
              y={0}
              radius={valveSize / 2}
              fill={ISA101Colors.equipmentFill}
              stroke={properties.status === 'fault' ? ISA101Colors.fault : ISA101Colors.equipmentOutline}
              strokeWidth={properties.status === 'fault' ? 3 : 2}
            />
            {/* Ball position indicator */}
            <Rect
              x={-valveSize / 4}
              y={-valveSize / 8}
              width={valveSize / 2}
              height={valveSize / 4}
              fill={getValveColor()}
              stroke={ISA101Colors.equipmentOutline}
              strokeWidth={1}
              rotation={isVertical ?
                (position > 50 ? 0 : 90) :
                (position > 50 ? 90 : 0)}
            />
          </Group>
        )

      case 'butterfly':
        return (
          <Group x={centerX} y={centerY}>
            {/* Butterfly valve body */}
            <Circle
              x={0}
              y={0}
              radius={valveSize / 2}
              fill={ISA101Colors.equipmentFill}
              stroke={properties.status === 'fault' ? ISA101Colors.fault : ISA101Colors.equipmentOutline}
              strokeWidth={properties.status === 'fault' ? 3 : 2}
            />
            {/* Butterfly disc */}
            <Line
              points={[0, -valveSize / 2, 0, valveSize / 2]}
              stroke={getValveColor()}
              strokeWidth={valveSize / 8}
              lineCap="round"
              rotation={isVertical ? 
                (90 - position * 0.9) :
                (position * 0.9)}
            />
          </Group>
        )

      case 'control':
        return (
          <Group x={centerX} y={centerY}>
            {/* Control valve body (globe style) */}
            <Shape
              sceneFunc={(ctx, shape) => {
                const s = valveSize / 2
                ctx.beginPath()
                ctx.moveTo(-s, -s)
                ctx.lineTo(s, -s)
                ctx.lineTo(s * 0.5, 0)
                ctx.lineTo(s, s)
                ctx.lineTo(-s, s)
                ctx.lineTo(-s * 0.5, 0)
                ctx.closePath()
                ctx.fillStrokeShape(shape)
              }}
              fill={ISA101Colors.equipmentFill}
              stroke={properties.status === 'fault' ? ISA101Colors.fault : ISA101Colors.equipmentOutline}
              strokeWidth={properties.status === 'fault' ? 3 : 2}
            />
            {/* Stem position */}
            <Line
              points={[0, -valveSize / 2, 0, -valveSize]}
              stroke={ISA101Colors.equipmentOutline}
              strokeWidth={2}
            />
            <Circle
              x={0}
              y={-valveSize + (position / 100) * (valveSize / 2)}
              radius={valveSize / 8}
              fill={getValveColor()}
              stroke={ISA101Colors.equipmentOutline}
              strokeWidth={1}
            />
          </Group>
        )

      default: // gate valve
        return (
          <Group x={centerX} y={centerY}>
            {/* Gate valve body */}
            <Shape
              sceneFunc={(ctx, shape) => {
                const s = valveSize / 2
                ctx.beginPath()
                ctx.moveTo(-s, -s * 0.6)
                ctx.lineTo(s, -s * 0.6)
                ctx.lineTo(s, s * 0.6)
                ctx.lineTo(-s, s * 0.6)
                ctx.closePath()
                ctx.fillStrokeShape(shape)
              }}
              fill={ISA101Colors.equipmentFill}
              stroke={properties.status === 'fault' ? ISA101Colors.fault : ISA101Colors.equipmentOutline}
              strokeWidth={properties.status === 'fault' ? 3 : 2}
            />
            {/* Gate position */}
            <Rect
              x={-valveSize / 3}
              y={-valveSize * 0.6 + (1 - position / 100) * valveSize * 1.2}
              width={valveSize * 2 / 3}
              height={valveSize * 0.2}
              fill={getValveColor()}
              stroke={ISA101Colors.equipmentOutline}
              strokeWidth={1}
            />
            {/* Stem */}
            <Line
              points={[0, -valveSize * 0.6, 0, -valveSize]}
              stroke={ISA101Colors.equipmentOutline}
              strokeWidth={2}
            />
          </Group>
        )
    }
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

      {/* Pipe connections */}
      {isVertical ? (
        <>
          <Line
            points={[centerX, 0, centerX, centerY - Math.min(width, height) * 0.3]}
            stroke={ISA101Colors.equipmentOutline}
            strokeWidth={4}
          />
          <Line
            points={[centerX, centerY + Math.min(width, height) * 0.3, centerX, height]}
            stroke={ISA101Colors.equipmentOutline}
            strokeWidth={4}
          />
        </>
      ) : (
        <>
          <Line
            points={[0, centerY, centerX - Math.min(width, height) * 0.3, centerY]}
            stroke={ISA101Colors.equipmentOutline}
            strokeWidth={4}
          />
          <Line
            points={[centerX + Math.min(width, height) * 0.3, centerY, width, centerY]}
            stroke={ISA101Colors.equipmentOutline}
            strokeWidth={4}
          />
        </>
      )}

      {/* Valve body */}
      {drawValve()}

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

      {/* Position indicator (if enabled) */}
      {properties.showPosition && (
        <Group x={0} y={height + 5}>
          <Rect
            x={width / 2 - 30}
            y={0}
            width={60}
            height={18}
            fill={ISA101Colors.background}
            stroke={ISA101Colors.equipmentOutline}
            strokeWidth={1}
          />
          <Text
            x={width / 2 - 30}
            y={2}
            width={60}
            text={`${position.toFixed(0)}%`}
            fontSize={12}
            fontStyle="bold"
            fill={ISA101Colors.processValue}
            align="center"
          />
        </Group>
      )}

      {/* Control mode indicator */}
      {properties.controlMode === 'manual' && (
        <Group x={width - 20} y={5}>
          <Circle
            x={0}
            y={0}
            radius={8}
            fill={ISA101Colors.manual}
            stroke={ISA101Colors.equipmentOutline}
            strokeWidth={1}
          />
          <Text
            x={-5}
            y={-5}
            text="M"
            fontSize={10}
            fontStyle="bold"
            fill="#FFFFFF"
          />
        </Group>
      )}

      {/* Interlock indicator */}
      {properties.interlocked && (
        <Group x={centerX - 20} y={height - 20}>
          <Rect
            x={0}
            y={0}
            width={40}
            height={16}
            fill={ISA101Colors.interlocked}
            stroke={ISA101Colors.equipmentOutline}
            strokeWidth={1}
          />
          <Text
            x={0}
            y={2}
            width={40}
            text="INTLK"
            fontSize={10}
            fontStyle="bold"
            fill="#FFFFFF"
            align="center"
          />
        </Group>
      )}

      {/* Fail position indicator */}
      {properties.failPosition && (
        <Text
          x={5}
          y={height - 10}
          text={`F${properties.failPosition === 'open' ? 'O' : properties.failPosition === 'closed' ? 'C' : 'L'}`}
          fontSize={10}
          fill={ISA101Colors.processValue}
        />
      )}

      {/* Fault indicator */}
      {properties.status === 'fault' && (
        <Group x={5} y={5}>
          <Shape
            sceneFunc={(ctx, shape) => {
              // Draw warning triangle
              ctx.beginPath()
              ctx.moveTo(10, 0)
              ctx.lineTo(0, 17)
              ctx.lineTo(20, 17)
              ctx.closePath()
              ctx.fillStrokeShape(shape)
            }}
            fill={ISA101Colors.fault}
            stroke={ISA101Colors.equipmentOutline}
            strokeWidth={1}
          />
          <Text
            x={7}
            y={8}
            text="!"
            fontSize={12}
            fontStyle="bold"
            fill="#FFFFFF"
          />
        </Group>
      )}
    </Group>
  )
}
