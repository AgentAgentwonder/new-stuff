"use client"

import { useEffect, useRef, useCallback, useState } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Switch } from "@/components/ui/switch"
import { Label } from "@/components/ui/label"
import { Button } from "@/components/ui/button"
import PriceChart from "./price-chart"
import TradingInterface from "./trading-interface"
import MemecoinFeed from "./memecoin-feed"
import BotSettingsPanel from "./bot-settings"
import TradeHistoryPanel from "./trade-history"
import PortfolioView from "./portfolio"
import { ArrowUpRight, ArrowDownRight, BarChart3, Settings, History, Briefcase, Wallet, Loader2 } from "lucide-react"
import { useDashboardStore, type MemecoinToken as TokenType, type CandleData } from "@/store/dashboardStore"

function generateNewPrice(currentPrice: number) {
  const change = (Math.random() - 0.5) * 0.05 * currentPrice
  const newPrice = currentPrice + change
  const percentChange = (change / currentPrice) * 100
  return {
    newPrice: Math.max(newPrice, 0.00001),
    newChange: Math.abs(percentChange),
    direction: change >= 0 ? "up" as const : "down" as const
  }
}

function generateNewCandle(lastCandle: CandleData, currentPrice: number, intervalMinutes: number): CandleData {
  return {
    time: lastCandle.time + intervalMinutes * 60 * 1000,
    open: lastCandle.close,
    high: Math.max(lastCandle.close, currentPrice),
    low: Math.min(lastCandle.close, currentPrice),
    close: currentPrice,
    volume: Math.random() * 10000
  }
}

