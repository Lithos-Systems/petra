import React, { useState, useEffect, useRef } from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';

interface TrendPoint {
  timestamp: number;
  value: number;
  quality?: 'good' | 'bad' | 'uncertain';
}

interface TrendSeries {
  name: string;
  color: string;
  data: TrendPoint[];
  unit?: string;
  min?: number;
  max?: number;
}

interface ISA101TrendProps {
  series: TrendSeries[];
  timeRange?: number; // seconds to display
  updateInterval?: number; // milliseconds
  height?: number;
  showGrid?: boolean;
  showLegend?: boolean;
  title?: string;
}

export default function ISA101Trend({
  series,
  timeRange = 300, // 5 minutes default
  updateInterval = 1000,
  height = 300,
  showGrid = true,
  showLegend = true,
  title
}: ISA101TrendProps) {
  const [chartData, setChartData] = useState<any[]>([]);
  const [timeWindow, setTimeWindow] = useState({ start: Date.now() - timeRange * 1000, end: Date.now() });

  useEffect(() => {
    // Convert series data to chart format
    const now = Date.now();
    const startTime = now - timeRange * 1000;
    
    // Create time points
    const timePoints: number[] = [];
    for (let t = startTime; t <= now; t += updateInterval) {
      timePoints.push(t);
    }

    // Build chart data
    const data = timePoints.map(timestamp => {
      const point: any = {
        time: new Date(timestamp).toLocaleTimeString(),
        timestamp
      };

      series.forEach(s => {
        const dataPoint = s.data.find(d => 
          Math.abs(d.timestamp - timestamp) < updateInterval / 2
        );
        point[s.name] = dataPoint?.value ?? null;
      });

      return point;
    });

    setChartData(data);
  }, [series, timeRange, updateInterval]);

  // Format Y-axis values
  const formatYAxis = (value: number) => {
    if (Math.abs(value) >= 1000) {
      return `${(value / 1000).toFixed(1)}k`;
    }
    return value.toFixed(1);
  };

  // Custom tooltip
  const CustomTooltip = ({ active, payload, label }: any) => {
    if (active && payload && payload.length) {
      return (
        <div className="isa101-panel p-2 text-xs">
          <p className="font-medium">{label}</p>
          {payload.map((entry: any, index: number) => (
            <p key={index} style={{ color: entry.color }}>
              {entry.name}: {entry.value?.toFixed(2)} {series.find(s => s.name === entry.name)?.unit || ''}
            </p>
          ))}
        </div>
      );
    }
    return null;
  };

  return (
    <div className="isa101-trend" style={{ height: `${height}px` }}>
      {title && (
        <div className="isa101-panel-header text-sm">
          {title}
        </div>
      )}
      <ResponsiveContainer width="100%" height={title ? height - 30 : height}>
        <LineChart
          data={chartData}
          margin={{ top: 5, right: 5, left: 5, bottom: 5 }}
        >
          {showGrid && (
            <CartesianGrid 
              strokeDasharray="3 3" 
              stroke="#E0E0E0"
              className="isa101-trend-grid"
            />
          )}
          <XAxis 
            dataKey="time"
            stroke={ISA_COLORS.text}
            tick={{ fontSize: 10 }}
            interval="preserveStartEnd"
          />
          <YAxis 
            stroke={ISA_COLORS.text}
            tick={{ fontSize: 10 }}
            tickFormatter={formatYAxis}
          />
          <Tooltip content={<CustomTooltip />} />
          {showLegend && (
            <Legend 
              wrapperStyle={{ fontSize: '12px' }}
              iconType="line"
            />
          )}
          {series.map((s, index) => (
            <Line
              key={s.name}
              type="monotone"
              dataKey={s.name}
              stroke={s.color || ISA_COLORS[`trend-${index + 1}`] || '#000080'}
              strokeWidth={2}
              dot={false}
              isAnimationActive={false}
            />
          ))}
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}

// ISA-101 colors for trends
const ISA_COLORS = {
  text: '#000000',
  'trend-1': '#000080', // Dark blue
  'trend-2': '#008000', // Dark green
  'trend-3': '#800000', // Dark red
  'trend-4': '#800080', // Dark purple
  'trend-5': '#008080', // Dark cyan
};

// Example component with live data
export function TrendExample() {
  const [series, setSeries] = useState<TrendSeries[]>([
    {
      name: 'Tank Level',
      color: '#000080',
      unit: 'ft',
      data: [],
      min: 0,
      max: 25
    },
    {
      name: 'Flow Rate',
      color: '#008000',
      unit: 'gpm',
      data: [],
      min: 0,
      max: 3000
    }
  ]);

  // Simulate live data
  useEffect(() => {
    const interval = setInterval(() => {
      const now = Date.now();
      setSeries(prevSeries => 
        prevSeries.map(s => ({
          ...s,
          data: [
            ...s.data.slice(-300), // Keep last 300 points
            {
              timestamp: now,
              value: s.name === 'Tank Level' 
                ? 12.5 + Math.sin(now / 10000) * 2 + Math.random() * 0.5
                : 2000 + Math.sin(now / 8000) * 500 + Math.random() * 100,
              quality: 'good'
            }
          ]
        }))
      );
    }, 1000);

    return () => clearInterval(interval);
  }, []);

  return (
    <div className="p-4 space-y-4">
      <ISA101Trend
        series={series}
        title="Process Trends - Tank System"
        height={400}
        timeRange={300}
      />
      
      <div className="grid grid-cols-2 gap-4">
        <ISA101Trend
          series={[series[0]]}
          title="Tank Level Trend"
          height={200}
          showLegend={false}
          timeRange={60}
        />
        <ISA101Trend
          series={[series[1]]}
          title="Flow Rate Trend"
          height={200}
          showLegend={false}
          timeRange={60}
        />
      </div>
    </div>
  );
}
