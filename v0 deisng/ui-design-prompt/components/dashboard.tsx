"use client"

import { useState, useEffect, useRef, useCallback } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Switch } from "@/components/ui/switch"
import { Label } from "@/components/ui/label"
import PriceChart from "@/components/price-chart"
import TradingInterface from "@/components/trading-interface"
import MemecoinFeed from "@/components/memecoin-feed"
import BotSettings from "@/components/bot-settings"
import TradeHistory from "@/components/trade-history"
import Portfolio from "@/components/portfolio"
import { ArrowUpRight, ArrowDownRight, BarChart3, Settings, History, Briefcase } from "lucide-react"
import {
  generateCandleData,
  generateNewPrice,
  generateNewCandle,
  generateMemecoins,
  sampleTrades,
  sampleHoldings,
  type CandleData,
  type MemecoinToken,
  type Trade,
} from "@/lib/mock-data"

export default function Dashboard() {
  const [activeTab, setActiveTab] = useState("dashboard")
  const [solPrice, setSolPrice] = useState(148.65)
  const [priceChange, setPriceChange] = useState(3.21)
  const [priceDirection, setPriceDirection] = useState<"up" | "down">("up")

  const [coins, setCoins] = useState<MemecoinToken[]>([])
  const [selectedCoin, setSelectedCoin] = useState<MemecoinToken | null>(null)
  const [candleData, setCandleData] = useState<CandleData[]>([])

  const [isRunning, setIsRunning] = useState(false)
  const [isSimulation, setIsSimulation] = useState(true)
  const [trades, setTrades] = useState<Trade[]>(sampleTrades)
  const [balance, setBalance] = useState(0.5)
  const [holdings, setHoldings] = useState(sampleHoldings)

  const coinPricesRef = useRef<Map<string, number>>(new Map())

  // Initialize
  useEffect(() => {
    const initial = generateMemecoins()
    setCoins(initial)
    initial.forEach((c) => coinPricesRef.current.set(c.id, c.price))
  }, [])

  // When a coin is selected, generate chart data for it
  useEffect(() => {
    if (!selectedCoin) return
    const data = generateCandleData(selectedCoin.price, 60, 1)
    setCandleData(data)
  }, [selectedCoin?.id])

  // Live updates every 100ms
  useEffect(() => {
    const interval = setInterval(() => {
      // Update SOL price
      const { newPrice: newSolPrice, newChange, direction } = generateNewPrice(solPrice)
      setSolPrice(newSolPrice)
      setPriceChange(newChange)
      setPriceDirection(direction)

      // Update all coin prices and ages
      setCoins((prev) =>
        prev.map((coin) => {
          const prevPrice = coinPricesRef.current.get(coin.id) || coin.price
          const volatility = (Math.random() - 0.5) * 0.08 * prevPrice
          const newPrice = Math.max(prevPrice + volatility, prevPrice * 0.01)
          coinPricesRef.current.set(coin.id, newPrice)
          const newAge = Math.floor((Date.now() - coin.launchedAt) / 1000)
          const newMC = newPrice * (coin.marketCap / coin.price)
          const whaleAlert = coin.whaleAlert || Math.random() > 0.998

          return {
            ...coin,
            price: newPrice,
            age: newAge,
            marketCap: newMC,
            isNew: newAge < 60,
            whaleAlert,
            change24h: coin.change24h + (Math.random() - 0.5) * 0.5,
          }
        })
      )

      // Occasionally add a brand new coin
      if (Math.random() > 0.995) {
        const newNames = ["DOGE2", "KEKW", "LUNAR", "ASTRO", "PUMP", "YOLO", "CHAD", "FOMO"]
        const name = newNames[Math.floor(Math.random() * newNames.length)]
        const id = `${name}-${Date.now()}`
        const price = Math.random() * 0.001 + 0.00001
        const mc = price * (Math.random() * 100000 + 5000)

        setCoins((prev) => [
          {
            id,
            symbol: name,
            name: `${name} Token`,
            price,
            marketCap: mc,
            liquidity: mc * 0.05,
            holders: Math.floor(Math.random() * 30) + 5,
            buyTax: Math.floor(Math.random() * 3),
            sellTax: Math.floor(Math.random() * 5),
            riskLevel: "high" as const,
            age: 0,
            launchedAt: Date.now(),
            change24h: 0,
            isNew: true,
            whaleAlert: false,
            volume24h: 0,
          },
          ...prev,
        ])
      }

      // Update selected coin chart
      if (selectedCoin) {
        const currentPrice = coinPricesRef.current.get(selectedCoin.id) || selectedCoin.price
        setCandleData((prev) => {
          if (prev.length === 0) return prev
          const lastCandle = prev[prev.length - 1]
          const updatedCandles = [...prev]

          if (Math.random() > 0.85) {
            const newCandle = generateNewCandle(lastCandle, currentPrice, 1)
            updatedCandles.push(newCandle)
            if (updatedCandles.length > 120) updatedCandles.shift()
          } else {
            updatedCandles[updatedCandles.length - 1] = {
              ...lastCandle,
              close: Number(currentPrice.toFixed(6)),
              high: Math.max(lastCandle.high, Number(currentPrice.toFixed(6))),
              low: Math.min(lastCandle.low, Number(currentPrice.toFixed(6))),
            }
          }
          return updatedCandles
        })
      }

      // Auto trading when bot is running
      if (isRunning && selectedCoin) {
        const cp = coinPricesRef.current.get(selectedCoin.id) || selectedCoin.price
        const shouldBuy = direction === "up" && newChange > 0.5 && balance > 0.05
        const shouldSell = direction === "down" && newChange > 0.5

        if (shouldBuy && Math.random() > 0.95) {
          const amt = 0.05
          setBalance((prev) => prev - amt)
          setTrades((prev) => [
            ...prev,
            {
              id: Date.now(),
              type: "buy",
              token: selectedCoin.symbol,
              amount: amt,
              price: cp,
              total: amt,
              time: new Date().toISOString(),
              status: "filled",
            },
          ])
        } else if (shouldSell && Math.random() > 0.95) {
          const amt = 0.03
          setBalance((prev) => prev + amt)
          setTrades((prev) => [
            ...prev,
            {
              id: Date.now(),
              type: "sell",
              token: selectedCoin.symbol,
              amount: amt,
              price: cp,
              total: amt,
              time: new Date().toISOString(),
              status: "filled",
            },
          ])
        }
      }
    }, 100)

    return () => clearInterval(interval)
  }, [solPrice, isRunning, selectedCoin, balance])

  const handleSelectCoin = useCallback((coin: MemecoinToken) => {
    setSelectedCoin(coin)
  }, [])

  const handleTrade = useCallback(
    (trade: { type: "buy" | "sell"; token: string; amount: number; price: number }) => {
      if (trade.type === "buy") {
        setBalance((prev) => prev - trade.amount)
      } else {
        setBalance((prev) => prev + trade.amount)
      }
      setTrades((prev) => [
        ...prev,
        {
          id: Date.now(),
          type: trade.type,
          token: trade.token,
          amount: trade.amount,
          price: trade.price,
          total: trade.amount,
          time: new Date().toISOString(),
          status: "filled",
        },
      ])
    },
    []
  )

  const portfolioValue = balance + holdings.reduce((sum, h) => sum + h.value, 0)

  return (
    <div className="w-full max-w-[1440px] mx-auto p-3 flex flex-col gap-3">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h1 className="text-lg font-bold text-foreground tracking-tight">Eclipse Market Pro</h1>
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-2">
            <Label htmlFor="sim-toggle" className="text-[11px] text-muted-foreground">Simulation</Label>
            <Switch id="sim-toggle" checked={isSimulation} onCheckedChange={setIsSimulation} />
          </div>
          <div className="flex items-center gap-1.5">
            <span
              className={`h-2 w-2 rounded-full ${isRunning ? "bg-emerald-500 shadow-[0_0_6px_rgba(16,185,129,0.6)]" : "bg-muted-foreground"}`}
            />
            <span className="text-[11px] text-muted-foreground">{isRunning ? "Bot Active" : "Bot Idle"}</span>
          </div>
        </div>
      </div>

      {/* Summary Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-2.5">
        <Card>
          <CardHeader className="pb-1.5 pt-3 px-4">
            <CardTitle className="text-[10px] font-medium text-muted-foreground uppercase tracking-wider">
              SOL Price
            </CardTitle>
          </CardHeader>
          <CardContent className="pb-3 px-4">
            <div className="flex items-center gap-2">
              <span className="text-lg font-bold font-mono text-foreground">${solPrice.toFixed(2)}</span>
              <span
                className={`flex items-center text-xs font-semibold ${
                  priceDirection === "up" ? "text-emerald-400" : "text-red-400"
                }`}
              >
                {priceDirection === "up" ? (
                  <ArrowUpRight className="h-3.5 w-3.5" />
                ) : (
                  <ArrowDownRight className="h-3.5 w-3.5" />
                )}
                {priceChange.toFixed(2)}%
              </span>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-1.5 pt-3 px-4">
            <CardTitle className="text-[10px] font-medium text-muted-foreground uppercase tracking-wider">
              Portfolio Value
            </CardTitle>
          </CardHeader>
          <CardContent className="pb-3 px-4">
            <div className="text-lg font-bold font-mono text-foreground">{portfolioValue.toFixed(4)} SOL</div>
            <div className="text-[11px] text-muted-foreground mt-0.5">
              ~${(portfolioValue * solPrice).toLocaleString(undefined, { maximumFractionDigits: 2 })}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-1.5 pt-3 px-4">
            <CardTitle className="text-[10px] font-medium text-muted-foreground uppercase tracking-wider">
              Bot Status
            </CardTitle>
          </CardHeader>
          <CardContent className="pb-3 px-4">
            <div className="flex items-center gap-2">
              <div
                className={`h-2.5 w-2.5 rounded-full ${
                  isRunning ? "bg-emerald-500 shadow-[0_0_6px_rgba(16,185,129,0.5)]" : "bg-red-500"
                }`}
              />
              <span className="text-sm font-medium text-foreground">{isRunning ? "Running" : "Stopped"}</span>
            </div>
            <div className="text-[11px] text-muted-foreground mt-0.5">
              {trades.length} trades | {isSimulation ? "SIM" : "LIVE"}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="grid grid-cols-4 w-full max-w-lg bg-muted">
          <TabsTrigger value="dashboard" className="flex items-center gap-1.5 text-xs">
            <BarChart3 className="h-3.5 w-3.5" />
            Dashboard
          </TabsTrigger>
          <TabsTrigger value="portfolio" className="flex items-center gap-1.5 text-xs">
            <Briefcase className="h-3.5 w-3.5" />
            Portfolio
          </TabsTrigger>
          <TabsTrigger value="settings" className="flex items-center gap-1.5 text-xs">
            <Settings className="h-3.5 w-3.5" />
            Settings
          </TabsTrigger>
          <TabsTrigger value="history" className="flex items-center gap-1.5 text-xs">
            <History className="h-3.5 w-3.5" />
            History
          </TabsTrigger>
        </TabsList>

        {/* Dashboard Tab */}
        <TabsContent value="dashboard" className="mt-3">
          <div className="grid grid-cols-1 lg:grid-cols-[340px_1fr] gap-3" style={{ minHeight: 580 }}>
            {/* Left: Memecoin Feed */}
            <Card className="overflow-hidden">
              <CardContent className="p-2 h-[580px]">
                <MemecoinFeed coins={coins} selectedId={selectedCoin?.id ?? null} onSelect={handleSelectCoin} />
              </CardContent>
            </Card>

            {/* Right: Chart + Trade Panel */}
            <div className="flex flex-col gap-3">
              {/* Chart */}
              <Card className="flex-1">
                <CardContent className="p-3">
                  {selectedCoin ? (
                    <PriceChart
                      data={candleData}
                      tokenSymbol={selectedCoin.symbol}
                    />
                  ) : (
                    <div className="flex items-center justify-center h-[420px] text-sm text-muted-foreground">
                      Select a token from the feed to view its chart
                    </div>
                  )}
                </CardContent>
              </Card>

              {/* Trade Panel */}
              <Card>
                <CardContent className="p-3">
                  <TradingInterface
                    token={selectedCoin}
                    balance={balance}
                    onTrade={handleTrade}
                  />
                </CardContent>
              </Card>
            </div>
          </div>
        </TabsContent>

        {/* Portfolio Tab */}
        <TabsContent value="portfolio" className="mt-3">
          <Portfolio holdings={holdings} solBalance={balance} />
        </TabsContent>

        {/* Settings Tab */}
        <TabsContent value="settings" className="mt-3">
          <BotSettings
            isRunning={isRunning}
            setIsRunning={setIsRunning}
            isSimulation={isSimulation}
            setIsSimulation={setIsSimulation}
          />
        </TabsContent>

        {/* History Tab */}
        <TabsContent value="history" className="mt-3">
          <TradeHistory trades={trades} />
        </TabsContent>
      </Tabs>
    </div>
  )
}
