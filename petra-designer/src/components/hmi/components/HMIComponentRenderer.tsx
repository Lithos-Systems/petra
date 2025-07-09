// @ts-nocheck
// src/components/hmi/components/HMIComponentRenderer.tsx

import { Group, Rect, Circle, Text, Transformer } from 'react-konva'
import { useEffect, useRef } from 'react'
import type {
  HMIComponent,
  TankProperties,
  PumpProperties,
  ValveProperties,
  GaugeProperties,
  TrendProperties,
  ButtonProperties
} from '@/types/hmi'
import TankComponent from './TankComponent'
import PumpComponent from './PumpComponent'
import ValveComponent from './ValveComponent'
import GaugeComponent from './GaugeComponent'
import TrendComponent from './TrendComponent'
import ButtonComponent from './ButtonComponent'
import HeatExchangerComponent from './HeatExchangerComponent'
import ConveyorComponent from './ConveyorComponent'
import MixerComponent from './MixerComponent'

interface HMIComponentRendererProps {
  component: HMIComponent
  isSelected: boolean
  onSelect: () => void
  onUpdate: (updates: Partial<HMIComponent>) => void
}

export function HMIComponentRenderer({
  component,
  isSelected,
  onSelect,
  onUpdate,
}: HMIComponentRendererProps) {
  const shapeRef = useRef<any>()
  const transformerRef = useRef<any>()

  useEffect(() => {
    if (isSelected && transformerRef.current && shapeRef.current) {
      transformerRef.current.nodes([shapeRef.current])
      transformerRef.current.getLayer()?.batchDraw()
    }
  }, [isSelected])

  const handleDragEnd = (e: any) => {
    onUpdate({
      position: {
        x: e.target.x(),
        y: e.target.y(),
      },
    })
  }

  const handleTransformEnd = () => {
    const node = shapeRef.current
    const scaleX = node.scaleX()
    const scaleY = node.scaleY()

    // Reset scale and update size
    node.scaleX(1)
    node.scaleY(1)

    onUpdate({
      position: {
        x: node.x(),
        y: node.y(),
      },
      size: {
        width: Math.max(5, node.width() * scaleX),
        height: Math.max(5, node.height() * scaleY),
      },
      rotation: node.rotation(),
    })
  }

  // Render component based on type
  const renderComponent = () => {
    const commonProps = {
      ref: shapeRef,
      id: component.id,
      x: component.position.x,
      y: component.position.y,
      width: component.size.width,
      height: component.size.height,
      rotation: component.rotation || 0,
      draggable: !component.locked,
      onDragEnd: handleDragEnd,
      onClick: onSelect,
      onTap: onSelect,
      isSelected,
    }

    switch (component.type) {
      case 'tank':
        return (
          <TankComponent
            {...commonProps}
            properties={component.properties as TankProperties}
            style={component.style}
          />
        )

      case 'pump':
        return (
          <PumpComponent
            {...commonProps}
            properties={component.properties as PumpProperties}
            style={component.style}
          />
        )

      case 'valve':
        return (
          <ValveComponent
            {...commonProps}
            properties={component.properties as ValveProperties}
            style={component.style}
          />
        )

      case 'gauge':
        return (
          <GaugeComponent
            {...commonProps}
            properties={component.properties as GaugeProperties}
            style={component.style}
          />
        )

      case 'trend':
        return (
          <TrendComponent
            {...commonProps}
            properties={component.properties as TrendProperties}
            style={component.style}
          />
        )

      case 'button':
        return (
          <ButtonComponent
            {...commonProps}
            properties={component.properties as ButtonProperties}
            style={component.style}
            interactions={component.interactions}
          />
        )

      case 'heat-exchanger':
        return (
          <HeatExchangerComponent
            {...commonProps}
            properties={component.properties as {
              hotInletTemp: number
              hotOutletTemp: number
              coldInletTemp: number
              coldOutletTemp: number
              efficiency: number
              showTemperatures: boolean
            }}
            style={component.style}
          />
        )

      case 'conveyor':
        return (
          <ConveyorComponent
            {...commonProps}
            properties={component.properties as {
              running: boolean
              speed: number
              direction: 'forward' | 'reverse'
              material: boolean
            }}
            style={component.style}
          />
        )

      case 'mixer':
        return (
          <MixerComponent
            {...commonProps}
            properties={component.properties as {
              running: boolean
              speed: number
              level: number
              agitatorType: 'anchor' | 'paddle' | 'turbine'
              temperature?: number
            }}
            style={component.style}
          />
        )

      case 'text':
        return (
          <Group {...commonProps}>
            <Text
              x={0}
              y={0}
              width={component.size.width}
              height={component.size.height}
              text={component.properties.text || 'Text'}
              fontSize={component.style.fontSize || 16}
              fontFamily={component.style.fontFamily || 'Arial'}
              fill={component.style.fill || '#000000'}
              align={component.properties.align || 'left'}
              verticalAlign="middle"
            />
          </Group>
        )

      case 'indicator':
        return (
          <Group {...commonProps}>
            <Circle
              x={component.size.width / 2}
              y={component.size.height / 2}
              radius={Math.min(component.size.width, component.size.height) / 2}
              fill={component.properties.on ? component.properties.onColor || '#00ff00' : component.properties.offColor || '#cccccc'}
              stroke={component.style.stroke || '#333333'}
              strokeWidth={component.style.strokeWidth || 2}
              shadowBlur={component.properties.on ? 10 : 0}
              shadowColor={component.properties.onColor || '#00ff00'}
            />
          </Group>
        )

      case 'motor':
        return (
          <Group {...commonProps}>
            {/* Motor body */}
            <Circle
              x={component.size.width / 2}
              y={component.size.height / 2}
              radius={Math.min(component.size.width, component.size.height) / 2}
              fill={component.properties.running ? '#10b981' : '#6b7280'}
              stroke={component.style.stroke || '#333333'}
              strokeWidth={component.style.strokeWidth || 2}
            />
            {/* M symbol */}
            <Text
              x={0}
              y={0}
              width={component.size.width}
              height={component.size.height}
              text="M"
              fontSize={component.size.height * 0.5}
              fontStyle="bold"
              fill="#ffffff"
              align="center"
              verticalAlign="middle"
            />
            {/* Rotation animation would go here */}
          </Group>
        )

      case 'pipe':
        return (
          <Group {...commonProps}>
            <Rect
              x={0}
              y={component.size.height / 2 - 10}
              width={component.size.width}
              height={20}
              fill={component.style.fill || '#666666'}
              stroke={component.style.stroke || '#333333'}
              strokeWidth={component.style.strokeWidth || 1}
            />
            {/* Flow animation could be added here */}
          </Group>
        )

      case 'shape':
        // Basic shape - rectangle by default
        return (
          <Group {...commonProps}>
            <Rect
              x={0}
              y={0}
              width={component.size.width}
              height={component.size.height}
              fill={component.style.fill || '#cccccc'}
              stroke={component.style.stroke || '#333333'}
              strokeWidth={component.style.strokeWidth || 1}
              cornerRadius={component.style.borderRadius || 0}
            />
          </Group>
        )

      default:
        // Fallback for unknown component types
        return (
          <Group {...commonProps}>
            <Rect
              x={0}
              y={0}
              width={component.size.width}
              height={component.size.height}
              fill="#ff0000"
              opacity={0.3}
              stroke="#ff0000"
              strokeWidth={2}
              dash={[5, 5]}
            />
            <Text
              x={0}
              y={0}
              width={component.size.width}
              height={component.size.height}
              text={`Unknown: ${component.type}`}
              fontSize={12}
              fill="#ff0000"
              align="center"
              verticalAlign="middle"
            />
          </Group>
        )
    }
  }

  return (
    <>
      {renderComponent()}
      {isSelected && (
        <Transformer
          ref={transformerRef}
          boundBoxFunc={(oldBox, newBox) => {
            // Limit resize
            if (newBox.width < 5 || newBox.height < 5) {
              return oldBox
            }
            return newBox
          }}
          onTransformEnd={handleTransformEnd}
        />
      )}
    </>
  )
}
