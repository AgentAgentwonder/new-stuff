"use client"

import { useMemo } from "react"
import { LineChart, Line, XAxis, YAxis, CartesianGrid, ResponsiveContainer, Area, AreaChart } from "recharts"
import { type CandleData } from "@/store/dashboardStore"

interface PriceChartProps {
  data: CandleData[]
  tokenSymbol: string
}

export default function PriceChart({ data, tokenSymbol }: PriceChartProps) {
  const chartData = useMemo(() => {
    return data.map((candle) => ({
      time: new Date(candle.time).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
      price: candle.close,
      high: candle.high,
      low: candle.low,
      volume: candle.volume,
    }))
  }, [data])

  const priceChange = useMemo(() => {
    if (chartData.length < 2) return 0
    const first = chartData[0]?.price || 0
    const last = chartData[chartData.length - 1]?.price || 0
    return ((last - first) / first) * 100
  }, [chartData])

  const isPositive = priceChange >= 0

  const formatPrice = (value: number) => {
    if (value >= 0.001) return `$${value.toFixed(4)}`
    if (value >= 0.0001) return `$${value.toFixed(6)}`
    return `$${value.toFixed(8)}`
  }

  const CustomTooltip = ({ active, payload, label }: any) => {
    if (active && payload && payload.length) {
      const data = payload[0].payload
      return (
        <div className="bg-card border border-border rounded-lg p-3 shadow-lg">
          <p className="text-sm text-muted-foreground mb-1">{label}</p>
          <p className="text-sm font-semibold text-foreground">
            Price: {formatPrice(data.price)}
          </p>
          <p className="text-xs text-muted-foreground">
            High: {formatPrice(data.high)}
          </p>
          <p className="text-xs text-muted-foreground">
            Low: {formatPrice(data.low)}
          </p>
          <p className="text-xs text-muted-foreground">
            Volume: {data.volume.toFixed(0)}
          </p>
        </div>
      )
    }
    return null
  }

  if (chartData.length === 0) {
    return (
      <div className="flex items-center justify-center h-[420px] text-sm text-muted-foreground">
        No chart data available
      </div>
    )
  }

  return (
    <div className="h-full flex flex-col">
      {/* Chart Header */}
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-full bg-gradient-to-br from-primary to-primary/60 flex items-center justify-center">
            <span className="text-xs font-bold text-primary-foreground">
              {tokenSymbol.charAt(0)}
            </span>
          </div>
          <div>
            <h3 className="font-semibold text-foreground">{tokenSymbol}</h3>
            <p className="text-sm text-muted-foreground">{formatPrice(chartData[chartData.length - 1]?.price || 0)}</p>
          </div>
        </div>
        <div className={`flex items-center gap-1 text-sm font-medium ${
          isPositive ? 'text-emerald-400' : 'text-red-400'
        }`}>
          {isPositive ? '↗' : '↘'} 
          {priceChange.toFixed(2)}%
        </div>
      </div>

      {/* Chart Container */}
      <div className="flex-1 min-h-0">
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart data={chartData} margin={{ top: 5, right: 30, left: 20, bottom: 5 }}>
            <defs>
              <linearGradient id={`priceGradient`} x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor={isPositive ? "#10B981" : "#EF4444"} stopOpacity={0.3} />
                <stop offset="95%" stopColor={isPositive ? "#10B981" : "#EF4444"} stopOpacity={0} />
              </linearGradient>
            </defs>
            <CartesianGrid strokeDasharray="3 3" stroke="hsl(225 20% 18%)" />
            <XAxis 
              dataKey="time" 
              axisLine={false}
              tickLine={false}
              tick={{ fontSize: 11, fill: 'hsl(215 15% 55%)' }}
            />
            <YAxis 
              domain={['dataMin * 0.95', 'dataMax * 1.05']}
              axisLine={false}
              tickLine={false}
              tick={{ fontSize: 11, fill: 'hsl(215 15% 55%)' }}
              tickFormatter={formatPrice}
            />
            <Area
              type="monotone"
              dataKey="price"
              stroke={isPositive ? "#10B981" : "#EF4444"}
              strokeWidth={2}
              fill={`url(#priceGradient)`}
              dot={false}
              activeDot={{ r: 4, fill: isPositive ? "#10B981" : "#EF4444" }}
            />
            <Line 
              type="monotone" 
              dataKey="high" 
              stroke="hsl(160 84% 39%)" 
              strokeWidth={1}
              strokeDasharray="2 2"
              dot={false}
              opacity={0.3}
            />
            <Line 
              type="monotone" 
              dataKey="low" 
              stroke="hsl(160 84% 39%)" 
              strokeWidth={1}
              strokeDasharray="2 2"
              dot={false}
              opacity={0.3}
            />
          </AreaChart>
        </ResponsiveContainer>
      </div>

      {/* Chart Stats */}
      <div className="grid grid-cols-4 gap-4 mt-4 pt-4 border-t border-border">
        <div>
          <div className="text-[11px] text-muted-foreground">24h High</div>
          <div className="text-sm font-medium text-foreground">
            {formatPrice(Math.max(...data.map(d => d.high)))}
          </div>
        </div>
        <div>
          <div className="text-[11px] text-muted-foreground">24h Low</div>
          <div className="text-sm font-medium text-foreground">
            {formatPrice(Math.min(...data.map(d => d.low)))}
          </div>
        </div>
        <div>
          <div className="text-[11px] text-muted-foreground">24h Volume</div>
          <div className="text-sm font-medium text-foreground">
            {data.reduce((sum, d) => sum + d.volume, 0).toFixed(0)}
          </div>
        </div>
        <div>
          <div className="text-[11px] text-muted-foreground">Timeframe</div>
          <div className="text-sm font-medium text-foreground">1m</div>
        </div>
      </div>
    </div>
  )
}