import { Group, Rect, Text } from 'react-konva'
import type { ButtonProperties, Interaction } from '@/types/hmi'

interface ButtonComponentProps {
  x: number
  y: number
  width: number
  height: number
  properties: ButtonProperties
  interactions: Interaction[]
  style: any
  [key: string]: any
}

export default function ButtonComponent({
  x, y, width, height, properties, interactions, style, ...rest
}: ButtonComponentProps) {
  const handleClick = () => {
    // Handle button click based on interactions
    console.log('Button clicked:', properties.action)
    
    // Process interactions
    interactions.forEach(interaction => {
      if (interaction.event === 'click') {
        // Handle the action
        switch (interaction.action.type) {
          case 'setSignal':
            console.log(`Set signal ${interaction.action.signal} to ${interaction.action.value}`)
            break
          case 'toggle':
            console.log(`Toggle signal ${interaction.action.signal}`)
            break
          default:
            console.log('Unknown action type:', interaction.action.type)
        }
      }
    })
  }
  
  return (
    <Group x={x} y={y} onClick={handleClick} {...rest}>
      {/* Button background */}
      <Rect
        x={0}
        y={0}
        width={width}
        height={height}
        fill={style.fill || '#3b82f6'}
        stroke={style.stroke || '#2563eb'}
        strokeWidth={style.strokeWidth || 1}
        cornerRadius={4}
        shadowBlur={2}
        shadowColor="#000000"
        shadowOpacity={0.2}
      />
      
      {/* Button text */}
      <Text
        x={0}
        y={0}
        width={width}
        height={height}
        text={properties.text || 'Button'}
        fontSize={style.fontSize || 14}
        fontFamily={style.fontFamily || 'Arial'}
        fill="#ffffff"
        align="center"
        verticalAlign="middle"
      />
    </Group>
  )
}
