// @ts-nocheck
import { Group, Rect, Text } from 'react-konva'
import { useState, useRef } from 'react'

const ISA101Colors = {
  equipmentOutline: '#000000',
  buttonInactive: '#E6E6E6',
  buttonActive: '#00FF00',
  buttonPressed: '#00CC00',
  textPrimary: '#000000',
  textOnActive: '#FFFFFF',
  alarmColor: '#FF0000',
}

interface ISA101ButtonProps {
  x: number
  y: number
  width: number
  height: number
  properties: {
    text: string
    action: 'momentary' | 'toggle' | 'set'
    confirmRequired?: boolean
    confirmMessage?: string
    activeColor?: string
    inactiveColor?: string
    value?: boolean
  }
  style?: any
  selected?: boolean
  draggable?: boolean
  onDragEnd?: (e: any) => void
  onDragStart?: (e: any) => void
  onClick?: () => void
  onContextMenu?: (e: any) => void
}

export default function ISA101ButtonComponent({
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
}: ISA101ButtonProps) {
  const [isPressed, setIsPressed] = useState(false)
  const [isActive, setIsActive] = useState(properties.value || false)
  const groupRef = useRef<any>()

  const handleMouseDown = () => {
    if (properties.action === 'momentary') {
      setIsPressed(true)
      setIsActive(true)
    }
  }

  const handleMouseUp = () => {
    if (properties.action === 'momentary') {
      setIsPressed(false)
      setIsActive(false)
    } else if (properties.action === 'toggle') {
      if (properties.confirmRequired && !isActive) {
        if (window.confirm(properties.confirmMessage || 'Confirm action?')) {
          setIsActive(!isActive)
        }
      } else {
        setIsActive(!isActive)
      }
    }
    onClick?.()
  }

  const getButtonColor = () => {
    if (isPressed) return ISA101Colors.buttonPressed
    if (isActive) return properties.activeColor || ISA101Colors.buttonActive
    return properties.inactiveColor || ISA101Colors.buttonInactive
  }

  const getTextColor = () => {
    return isActive ? ISA101Colors.textOnActive : ISA101Colors.textPrimary
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
      onMouseDown={handleMouseDown}
      onMouseUp={handleMouseUp}
      onMouseLeave={() => {
        if (properties.action === 'momentary') {
          setIsPressed(false)
          setIsActive(false)
        }
      }}
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

      {/* Button body */}
      <Rect
        x={0}
        y={0}
        width={width}
        height={height}
        fill={getButtonColor()}
        stroke={ISA101Colors.equipmentOutline}
        strokeWidth={2}
        cornerRadius={4}
        shadowBlur={isPressed ? 0 : 2}
        shadowOffsetY={isPressed ? 0 : 2}
        shadowOpacity={0.3}
      />

      {/* Button text */}
      <Text
        x={0}
        y={0}
        width={width}
        height={height}
        text={properties.text}
        fontSize={14}
        fontStyle="bold"
        fill={getTextColor()}
        align="center"
        verticalAlign="middle"
      />

      {/* Confirm indicator */}
      {properties.confirmRequired && (
        <Group x={width - 15} y={5}>
          <Rect
            x={0}
            y={0}
            width={10}
            height={10}
            fill={ISA101Colors.alarmColor}
            cornerRadius={2}
          />
          <Text
            x={0}
            y={0}
            width={10}
            height={10}
            text="!"
            fontSize={8}
            fontStyle="bold"
            fill="#FFFFFF"
            align="center"
            verticalAlign="middle"
          />
        </Group>
      )}
    </Group>
  )
}
