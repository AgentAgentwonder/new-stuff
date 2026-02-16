"use client"

import { useEffect, useRef, useState, useCallback } from "react"
import type { CandleData } from "@/lib/mock-data"
import { Maximize2, Minimize2 } from "lucide-react"

interface PriceChartProps {
  data: CandleData[]
  tokenSymbol?: string
  onTimeframeChange?: (tf: string) => void
}

const TIMEFRAMES = [
  { label: "100ms", value: "100ms" },
  { label: "1s", value: "1s" },
  { label: "5s", value: "5s" },
  { label: "30s", value: "30s" },
  { label: "1m", value: "1m" },
  { label: "5m", value: "5m" },
]

export default function PriceChart({ data, tokenSymbol = "TOKEN", onTimeframeChange }: PriceChartProps) {
  const chartContainerRef = useRef<HTMLDivElement>(null)
  const chartRef = useRef<any>(null)
  const seriesRef = useRef<any>(null)
  const [timeframe, setTimeframe] = useState("1s")
  const [mounted, setMounted] = useState(false)
  const [isFullscreen, setIsFullscreen] = useState(false)

  useEffect(() => {
    setMounted(true)
  }, [])

  const getChartHeight = useCallback(() => {
    return isFullscreen ? window.innerHeight - 60 : 420
  }, [isFullscreen])

  useEffect(() => {
    if (!mounted || !chartContainerRef.current) return

    let chart: any = null
    let ro: ResizeObserver | null = null

    const initChart = async () => {
      const { createChart, ColorType } = await import("lightweight-charts")
      if (!chartContainerRef.current) return

      chart = createChart(chartContainerRef.current, {
        layout: {
          background: { type: ColorType.Solid, color: "transparent" },
          textColor: "hsl(215, 15%, 55%)",
          fontSize: 11,
        },
        grid: {
          vertLines: { color: "rgba(255,255,255,0.04)" },
          horzLines: { color: "rgba(255,255,255,0.04)" },
        },
        crosshair: {
          mode: 0,
          vertLine: { color: "rgba(16,185,129,0.4)", width: 1, style: 2 },
          horzLine: { color: "rgba(16,185,129,0.4)", width: 1, style: 2 },
        },
        rightPriceScale: {
          borderColor: "rgba(255,255,255,0.06)",
          scaleMargins: { top: 0.05, bottom: 0.05 },
        },
        timeScale: {
          borderColor: "rgba(255,255,255,0.06)",
          timeVisible: true,
          secondsVisible: true,
        },
        width: chartContainerRef.current.clientWidth,
        height: getChartHeight(),
      })

      const series = chart.addCandlestickSeries({
        upColor: "#10b981",
        downColor: "#ef4444",
        borderUpColor: "#10b981",
        borderDownColor: "#ef4444",
        wickUpColor: "#10b981",
        wickDownColor: "#ef4444",
      })

      chartRef.current = chart
      seriesRef.current = series

      if (data.length > 0) {
        series.setData(data)
        chart.timeScale().fitContent()
      }

      ro = new ResizeObserver((entries) => {
        for (const entry of entries) {
          if (chart) {
            chart.applyOptions({
              width: entry.contentRect.width,
              height: getChartHeight(),
            })
          }
        }
      })

      ro.observe(chartContainerRef.current)
    }

    initChart()

    return () => {
      ro?.disconnect()
      chart?.remove()
      chartRef.current = null
      seriesRef.current = null
    }
  }, [mounted, isFullscreen, getChartHeight])

  // Update data when it changes
  useEffect(() => {
    if (seriesRef.current && data.length > 0) {
      seriesRef.current.setData(data)
    }
  }, [data])

  const handleTimeframeChange = (tf: string) => {
    setTimeframe(tf)
    onTimeframeChange?.(tf)
  }

  const toggleFullscreen = () => {
    setIsFullscreen((prev) => !prev)
  }

  if (!mounted) {
    return <div className="w-full h-[420px] bg-card rounded-lg animate-pulse" />
  }

  const containerClass = isFullscreen
    ? "fixed inset-0 z-50 bg-background p-4 flex flex-col"
    : "w-full"

  return (
    <div className={containerClass}>
      {/* Toolbar */}
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-1">
          {TIMEFRAMES.map((tf) => (
            <button
              key={tf.value}
              onClick={() => handleTimeframeChange(tf.value)}
              className={`px-2.5 py-1 text-[11px] font-medium rounded transition-colors ${
                timeframe === tf.value
                  ? "bg-emerald-500/20 text-emerald-400"
                  : "text-muted-foreground hover:text-foreground hover:bg-muted"
              }`}
            >
              {tf.label}
            </button>
          ))}
        </div>
        <div className="flex items-center gap-2">
          <span className="text-xs text-muted-foreground font-mono">{tokenSymbol}/SOL</span>
          <button
            onClick={toggleFullscreen}
            className="p-1.5 rounded text-muted-foreground hover:text-foreground hover:bg-muted transition-colors"
            aria-label={isFullscreen ? "Exit fullscreen" : "Enter fullscreen"}
          >
            {isFullscreen ? <Minimize2 className="h-3.5 w-3.5" /> : <Maximize2 className="h-3.5 w-3.5" />}
          </button>
        </div>
      </div>

      {/* Chart */}
      <div
        ref={chartContainerRef}
        className="w-full rounded-lg overflow-hidden flex-1"
        style={{ minWidth: 0, height: isFullscreen ? undefined : 420 }}
      />
    </div>
  )
}
