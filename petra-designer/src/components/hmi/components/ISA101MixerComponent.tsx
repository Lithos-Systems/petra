import { useEffect, useRef } from 'react'
import { Group, Circle, Line, Path, Text, Rect, Shape } from 'react-konva'

const ISA101Colors = {
  equipmentOutline: '#000000',
  equipmentFill: '#E6E6E6',
  running: '#00FF00',
  stopped: '#808080',
  liquidNormal: '#87CEEB',
  processValue: '#000000',
  background: '#FFFFFF',
}

interface ISA101MixerProps {
  x: number
  y: number
  width: number
  height: number
  properties: {
    tagName?: string
    running: boolean
    speed: number // RPM
    level: number // 0-100%
    agitatorType: 'paddle' | 'turbine' | 'anchor'
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

export default function ISA101MixerComponent({
  x, y, width, height, properties, style = {}, selected,
  draggable, onDragEnd, onDragStart, onClick, onContextMenu,
  ...restProps
}: ISA101MixerProps) {
  const rotationRef = useRef(0)
  const animationRef = useRef<any>()
  const groupRef = useRef<any>()

  useEffect(() => {
    if (properties.running) {
      const animate = () => {
        rotationRef.current = (rotationRef.current + properties.speed / 10) % 360
        if (groupRef.current) {
          groupRef.current.getLayer()?.batchDraw()
        }
        animationRef.current = requestAnimationFrame(animate)
      }
      animationRef.current = requestAnimationFrame(animate)
    }

    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current)
      }
    }
  }, [properties.running, properties.speed])

  const centerX = width / 2
  const tankRadius = width / 2 - 10
  const liquidHeight = (height * 0.7 * properties.level) / 100

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

      {/* Tank body - cylindrical */}
      <Shape
        sceneFunc={(ctx, shape) => {
          ctx.beginPath()
          ctx.moveTo(10, height * 0.3)
          ctx.lineTo(10, height - 10)
          ctx.quadraticCurveTo(10, height, 20, height)
          ctx.lineTo(width - 20, height)
          ctx.quadraticCurveTo(width - 10, height, width - 10, height - 10)
          ctx.lineTo(width - 10, height * 0.3)
          ctx.fillStrokeShape(shape)
        }}
        fill={ISA101Colors.equipmentFill}
        stroke={ISA101Colors.equipmentOutline}
        strokeWidth={2}
      />

      {/* Tank top - elliptical */}
      <Shape
        sceneFunc={(ctx, shape) => {
          ctx.beginPath()
          ctx.ellipse(centerX, height * 0.3, tankRadius, 15, 0, 0, Math.PI * 2)
          ctx.fillStrokeShape(shape)
        }}
        fill={ISA101Colors.equipmentFill}
        stroke={ISA101Colors.equipmentOutline}
        strokeWidth={2}
      />

      {/* Liquid */}
      {liquidHeight > 0 && (
        <Shape
          sceneFunc={(ctx, shape) => {
            const liquidTop = height - liquidHeight - 10
            ctx.beginPath()
            ctx.moveTo(11, height - 11)
            ctx.lineTo(11, liquidTop)
            ctx.lineTo(width - 11, liquidTop)
            ctx.lineTo(width - 11, height - 11)
            ctx.quadraticCurveTo(width - 11, height - 6, width - 21, height - 6)
            ctx.lineTo(21, height - 6)
            ctx.quadraticCurveTo(11, height - 6, 11, height - 11)
            ctx.fillStrokeShape(shape)
          }}
          fill={ISA101Colors.liquidNormal}
          opacity={0.7}
        />
      )}

      {/* Motor on top */}
      <Rect
        x={centerX - 20}
        y={5}
        width={40}
        height={25}
        fill={properties.running ? ISA101Colors.running : ISA101Colors.stopped}
        stroke={ISA101Colors.equipmentOutline}
        strokeWidth={2}
        cornerRadius={3}
      />
      <Text
        x={centerX - 20}
        y={12}
        width={40}
        text="M"
        fontSize={14}
        fill="#FFFFFF"
        align="center"
        fontStyle="bold"
      />

      {/* Shaft */}
      <Line
        points={[centerX, 30, centerX, height - 20]}
        stroke={ISA101Colors.equipmentOutline}
        strokeWidth={4}
      />

      {/* Agitator (rotating part) */}
      <Group x={centerX} y={height - liquidHeight / 2 - 15} rotation={rotationRef.current}>
        {properties.agitatorType === 'paddle' && (
          <>
            <Rect
              x={-30}
              y={-5}
              width={60}
              height={10}
              fill={ISA101Colors.equipmentOutline}
              stroke={ISA101Colors.equipmentOutline}
              strokeWidth={1}
            />
            <Rect
              x={-5}
              y={-30}
              width={10}
              height={60}
              fill={ISA101Colors.equipmentOutline}
              stroke={ISA101Colors.equipmentOutline}
              strokeWidth={1}
            />
          </>
        )}
        {properties.agitatorType === 'turbine' && (
          <>
            {[0, 60, 120, 180, 240, 300].map((angle) => (
              <Line
                key={angle}
                points={[0, 0, 25, 0]}
                stroke={ISA101Colors.equipmentOutline}
                strokeWidth={6}
                rotation={angle}
                lineCap="round"
              />
            ))}
            <Circle
              x={0}
              y={0}
              radius={8}
              fill={ISA101Colors.equipmentOutline}
            />
          </>
        )}
      </Group>

      {/* Tag name */}
      {properties.tagName && (
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
      )}

      {/* Speed indicator */}
      {properties.running && (
        <Text
          x={0}
          y={height + 5}
          width={width}
          text={`${properties.speed} RPM`}
          fontSize={10}
          fill={ISA101Colors.processValue}
          align="center"
        />
      )}

      {/* Temperature indicator */}
      {properties.temperature !== undefined && (
        <Group x={width - 40} y={height * 0.5}>
          <Rect
            x={0}
            y={0}
            width={35}
            height={20}
            fill={ISA101Colors.background}
            stroke={ISA101Colors.equipmentOutline}
            strokeWidth={1}
            cornerRadius={3}
          />
          <Text
            x={0}
            y={3}
            width={35}
            text={`${properties.temperature}°C`}
            fontSize={10}
            fill={ISA101Colors.processValue}
            align="center"
          />
        </Group>
      )}
    </Group>
  )
}
