// Enhanced Pump Component with realistic animations and effects
import { useEffect, useState, useRef } from 'react'
import Konva from 'konva'
import { Group, Circle, Shape, Text, Arc, Line, Rect } from 'react-konva'

interface PumpComponentProps {
  x: number
  y: number
  width: number
  height: number
  properties: {
    running: boolean
    fault: boolean
    speed: number // 0-100%
    flowRate?: number
    pressure?: number
    temperature?: number
    vibration?: number
    efficiency?: number
    runHours?: number
    showStatus?: boolean
    runAnimation?: boolean
    pumpType?: 'centrifugal' | 'positive-displacement' | 'axial'
    direction?: 'horizontal' | 'vertical'
  }
  style?: {
    fill?: string
    stroke?: string
    strokeWidth?: number
    runningColor?: string
    faultColor?: string
  }
  bindings?: any[]
}

export default function EnhancedPumpComponent({
  x,
  y,
  width,
  height,
  properties,
  style = {},
  bindings = []
}: PumpComponentProps) {
  const [rotation, setRotation] = useState(0)
  const [vibrationOffset, setVibrationOffset] = useState({ x: 0, y: 0 })
  const [glowOpacity, setGlowOpacity] = useState(0)
  const groupRef = useRef<any>(null)

  useEffect(() => {
    if (!properties.running || !properties.runAnimation) {
      setVibrationOffset({ x: 0, y: 0 })
      return
    }

    const anim = new Konva.Animation((frame) => {
      if (!frame) return
      
      // Rotation animation based on speed
      const speedFactor = (properties.speed || 100) / 100
      setRotation(prev => prev + (6 * speedFactor))
      
      // Vibration effect
      if (properties.vibration || properties.running) {
        const vibrationLevel = properties.vibration || 0.5
        setVibrationOffset({
          x: Math.sin(frame.time * 0.05) * vibrationLevel,
          y: Math.cos(frame.time * 0.05) * vibrationLevel * 0.5
        })
      }
      
      // Glow pulse effect
      setGlowOpacity(Math.sin(frame.time * 0.002) * 0.3 + 0.7)
    })
    
    anim.start()
    return () => anim.stop()
  }, [properties.running, properties.runAnimation, properties.speed, properties.vibration])

  const centerX = width / 2
  const centerY = height / 2
  const radius = Math.min(width, height) / 2 - 10

  // Calculate colors based on state
  const getPumpColor = () => {
    if (properties.fault) return style.faultColor || '#ef4444'
    if (properties.running) return style.runningColor || '#10b981'
    return style.fill || '#6b7280'
  }

  const getEfficiencyColor = (efficiency: number) => {
    if (efficiency >= 80) return '#10b981' // Green
    if (efficiency >= 60) return '#f59e0b' // Amber
    return '#ef4444' // Red
  }

  const getTemperatureColor = (temp: number) => {
    if (temp < 60) return '#3b82f6' // Blue
    if (temp < 80) return '#10b981' // Green
    if (temp < 100) return '#f59e0b' // Amber
    return '#ef4444' // Red
  }

  return (
    <Group 
      x={x + vibrationOffset.x} 
      y={y + vibrationOffset.y} 
      ref={groupRef}
    >
      {/* Shadow */}
      <Circle
        x={centerX + 2}
        y={centerY + 2}
        radius={radius + 5}
        fill="rgba(0,0,0,0.2)"
      />
      
      {/* Outer glow when running */}
      {properties.running && (
        <Circle
          x={centerX}
          y={centerY}
          radius={radius + 8}
          stroke={getPumpColor()}
          strokeWidth={3}
          opacity={glowOpacity * 0.3}
        />
      )}
      
      {/* Pump casing with gradient */}
      <Shape
        sceneFunc={(context, shape) => {
          const gradient = context.createRadialGradient(
            centerX - radius / 3,
            centerY - radius / 3,
            0,
            centerX,
            centerY,
            radius
          )
          
          const baseColor = getPumpColor()
          if (baseColor === '#10b981') {
            gradient.addColorStop(0, '#34d399')
            gradient.addColorStop(0.7, '#10b981')
            gradient.addColorStop(1, '#059669')
          } else if (baseColor === '#ef4444') {
            gradient.addColorStop(0, '#f87171')
            gradient.addColorStop(0.7, '#ef4444')
            gradient.addColorStop(1, '#dc2626')
          } else {
            gradient.addColorStop(0, '#9ca3af')
            gradient.addColorStop(0.7, '#6b7280')
            gradient.addColorStop(1, '#4b5563')
          }
          
          context.beginPath()
          context.arc(centerX, centerY, radius, 0, Math.PI * 2)
          context.fillStyle = gradient
          context.fill()
          context.strokeStyle = style.stroke || '#374151'
          context.lineWidth = style.strokeWidth || 3
          context.stroke()
        }}
      />
      
      {/* Impeller/rotor */}
      <Group rotation={rotation} offsetX={centerX} offsetY={centerY} x={centerX} y={centerY}>
        {properties.pumpType === 'centrifugal' ? (
          // Centrifugal impeller
          <Shape
            sceneFunc={(context) => {
              context.translate(centerX, centerY)
              
              for (let i = 0; i < 6; i++) {
                context.save()
                context.rotate((i * Math.PI * 2) / 6)
                
                context.beginPath()
                context.moveTo(0, 0)
                context.quadraticCurveTo(
                  radius * 0.3,
                  -radius * 0.1,
                  radius * 0.7,
                  -radius * 0.05
                )
                context.quadraticCurveTo(
                  radius * 0.5,
                  radius * 0.1,
                  0,
                  0
                )
                
                context.fillStyle = 'rgba(255,255,255,0.3)'
                context.fill()
                context.strokeStyle = 'rgba(255,255,255,0.5)'
                context.lineWidth = 1
                context.stroke()
                
                context.restore()
              }
            }}
          />
        ) : (
          // Positive displacement gears
          <>
            <Circle
              x={centerX - radius * 0.3}
              y={centerY}
              radius={radius * 0.35}
              fill="rgba(255,255,255,0.2)"
              stroke="rgba(255,255,255,0.4)"
              strokeWidth={2}
            />
            <Circle
              x={centerX + radius * 0.3}
              y={centerY}
              radius={radius * 0.35}
              fill="rgba(255,255,255,0.2)"
              stroke="rgba(255,255,255,0.4)"
              strokeWidth={2}
            />
          </>
        )}
        
        {/* Center hub */}
        <Circle
          x={centerX}
          y={centerY}
          radius={radius * 0.15}
          fill="#374151"
          stroke="#1f2937"
          strokeWidth={2}
        />
      </Group>
      
      {/* Flow direction arrows */}
      {properties.running && (
        <Group opacity={0.7}>
          {/* Inlet arrow */}
          <Shape
            x={-radius}
            y={centerY}
            sceneFunc={(context) => {
              context.beginPath()
              context.moveTo(0, -8)
              context.lineTo(15, 0)
              context.lineTo(0, 8)
              context.strokeStyle = '#3b82f6'
              context.lineWidth = 3
              context.stroke()
            }}
          />
          
          {/* Outlet arrow */}
          <Shape
            x={width + radius - 15}
            y={centerY}
            sceneFunc={(context) => {
              context.beginPath()
              context.moveTo(0, -8)
              context.lineTo(15, 0)
              context.lineTo(0, 8)
              context.strokeStyle = '#3b82f6'
              context.lineWidth = 3
              context.stroke()
            }}
          />
        </Group>
      )}
      
      {/* Status indicators */}
      {properties.showStatus && (
        <Group y={height + 10}>
          {/* Main status */}
          <Text
            x={0}
            y={0}
            width={width}
            text={properties.fault ? 'FAULT' : (properties.running ? 'RUNNING' : 'STOPPED')}
            fontSize={12}
            fill={getPumpColor()}
            align="center"
            fontStyle="bold"
          />
          
          {/* Speed indicator */}
          {properties.running && (
            <Group y={15}>
              <Rect
                x={5}
                y={0}
                width={width - 10}
                height={6}
                fill="#e5e7eb"
                cornerRadius={3}
              />
              <Rect
                x={5}
                y={0}
                width={(width - 10) * (properties.speed / 100)}
                height={6}
                fill="#3b82f6"
                cornerRadius={3}
              />
              <Text
                x={0}
                y={8}
                width={width}
                text={`${properties.speed}% Speed`}
                fontSize={10}
                fill="#6b7280"
                align="center"
              />
            </Group>
          )}
          
          {/* Metrics display */}
          {properties.running && (
            <Group y={properties.speed ? 35 : 20}>
              {properties.flowRate !== undefined && (
                <Text
                  x={0}
                  y={0}
                  width={width}
                  text={`${properties.flowRate} GPM`}
                  fontSize={10}
                  fill="#374151"
                  align="center"
                />
              )}
              
              {properties.pressure !== undefined && (
                <Text
                  x={0}
                  y={12}
                  width={width}
                  text={`${properties.pressure} PSI`}
                  fontSize={10}
                  fill="#374151"
                  align="center"
                />
              )}
              
              {properties.temperature !== undefined && (
                <Text
                  x={0}
                  y={24}
                  width={width}
                  text={`${properties.temperature}°F`}
                  fontSize={10}
                  fill={getTemperatureColor(properties.temperature)}
                  align="center"
                />
              )}
              
              {properties.efficiency !== undefined && (
                <Group y={36}>
                  <Text
                    x={0}
                    y={0}
                    width={width / 2}
                    text="Eff:"
                    fontSize={9}
                    fill="#6b7280"
                    align="right"
                  />
                  <Text
                    x={width / 2 + 2}
                    y={0}
                    width={width / 2 - 2}
                    text={`${properties.efficiency}%`}
                    fontSize={9}
                    fill={getEfficiencyColor(properties.efficiency)}
                    align="left"
                    fontStyle="bold"
                  />
                </Group>
              )}
            </Group>
          )}
        </Group>
      )}
      
      {/* Fault indicator */}
      {properties.fault && (
        <Group>
          <Circle
            x={centerX + radius * 0.7}
            y={centerY - radius * 0.7}
            radius={12}
            fill="#ef4444"
            stroke="#dc2626"
            strokeWidth={2}
          />
          <Text
            x={centerX + radius * 0.7 - 6}
            y={centerY - radius * 0.7 - 8}
            text="!"
            fontSize={16}
            fill="#ffffff"
            fontStyle="bold"
            align="center"
          />
        </Group>
      )}
      
      {/* Efficiency arc indicator */}
      {properties.efficiency !== undefined && properties.running && (
        <Arc
          x={centerX}
          y={centerY}
          innerRadius={radius + 12}
          outerRadius={radius + 16}
          angle={270 * (properties.efficiency / 100)}
          rotation={-135}
          fill={getEfficiencyColor(properties.efficiency)}
          opacity={0.6}
        />
      )}
      
      {/* Connection flanges */}
      <Group>
        {/* Inlet flange */}
        <Rect
          x={-5}
          y={centerY - 10}
          width={10}
          height={20}
          fill="#6b7280"
          stroke="#374151"
          strokeWidth={1}
          cornerRadius={2}
        />
        
        {/* Outlet flange */}
        <Rect
          x={width - 5}
          y={centerY - 10}
          width={10}
          height={20}
          fill="#6b7280"
          stroke="#374151"
          strokeWidth={1}
          cornerRadius={2}
        />
      </Group>
    </Group>
  )
}
