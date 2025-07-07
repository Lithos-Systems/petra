// src/components/hmi/components/MixerComponent.tsx

import { useEffect, useRef } from 'react'
import { Group, Circle, Line, Path, Text, Rect } from 'react-konva'

interface MixerProps {
  x: number
  y: number
  width: number
  height: number
  properties: {
    running: boolean
    speed: number // RPM
    level: number // 0-100%
    agitatorType: 'paddle' | 'turbine' | 'anchor'
    temperature?: number
  }
  style: any
  [key: string]: any
}

export default function MixerComponent({
  x, y, width, height, properties, style, ...rest
}: MixerProps) {
  const rotationRef = useRef(0)
  const animationRef = useRef<any>()

  useEffect(() => {
    if (properties.running) {
      const animate = () => {
        rotationRef.current = (rotationRef.current + properties.speed / 10) % 360
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
    <Group x={x} y={y} {...rest}>
      {/* Tank body - cylindrical */}
      <Path
        data={`
          M ${10} ${height * 0.3}
          L ${10} ${height - 10}
          Q ${10} ${height} ${20} ${height}
          L ${width - 20} ${height}
          Q ${width - 10} ${height} ${width - 10} ${height - 10}
          L ${width - 10} ${height * 0.3}
        `}
        fill="transparent"
        stroke={style.stroke || '#333333'}
        strokeWidth={style.strokeWidth || 2}
      />

      {/* Tank top - elliptical */}
      <Ellipse
        x={centerX}
        y={height * 0.3}
        radiusX={tankRadius}
        radiusY={15}
        fill={style.fill || '#e5e7eb'}
        stroke={style.stroke || '#333333'}
        strokeWidth={style.strokeWidth || 2}
      />

      {/* Liquid with clipping */}
      <Group clipFunc={(ctx) => {
        ctx.beginPath()
        ctx.moveTo(15, height * 0.3)
        ctx.lineTo(15, height - 15)
        ctx.quadraticCurveTo(15, height - 5, 25, height - 5)
        ctx.lineTo(width - 25, height - 5)
        ctx.quadraticCurveTo(width - 15, height - 5, width - 15, height - 15)
        ctx.lineTo(width - 15, height * 0.3)
        ctx.closePath()
      }}>
        <Rect
          x={15}
          y={height - liquidHeight - 5}
          width={width - 30}
          height={liquidHeight}
          fill="#6495ed"
          opacity={0.7}
        />
        
        {/* Liquid surface waves when mixing */}
        {properties.running && (
          <Path
            data={`
              M ${15} ${height - liquidHeight - 5}
              Q ${width * 0.25} ${height - liquidHeight - 8 + Math.sin(rotationRef.current * 0.1) * 3}
                ${centerX} ${height - liquidHeight - 5 + Math.cos(rotationRef.current * 0.1) * 3}
              Q ${width * 0.75} ${height - liquidHeight - 8 - Math.sin(rotationRef.current * 0.1) * 3}
                ${width - 15} ${height - liquidHeight - 5}
            `}
            fill="#5483dc"
            opacity={0.3}
          />
        )}
      </Group>

      {/* Motor on top */}
      <Rect
        x={centerX - 20}
        y={5}
        width={40}
        height={25}
        fill={properties.running ? '#10b981' : '#6b7280'}
        stroke="#333333"
        strokeWidth={1}
        cornerRadius={3}
      />
      <Text
        x={centerX - 20}
        y={12}
        width={40}
        text="M"
        fontSize={14}
        fill="#ffffff"
        align="center"
        fontStyle="bold"
      />

      {/* Shaft */}
      <Line
        points={[centerX, 30, centerX, height - 20]}
        stroke="#666666"
        strokeWidth={4}
      />

      {/* Agitator (rotating part) */}
      <Group x={centerX} y={height - liquidHeight / 2 - 5} rotation={rotationRef.current}>
        {properties.agitatorType === 'paddle' && (
          <>
            <Rect
              x={-30}
              y={-5}
              width={60}
              height={10}
              fill="#999999"
              stroke="#666666"
              strokeWidth={1}
            />
            <Rect
              x={-5}
              y={-30}
              width={10}
              height={60}
              fill="#999999"
              stroke="#666666"
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
                stroke="#999999"
                strokeWidth={6}
                rotation={angle}
                lineCap="round"
              />
            ))}
            <Circle
              x={0}
              y={0}
              radius={8}
              fill="#666666"
            />
          </>
        )}
      </Group>

      {/* Speed indicator */}
      {properties.running && (
        <Text
          x={0}
          y={height + 5}
          width={width}
          text={`${properties.speed} RPM`}
          fontSize={11}
          fill="#666666"
          align="center"
        />
      )}

      {/* Temperature indicator */}
      {properties.temperature !== undefined && (
        <Group x={width - 40} y={height * 0.5}>
          <Rect
            x={0}
            y={0}
            width={30}
            height={20}
            fill="#ffffff"
            stroke="#333333"
            strokeWidth={1}
            cornerRadius={3}
          />
          <Text
            x={0}
            y={3}
            width={30}
            text={`${properties.temperature}°`}
            fontSize={10}
            fill="#333333"
            align="center"
          />
        </Group>
      )}
    </Group>
  )
}

// Helper component for Ellipse (Konva doesn't have it by default)
function Ellipse({ x, y, radiusX, radiusY, ...props }: any) {
  return (
    <Path
      data={`
        M ${x - radiusX} ${y}
        A ${radiusX} ${radiusY} 0 0 1 ${x + radiusX} ${y}
        A ${radiusX} ${radiusY} 0 0 1 ${x - radiusX} ${y}
        Z
      `}
      {...props}
    />
  )
}
