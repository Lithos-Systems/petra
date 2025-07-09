// Enhanced Tank Component with modern graphics and animations
import { useEffect, useState, useRef } from 'react'
import Konva from 'konva'
import { Group, Rect, Shape, Text, Line, Circle } from 'react-konva'

interface TankComponentProps {
  x: number
  y: number
  width: number
  height: number
  properties: {
    maxLevel: number
    currentLevel: number
    alarmHigh: number
    alarmLow: number
    showLabel: boolean
    units: string
    liquidColor?: string
    showWaveAnimation?: boolean
    fillColor?: string
    label?: string
    isMetric?: boolean
    temperature?: number
    showGlassLevel?: boolean
  }
  style?: {
    fill?: string
    stroke?: string
    strokeWidth?: number
  }
  bindings?: any[]
}

export default function EnhancedTankComponent({
  x,
  y,
  width,
  height,
  properties,
  style = {},
  bindings = []
}: TankComponentProps) {
  const [animatedLevel, setAnimatedLevel] = useState(properties.currentLevel)
  const [waveOffset, setWaveOffset] = useState(0)
  const [alarmPulse, setAlarmPulse] = useState(0)
  const groupRef = useRef<any>(null)

  // Animate level changes
  useEffect(() => {
    const anim = new Konva.Animation((frame) => {
      if (!frame) return
      
      // Smooth level animation
      const diff = properties.currentLevel - animatedLevel
      if (Math.abs(diff) > 0.1) {
        setAnimatedLevel(prev => prev + diff * 0.1)
      }
      
      // Wave animation
      if (properties.showWaveAnimation) {
        setWaveOffset(frame.time * 0.001)
      }
      
      // Alarm pulse animation
      if (properties.currentLevel >= properties.alarmHigh || properties.currentLevel <= properties.alarmLow) {
        setAlarmPulse(Math.sin(frame.time * 0.003) * 0.5 + 0.5)
      }
    })
    
    anim.start()
    return () => anim.stop()
  }, [properties.currentLevel, animatedLevel, properties.showWaveAnimation, properties.alarmHigh, properties.alarmLow])

  const levelPercent = (animatedLevel / properties.maxLevel) * 100
  const levelHeight = (height * 0.7) * (levelPercent / 100)
  const liquidY = y + height * 0.15 + (height * 0.7) - levelHeight

  // Calculate temperature-based color
  const getLiquidColor = () => {
    if (!properties.temperature) return properties.liquidColor || '#3b82f6'
    
    const temp = properties.temperature
    if (temp < 20) return '#60a5fa' // Cold - light blue
    if (temp < 40) return '#3b82f6' // Normal - blue
    if (temp < 60) return '#f59e0b' // Warm - amber
    return '#ef4444' // Hot - red
  }

  const isAlarming = animatedLevel >= properties.alarmHigh || animatedLevel <= properties.alarmLow

  return (
    <Group x={x} y={y} ref={groupRef}>
      {/* Tank shadow */}
      <Rect
        x={2}
        y={2}
        width={width}
        height={height}
        fill="rgba(0,0,0,0.2)"
        cornerRadius={4}
      />
      
      {/* Tank body with gradient */}
      <Shape
        sceneFunc={(context, shape) => {
          const gradient = context.createLinearGradient(0, 0, width, 0)
          gradient.addColorStop(0, '#e5e7eb')
          gradient.addColorStop(0.5, '#f3f4f6')
          gradient.addColorStop(1, '#e5e7eb')
          
          context.beginPath()
          context.roundRect(0, 0, width, height, 4)
          context.fillStyle = gradient
          context.fill()
          context.strokeStyle = style.stroke || '#6b7280'
          context.lineWidth = style.strokeWidth || 2
          context.stroke()
        }}
      />
      
      {/* Tank top */}
      <Shape
        sceneFunc={(context) => {
          const gradient = context.createLinearGradient(0, 0, 0, height * 0.15)
          gradient.addColorStop(0, '#f9fafb')
          gradient.addColorStop(1, '#e5e7eb')
          
          context.beginPath()
          context.roundRect(0, 0, width, height * 0.15, [4, 4, 0, 0])
          context.fillStyle = gradient
          context.fill()
          context.strokeStyle = '#6b7280'
          context.lineWidth = 2
          context.stroke()
        }}
      />
      
      {/* Tank bottom */}
      <Shape
        y={height * 0.85}
        sceneFunc={(context) => {
          const gradient = context.createLinearGradient(0, 0, 0, height * 0.15)
          gradient.addColorStop(0, '#d1d5db')
          gradient.addColorStop(1, '#9ca3af')
          
          context.beginPath()
          context.roundRect(0, 0, width, height * 0.15, [0, 0, 4, 4])
          context.fillStyle = gradient
          context.fill()
          context.strokeStyle = '#6b7280'
          context.lineWidth = 2
          context.stroke()
        }}
      />
      
      {/* Liquid with wave effect */}
      <Group clip={{
        x: 2,
        y: height * 0.15,
        width: width - 4,
        height: height * 0.7
      }}>
        <Shape
          sceneFunc={(context) => {
            const liquidColor = getLiquidColor()
            const gradient = context.createLinearGradient(0, liquidY, 0, y + height * 0.85)
            
            // Create gradient based on liquid color
            if (liquidColor === '#3b82f6') {
              gradient.addColorStop(0, '#60a5fa')
              gradient.addColorStop(0.5, '#3b82f6')
              gradient.addColorStop(1, '#1e40af')
            } else if (liquidColor === '#ef4444') {
              gradient.addColorStop(0, '#f87171')
              gradient.addColorStop(0.5, '#ef4444')
              gradient.addColorStop(1, '#dc2626')
            } else if (liquidColor === '#f59e0b') {
              gradient.addColorStop(0, '#fbbf24')
              gradient.addColorStop(0.5, '#f59e0b')
              gradient.addColorStop(1, '#d97706')
            } else {
              gradient.addColorStop(0, '#93c5fd')
              gradient.addColorStop(0.5, '#60a5fa')
              gradient.addColorStop(1, '#3b82f6')
            }
            
            context.beginPath()
            context.moveTo(2, liquidY)
            
            // Add wave effect
            if (properties.showWaveAnimation) {
              for (let i = 0; i <= width - 4; i++) {
                const waveHeight = Math.sin((i * 0.02) + waveOffset) * 3
                context.lineTo(2 + i, liquidY + waveHeight)
              }
            } else {
              context.lineTo(width - 2, liquidY)
            }
            
            context.lineTo(width - 2, y + height * 0.85)
            context.lineTo(2, y + height * 0.85)
            context.closePath()
            
            context.fillStyle = gradient
            context.fill()
            
            // Add bubble effect for hot liquids
            if (properties.temperature && properties.temperature > 60) {
              context.globalAlpha = 0.3
              for (let i = 0; i < 5; i++) {
                const bubbleX = Math.random() * (width - 10) + 5
                const bubbleY = liquidY + Math.random() * levelHeight
                const bubbleR = Math.random() * 3 + 1
                
                context.beginPath()
                context.arc(bubbleX, bubbleY, bubbleR, 0, Math.PI * 2)
                context.fillStyle = '#ffffff'
                context.fill()
              }
              context.globalAlpha = 1
            }
          }}
        />
      </Group>
      
      {/* Glass level indicator */}
      {properties.showGlassLevel && (
        <Group x={width + 10} y={height * 0.15}>
          {/* Glass tube background */}
          <Rect
            x={0}
            y={0}
            width={12}
            height={height * 0.7}
            fill="rgba(255,255,255,0.9)"
            stroke="#6b7280"
            strokeWidth={1}
            cornerRadius={6}
          />
          
          {/* Liquid in glass tube */}
          <Rect
            x={2}
            y={(height * 0.7) - levelHeight}
            width={8}
            height={levelHeight}
            fill={getLiquidColor()}
            cornerRadius={4}
          />
          
          {/* Scale marks */}
          {[0, 25, 50, 75, 100].map((mark) => (
            <Group key={mark} y={(height * 0.7) * (1 - mark / 100)}>
              <Line
                points={[12, 0, 18, 0]}
                stroke="#6b7280"
                strokeWidth={1}
              />
              <Text
                x={20}
                y={-6}
                text={`${mark}%`}
                fontSize={8}
                fill="#6b7280"
              />
            </Group>
          ))}
        </Group>
      )}
      
      {/* High/Low level marks */}
      <Group>
        {/* High level mark */}
        <Line
          points={[
            5,
            height * 0.15 + (height * 0.7) * (1 - properties.alarmHigh / properties.maxLevel),
            width - 5,
            height * 0.15 + (height * 0.7) * (1 - properties.alarmHigh / properties.maxLevel)
          ]}
          stroke="#ef4444"
          strokeWidth={2}
          dash={[5, 5]}
          opacity={0.7}
        />
        <Text
          x={5}
          y={height * 0.15 + (height * 0.7) * (1 - properties.alarmHigh / properties.maxLevel) - 12}
          text="HH"
          fontSize={10}
          fill="#ef4444"
          fontStyle="bold"
        />
        
        {/* Low level mark */}
        <Line
          points={[
            5,
            height * 0.15 + (height * 0.7) * (1 - properties.alarmLow / properties.maxLevel),
            width - 5,
            height * 0.15 + (height * 0.7) * (1 - properties.alarmLow / properties.maxLevel)
          ]}
          stroke="#f59e0b"
          strokeWidth={2}
          dash={[5, 5]}
          opacity={0.7}
        />
        <Text
          x={5}
          y={height * 0.15 + (height * 0.7) * (1 - properties.alarmLow / properties.maxLevel) - 12}
          text="LL"
          fontSize={10}
          fill="#f59e0b"
          fontStyle="bold"
        />
      </Group>
      
      {/* Alarm indicators */}
      {isAlarming && (
        <Group>
          {/* Alarm background pulse */}
          <Rect
            x={-5}
            y={-5}
            width={width + 10}
            height={height + 10}
            stroke="#ef4444"
            strokeWidth={3}
            cornerRadius={6}
            opacity={alarmPulse * 0.5}
          />
          
          {/* Alarm icon */}
          <Circle
            x={width - 15}
            y={15}
            radius={10}
            fill="#ef4444"
            opacity={0.8 + alarmPulse * 0.2}
          />
          <Text
            x={width - 20}
            y={10}
            text="!"
            fontSize={14}
            fill="#ffffff"
            fontStyle="bold"
            align="center"
          />
        </Group>
      )}
      
      {/* Level text and label */}
      {properties.showLabel && (
        <Group y={height + 10}>
          {/* Level value */}
          <Text
            x={0}
            y={0}
            width={width}
            text={`${animatedLevel.toFixed(1)} ${properties.units}`}
            fontSize={16}
            fill={isAlarming ? '#ef4444' : '#1f2937'}
            align="center"
            fontStyle="bold"
          />
          
          {/* Tank label */}
          {properties.label && (
            <Text
              x={0}
              y={20}
              width={width}
              text={properties.label}
              fontSize={12}
              fill="#6b7280"
              align="center"
            />
          )}
          
          {/* Temperature display */}
          {properties.temperature !== undefined && (
            <Text
              x={0}
              y={35}
              width={width}
              text={`${properties.temperature}°${properties.isMetric ? 'C' : 'F'}`}
              fontSize={11}
              fill={properties.temperature > 60 ? '#ef4444' : '#6b7280'}
              align="center"
            />
          )}
        </Group>
      )}
      
      {/* Connection points */}
      <Group>
        {/* Inlet */}
        <Circle
          x={width}
          y={height * 0.3}
          radius={4}
          fill="#6b7280"
          stroke="#374151"
          strokeWidth={1}
        />
        
        {/* Outlet */}
        <Circle
          x={width / 2}
          y={height}
          radius={4}
          fill="#6b7280"
          stroke="#374151"
          strokeWidth={1}
        />
        
        {/* Overflow */}
        <Circle
          x={0}
          y={height * 0.2}
          radius={3}
          fill="#f59e0b"
          stroke="#374151"
          strokeWidth={1}
        />
      </Group>
    </Group>
  )
}