export default function Dashboard() {
  const {
    solPrice,
    priceChange,
    priceDirection,
    coins,
    selectedCoin,
    candleData,
    botSettings,
    trades,
    balance,
    holdings,
    activeTab,
    wallet,
    isLoading,
    error,
    initialize,
    setActiveTab,
    selectCoin,
    executeTrade,
    toggleBot,
    toggleSimulation,
    connectWallet,
    disconnectWallet,
    refreshMarketData,
  } = useDashboardStore()

  const coinPricesRef = useRef<Map<string, number>>(new Map())
  const [isUpdating, setIsUpdating] = useState(false)

  // Initialize on mount
  useEffect(() => {
    initialize()
  }, [initialize])

  // Live price updates
  useEffect(() => {
    const interval = setInterval(() => {
      // Update SOL price
      const { newPrice, newChange, direction } = generateNewPrice(solPrice)
      
      // Update all coin prices
      useDashboardStore.setState((state) => ({
        solPrice: newPrice,
        priceChange: newChange,
        priceDirection: direction,
        coins: state.coins.map((coin) => {
          const prevPrice = coinPricesRef.current.get(coin.id) || coin.price
          const volatility = (Math.random() - 0.5) * 0.08 * prevPrice
          const newPriceVal = Math.max(prevPrice + volatility, prevPrice * 0.01)
          coinPricesRef.current.set(coin.id, newPriceVal)
          const newAge = Math.floor((Date.now() - coin.launchedAt) / 1000)
          const newMC = newPriceVal * (coin.marketCap / coin.price)
          const whaleAlert = coin.whaleAlert || Math.random() > 0.998

          return {
            ...coin,
            price: newPriceVal,
            age: newAge,
            marketCap: newMC,
            isNew: newAge < 60,
            whaleAlert,
            change24h: coin.change24h + (Math.random() - 0.5) * 0.5,
          }
        })
      }))

      // Occasionally add a brand new coin
      if (Math.random() > 0.995) {
        const newNames = ["DOGE2", "KEKW", "LUNAR", "ASTRO", "PUMP", "YOLO", "CHAD", "FOMO"]
        const name = newNames[Math.floor(Math.random() * newNames.length)]
        const id = `${name}-${Date.now()}`
        const price = Math.random() * 0.001 + 0.00001
        const mc = price * (Math.random() * 100000 + 5000)

        useDashboardStore.setState((state) => ({
          coins: [
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
            ...state.coins
          ].slice(0, 50)
        }))
      }

      // Update selected coin chart
      if (selectedCoin) {
        const currentPrice = coinPricesRef.current.get(selectedCoin.id) || selectedCoin.price
        useDashboardStore.setState((state) => {
          if (state.candleData.length === 0) return state
          const lastCandle = state.candleData[state.candleData.length - 1]
          const updatedCandles = [...state.candleData]

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
          return { candleData: updatedCandles }
        })
      }

      // Auto trading when bot is running
      if (botSettings.isRunning && selectedCoin) {
        const currentPrice = coinPricesRef.current.get(selectedCoin.id) || selectedCoin.price
        const shouldBuy = priceDirection === "up" && priceChange > 0.5 && balance > 0.05
        const shouldSell = priceDirection === "down" && priceChange > 0.5

        if (shouldBuy && Math.random() > 0.95) {
          const amt = botSettings.tradeAmount
          executeTrade("buy", selectedCoin.symbol, amt)
        } else if (shouldSell && Math.random() > 0.95) {
          const amt = botSettings.tradeAmount * 0.6
          executeTrade("sell", selectedCoin.symbol, amt)
        }
      }
    }, 100)

    return () => clearInterval(interval)
  }, [solPrice, botSettings.isRunning, selectedCoin, balance, priceDirection, priceChange, executeTrade])

  const handleSelectCoin = useCallback((coin: TokenType) => {
    selectCoin(coin)
  }, [selectCoin])

  const handleTrade = useCallback(
    async (trade: { type: "buy" | "sell"; token: string; amount: number; price: number }) => {
      await executeTrade(trade.type, trade.token, trade.amount)
    },
    [executeTrade]
  )

  const handleConnectWallet = async () => {
    // In a real implementation, this would connect to Phantom or another wallet
    // For now, simulate wallet connection
    const mockAddress = "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU"
    await connectWallet(mockAddress)
  }

  const portfolioValue = balance + holdings.reduce((sum, h) => sum + h.value, 0)

  return (
    <div className="w-full max-w-[1440px] mx-auto p-3 flex flex-col gap-3 v0-theme">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h1 className="text-lg font-bold text-foreground tracking-tight">Eclipse Market Pro</h1>
        <div className="flex items-center gap-4">
          {/* Wallet Connection */}
          {wallet.connected ? (
            <div className="flex items-center gap-2">
              <Wallet className="h-4 w-4 text-muted-foreground" />
              <span className="text-xs text-muted-foreground font-mono">
                {wallet.address.slice(0, 6)}...{wallet.address.slice(-4)}
              </span>
              <Button 
                variant="ghost" 
                size="sm" 
                onClick={disconnectWallet}
                className="text-xs"
              >
                Disconnect
              </Button>
            </div>
          ) : (
            <Button 
              variant="outline" 
              size="sm" 
              onClick={handleConnectWallet}
              className="text-xs"
            >
              <Wallet className="h-3.5 w-3.5 mr-1" />
              Connect Wallet
            </Button>
          )}
          
          {/* Simulation Toggle */}
          <div className="flex items-center gap-2">
            <Label htmlFor="sim-toggle" className="text-[11px] text-muted-foreground">Simulation</Label>
            <Switch id="sim-toggle" checked={botSettings.isSimulation} onCheckedChange={toggleSimulation} />
          </div>
          
          {/* Bot Status */}
          <div className="flex items-center gap-1.5">
            <span
              className={`h-2 w-2 rounded-full ${botSettings.isRunning ? "bg-emerald-500 shadow-[0_0_6px_rgba(16,185,129,0.6)]" : "bg-muted-foreground"}`}
            />
            <span className="text-[11px] text-muted-foreground">{botSettings.isRunning ? "Bot Active" : "Bot Idle"}</span>
          </div>
        </div>
      </div>

      {/* Loading/Error States */}
      {isLoading && (
        <div className="flex items-center justify-center p-4">
          <Loader2 className="h-6 w-6 animate-spin text-primary" />
          <span className="ml-2 text-sm text-muted-foreground">Loading market data...</span>
        </div>
      )}
      
      {error && (
        <div className="p-3 bg-destructive/10 border border-destructive rounded-lg">
          <span className="text-sm text-destructive">{error}</span>
          <Button variant="ghost" size="sm" onClick={refreshMarketData} className="ml-2">
            Retry
          </Button>
        </div>
      )}

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
                  botSettings.isRunning ? "bg-emerald-500 shadow-[0_0_6px_rgba(16,185,129,0.5)]" : "bg-red-500"
                }`}
              />
              <span className="text-sm font-medium text-foreground">{botSettings.isRunning ? "Running" : "Stopped"}</span>
            </div>
            <div className="text-[11px] text-muted-foreground mt-0.5">
              {trades.length} trades | {botSettings.isSimulation ? "SIM" : "LIVE"}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Tabs */}
      <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as any)}>
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
                <MemecoinFeed 
                  coins={coins} 
                  selectedId={selectedCoin?.id ?? null} 
                  onSelect={handleSelectCoin} 
                />
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
          <PortfolioView holdings={holdings} solBalance={balance} />
        </TabsContent>

        {/* Settings Tab */}
        <TabsContent value="settings" className="mt-3">
          <BotSettingsPanel
            isRunning={botSettings.isRunning}
            setIsRunning={toggleBot}
            isSimulation={botSettings.isSimulation}
            setIsSimulation={toggleSimulation}
          />
        </TabsContent>

        {/* History Tab */}
        <TabsContent value="history" className="mt-3">
          <TradeHistoryPanel trades={trades} />
        </TabsContent>
      </Tabs>
    </div>
  )
}