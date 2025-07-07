// src/components/hmi/components/ConveyorComponent.tsx

import { useEffect, useRef } from 'react'
import { Group, Rect, Line, Circle, Text } from 'react-konva'

interface ConveyorProps {
  x: number
  y: number
  width: number
  height: number
  properties: {
    running: boolean
    speed: number // 0-100%
    direction: 'forward' | 'reverse'
    material: boolean // has material on belt
  }
  style: any
  [key: string]: any
}

export default function ConveyorComponent({
  x, y, width, height, properties, style, ...rest
}: ConveyorProps) {
  const animationRef = useRef(0)
  const frameRef = useRef<any>()

  useEffect(() => {
    if (properties.running) {
      const animate = () => {
        const speed = (properties.speed / 100) * 5
        animationRef.current += properties.direction === 'forward' ? speed : -speed
        if (animationRef.current > 20) animationRef.current = 0
        if (animationRef.current < 0) animationRef.current = 20
        frameRef.current = requestAnimationFrame(animate)
      }
      frameRef.current = requestAnimationFrame(animate)
    }

    return () => {
      if (frameRef.current) {
        cancelAnimationFrame(frameRef.current)
      }
    }
  }, [properties.running, properties.speed, properties.direction])

  const rollerCount = Math.floor(width / 30)
  const beltY = height * 0.3

  return (
    <Group x={x} y={y} {...rest}>
      {/* Conveyor frame */}
      <Rect
        x={0}
        y={0}
        width={width}
        height={height}
        fill={style.fill || '#d1d5db'}
        stroke={style.stroke || '#333333'}
        strokeWidth={style.strokeWidth || 2}
      />

      {/* Support legs */}
      <Line
        points={[20, height, 20, height + 20]}
        stroke="#666666"
        strokeWidth={4}
      />
      <Line
        points={[width - 20, height, width - 20, height + 20]}
        stroke="#666666"
        strokeWidth={4}
      />

      {/* Belt */}
      <Rect
        x={5}
        y={beltY}
        width={width - 10}
        height={height * 0.4}
        fill={properties.running ? '#4b5563' : '#6b7280'}
        stroke="#333333"
        strokeWidth={1}
      />

      {/* Belt segments for animation */}
      {Array.from({ length: Math.ceil(width / 20) + 1 }).map((_, i) => {
        const segmentX = (i * 20 - animationRef.current) % width
        return (
          <Line
            key={i}
            points={[
              segmentX, beltY,
              segmentX, beltY + height * 0.4
            ]}
            stroke="#333333"
            strokeWidth={1}
            opacity={0.3}
          />
        )
      })}

      {/* Rollers */}
      {Array.from({ length: rollerCount }).map((_, i) => {
        const rollerX = (i + 1) * (width / (rollerCount + 1))
        return (
          <Circle
            key={i}
            x={rollerX}
            y={beltY + height * 0.2}
            radius={8}
            fill="#9ca3af"
            stroke="#333333"
            strokeWidth={1}
          />
        )
      })}

      {/* Material on belt */}
      {properties.material && (
        <Rect
          x={width * 0.4}
          y={beltY - 15}
          width={width * 0.2}
          height={15}
          fill="#8b4513"
          stroke="#654321"
          strokeWidth={1}
          cornerRadius={3}
        />
      )}

      {/* Direction indicator */}
      <Group x={width / 2} y={height * 0.8}>
        {properties.direction === 'forward' ? (
          <Line
            points={[-15, 0, 15, 0, 10, -5, 15, 0, 10, 5]}
            stroke={properties.running ? '#10b981' : '#6b7280'}
            strokeWidth={2}
          />
        ) : (
          <Line
            points={[15, 0, -15, 0, -10, -5, -15, 0, -10, 5]}
            stroke={properties.running ? '#10b981' : '#6b7280'}
            strokeWidth={2}
          />
        )}
      </Group>

      {/* Speed indicator */}
      {properties.running && (
        <Text
          x={0}
          y={height + 25}
          width={width}
          text={`${properties.speed}%`}
          fontSize={11}
          fill="#666666"
          align="center"
        />
      )}
    </Group>
  )
}
