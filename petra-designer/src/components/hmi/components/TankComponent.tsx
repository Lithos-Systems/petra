// src/components/hmi/components/TankComponent.tsx

import { useEffect, useState, useRef } from 'react'
import { Group, Rect, Text, Line, Shape } from 'react-konva'
import type { TankProperties } from '@/types/hmi'

// Helper function to adjust color brightness
function adjustColor(color: string, factor: number): string {
  const hex = color.replace('#', '')
  const r = parseInt(hex.substr(0, 2), 16)
  const g = parseInt(hex.substr(2, 2), 16)
  const b = parseInt(hex.substr(4, 2), 16)
  
  return `#${[r, g, b]
    .map(c => Math.round(c * factor))
    .map(c => Math.min(255, c))
    .map(c => c.toString(16).padStart(2, '0'))
    .join('')}`
}

interface TankComponentProps {
  x: number
  y: number
  width: number
  height: number
  properties: TankProperties
  style: any
  isSelected?: boolean
  draggable?: boolean
  onDragEnd?: (e: any) => void
  onClick?: () => void
}

export default function TankComponent({
  x,
  y,
  width,
  height,
  properties,
  style,
  isSelected,
  draggable = true,
  onDragEnd,
  onClick,
}: TankComponentProps) {
  const [currentLevel] = useState(properties.currentLevel || 50)
  const [animatedLevel, setAnimatedLevel] = useState(currentLevel)
  const animationRef = useRef<any>()

  // Smooth level animation
  useEffect(() => {
    const startLevel = animatedLevel
    const endLevel = currentLevel
    const duration = 1000 // 1 second animation
    const startTime = Date.now()

    const animate = () => {
      const elapsed = Date.now() - startTime
      const progress = Math.min(elapsed / duration, 1)
      
      // Easing function
      const easeInOut = progress < 0.5
        ? 2 * progress * progress
        : 1 - Math.pow(-2 * progress + 2, 2) / 2

      const newLevel = startLevel + (endLevel - startLevel) * easeInOut
      setAnimatedLevel(newLevel)

      if (progress < 1) {
        animationRef.current = requestAnimationFrame(animate)
      }
    }

    animationRef.current = requestAnimationFrame(animate)

    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current)
      }
    }
  }, [currentLevel])

  // Calculate fill height
  const fillHeight = (animatedLevel / properties.maxLevel) * height * 0.85
  const fillY = y + height * 0.85 - fillHeight

  // Determine liquid color based on alarms
  const getLiquidColor = () => {
    if (currentLevel >= properties.alarmHigh) return '#ff4444'
    if (currentLevel <= properties.alarmLow) return '#ffaa44'
    return properties.liquidColor || '#4488ff'
  }

  // Enhanced wave animation for liquid surface
  const [waveOffset, setWaveOffset] = useState(0)
  const waveAnimationRef = useRef<any>()
  
  useEffect(() => {
    if (properties.showWaveAnimation) {
      const animate = () => {
        setWaveOffset(prev => prev + 0.05)
        waveAnimationRef.current = requestAnimationFrame(animate)
      }
      waveAnimationRef.current = requestAnimationFrame(animate)
    }
    
    return () => {
      if (waveAnimationRef.current) {
        cancelAnimationFrame(waveAnimationRef.current)
      }
    }
  }, [properties.showWaveAnimation])
  
  const wavePoints: number[] = []
  const waveAmplitude = Math.min(5, (animatedLevel / 100) * 8)
  const waveFrequency = 0.02
  for (let i = 0; i <= width; i += 2) {
    const waveY = Math.sin((i * waveFrequency) + waveOffset) * waveAmplitude
    wavePoints.push(i, waveY)
  }

  return (
    <Group
      x={x}
      y={y}
      draggable={draggable}
      onDragEnd={onDragEnd}
      onClick={onClick}
      onTap={onClick}
    >
      {/* Selection indicator */}
      {isSelected && (
        <Rect
          x={-5}
          y={-5}
          width={width + 10}
          height={height + 10}
          stroke="#3b82f6"
          strokeWidth={2}
          dash={[5, 5]}
          fill="transparent"
        />
      )}

      {/* Tank body */}
      <Rect
        x={0}
        y={0}
        width={width}
        height={height * 0.85}
        fill={style.fill || 'transparent'}
        stroke={style.stroke || '#333333'}
        strokeWidth={style.strokeWidth || 2}
        cornerRadius={[0, 0, 10, 10]}
      />

      {/* Tank top */}
      <Rect
        x={width * 0.2}
        y={-height * 0.05}
        width={width * 0.6}
        height={height * 0.05}
        fill={style.stroke || '#333333'}
        stroke={style.stroke || '#333333'}
        strokeWidth={1}
      />

      {/* Liquid fill with clipping */}
      <Group clipFunc={(ctx) => {
        ctx.beginPath()
        ctx.moveTo(2, 2)
        ctx.lineTo(2, height * 0.85 - 2)
        ctx.quadraticCurveTo(2, height * 0.85 - 2, 12, height * 0.85 - 2)
        ctx.lineTo(width - 12, height * 0.85 - 2)
        ctx.quadraticCurveTo(width - 2, height * 0.85 - 2, width - 2, height * 0.85 - 2)
        ctx.lineTo(width - 2, 2)
        ctx.closePath()
      }}>
        {/* Gradient liquid fill */}
        <Rect
          x={2}
          y={fillY}
          width={width - 4}
          height={fillHeight + 10}
          fillLinearGradientStartPoint={{ x: 0, y: 0 }}
          fillLinearGradientEndPoint={{ x: 0, y: fillHeight }}
          fillLinearGradientColorStops={[
            0, getLiquidColor(),
            0.5, adjustColor(getLiquidColor(), 0.8),
            1, adjustColor(getLiquidColor(), 0.6)
          ]}
          opacity={0.9}
        />

        {/* Wave animation on surface */}
        {properties.showWaveAnimation && (
          <Shape
            x={0}
            y={fillY}
            sceneFunc={(context, shape) => {
              context.beginPath()
              context.moveTo(0, 0)
              for (let i = 0; i < wavePoints.length; i += 2) {
                context.lineTo(wavePoints[i], wavePoints[i + 1])
              }
              context.lineTo(width, 10)
              context.lineTo(0, 10)
              context.closePath()
              context.fillStrokeShape(shape)
            }}
            fill={getLiquidColor()}
            opacity={0.9}
          />
        )}
      </Group>

      {/* Level marks */}
      {[0, 25, 50, 75, 100].map((mark) => {
        const markY = height * 0.85 * (1 - mark / 100)
        return (
          <Group key={mark}>
            <Line
              points={[width - 15, markY, width - 5, markY]}
              stroke="#666666"
              strokeWidth={1}
            />
            <Text
              x={width + 5}
              y={markY - 6}
              text={`${mark}%`}
              fontSize={10}
              fill="#666666"
            />
          </Group>
        )
      })}

      {/* Alarm indicators */}
      {currentLevel >= properties.alarmHigh && (
        <Group>
          <Rect
            x={5}
            y={5}
            width={30}
            height={20}
            fill="#ff0000"
            cornerRadius={3}
          />
          <Text
            x={8}
            y={9}
            text="HI"
            fontSize={12}
            fill="#ffffff"
            fontStyle="bold"
          />
        </Group>
      )}

      {currentLevel <= properties.alarmLow && (
        <Group>
          <Rect
            x={5}
            y={5}
            width={30}
            height={20}
            fill="#ff8800"
            cornerRadius={3}
          />
          <Text
            x={8}
            y={9}
            text="LO"
            fontSize={12}
            fill="#ffffff"
            fontStyle="bold"
          />
        </Group>
      )}

      {/* Level text */}
      {properties.showLabel && (
        <Group y={height * 0.9}>
          <Text
            x={0}
            y={0}
            width={width}
            text={`${currentLevel.toFixed(1)}${properties.units}`}
            fontSize={14}
            fill="#333333"
            align="center"
            fontStyle="bold"
          />
          <Text
            x={0}
            y={16}
            width={width}
            text="Level"
            fontSize={11}
            fill="#666666"
            align="center"
          />
        </Group>
      )}

      {/* Inlet pipe */}
      <Line
        points={[width, height * 0.2, width + 20, height * 0.2]}
        stroke="#666666"
        strokeWidth={6}
      />

      {/* Outlet pipe */}
      <Line
        points={[width / 2, height * 0.85, width / 2, height * 0.95]}
        stroke="#666666"
        strokeWidth={6}
      />
    </Group>
  )
}

// Hook for real-time signal updates (to be implemented)
export function useTankSignals(_bindings: any[]) {
  // This will connect to PETRA's signal bus via WebSocket
  // For now, return mock data
  return {
    level: 65,
    temperature: 72,
    pressure: 45,
  }
}
