// src/components/hmi/components/HeatExchangerComponent.tsx

import { Group, Rect, Line, Circle, Text } from 'react-konva'

interface HeatExchangerProps {
  x: number
  y: number
  width: number
  height: number
  properties: {
    hotInletTemp: number
    hotOutletTemp: number
    coldInletTemp: number
    coldOutletTemp: number
    efficiency: number
    showTemperatures: boolean
  }
  style: any
  [key: string]: any
}

export default function HeatExchangerComponent({
  x, y, width, height, properties, style, ...rest
}: HeatExchangerProps) {
  const plateCount = 5
  const plateSpacing = width / (plateCount + 1)
  
  // Calculate color based on temperature
  const getTemperatureColor = (temp: number) => {
    const normalized = Math.min(Math.max((temp - 20) / 80, 0), 1)
    const r = Math.round(255 * normalized)
    const b = Math.round(255 * (1 - normalized))
    return `rgb(${r}, 100, ${b})`
  }

  return (
    <Group x={x} y={y} {...rest}>
      {/* Main body */}
      <Rect
        x={0}
        y={0}
        width={width}
        height={height}
        fill={style.fill || '#e5e7eb'}
        stroke={style.stroke || '#333333'}
        strokeWidth={style.strokeWidth || 2}
        cornerRadius={5}
      />

      {/* Heat exchanger plates */}
      {Array.from({ length: plateCount }).map((_, i) => {
        const plateX = plateSpacing * (i + 1) - 5
        const gradient = i / plateCount
        const plateTemp = properties.hotInletTemp * (1 - gradient) + properties.coldOutletTemp * gradient
        
        return (
          <Group key={i}>
            <Rect
              x={plateX}
              y={10}
              width={10}
              height={height - 20}
              fill={getTemperatureColor(plateTemp)}
              stroke="#666666"
              strokeWidth={1}
            />
            {/* Chevron pattern for plates */}
            {[0, 1, 2].map((j) => (
              <Line
                key={j}
                points={[
                  plateX, 20 + j * 20,
                  plateX + 5, 25 + j * 20,
                  plateX + 10, 20 + j * 20
                ]}
                stroke="#333333"
                strokeWidth={1}
              />
            ))}
          </Group>
        )
      })}

      {/* Hot side inlet/outlet */}
      <Line
        points={[0, 20, -20, 20]}
        stroke="#ff4444"
        strokeWidth={6}
      />
      <Line
        points={[width, 20, width + 20, 20]}
        stroke="#ff8888"
        strokeWidth={6}
      />

      {/* Cold side inlet/outlet */}
      <Line
        points={[0, height - 20, -20, height - 20]}
        stroke="#4444ff"
        strokeWidth={6}
      />
      <Line
        points={[width, height - 20, width + 20, height - 20]}
        stroke="#8888ff"
        strokeWidth={6}
      />

      {/* Temperature labels */}
      {properties.showTemperatures && (
        <Group>
          <Text
            x={-25}
            y={5}
            text={`${properties.hotInletTemp}°C`}
            fontSize={10}
            fill="#ff4444"
            align="right"
            width={20}
          />
          <Text
            x={width + 25}
            y={5}
            text={`${properties.hotOutletTemp}°C`}
            fontSize={10}
            fill="#ff8888"
          />
          <Text
            x={-25}
            y={height - 25}
            text={`${properties.coldInletTemp}°C`}
            fontSize={10}
            fill="#4444ff"
            align="right"
            width={20}
          />
          <Text
            x={width + 25}
            y={height - 25}
            text={`${properties.coldOutletTemp}°C`}
            fontSize={10}
            fill="#8888ff"
          />
        </Group>
      )}

      {/* Efficiency indicator */}
      <Text
        x={0}
        y={height + 5}
        width={width}
        text={`η: ${properties.efficiency}%`}
        fontSize={12}
        fill="#333333"
        align="center"
        fontStyle="bold"
      />
    </Group>
  )
}
