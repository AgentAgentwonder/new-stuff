"use client"

import { useState } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { ArrowUpRight, ArrowDownRight, Zap, AlertTriangle, TrendingUp, TrendingDown } from "lucide-react"
import { type MemecoinToken } from "@/store/dashboardStore"

interface TradingInterfaceProps {
  token: MemecoinToken | null
  balance: number
  onTrade: (trade: { type: "buy" | "sell"; token: string; amount: number; price: number }) => Promise<void>
}

export default function TradingInterface({ token, balance, onTrade }: TradingInterfaceProps) {
  const [amount, setAmount] = useState("")
  const [isProcessing, setIsProcessing] = useState(false)
  const [activeTab, setActiveTab] = useState<"buy" | "sell">("buy")

  const amountFloat = parseFloat(amount) || 0
  const total = amountFloat * (token?.price || 0)

  const quickAmounts = [0.1, 0.25, 0.5, 1.0]

  const handleTrade = async (type: "buy" | "sell", quickAmount?: number) => {
    if (!token) return
    
    const tradeAmount = quickAmount || amountFloat
    if (tradeAmount <= 0) return
    
    if (type === "buy" && tradeAmount > balance) {
      alert("Insufficient balance")
      return
    }

    setIsProcessing(true)
    try {
      await onTrade({
        type,
        token: token.symbol,
        amount: tradeAmount,
        price: token.price
      })
      setAmount("")
    } catch (error) {
      console.error("Trade failed:", error)
      alert("Trade failed: " + error)
    } finally {
      setIsProcessing(false)
    }
  }

  const getRiskColor = (riskLevel: string) => {
    switch (riskLevel) {
      case 'low': return 'text-emerald-400'
      case 'medium': return 'text-yellow-400'
      case 'high': return 'text-red-400'
      default: return 'text-muted-foreground'
    }
  }

  const getRiskIcon = (riskLevel: string) => {
    switch (riskLevel) {
      case 'low': return <Zap className="h-3 w-3" />
      case 'medium': return <AlertTriangle className="h-3 w-3" />
      case 'high': return <AlertTriangle className="h-3 w-3" />
      default: return null
    }
  }

  if (!token) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="text-sm font-medium">Trading Interface</CardTitle>
        </CardHeader>
        <CardContent className="flex items-center justify-center h-32">
          <p className="text-sm text-muted-foreground">Select a token to start trading</p>
        </CardContent>
      </Card>
    )
  }

  return (
    <div className="space-y-4">
      {/* Token Info */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-full bg-gradient-to-br from-primary to-primary/60 flex items-center justify-center">
            <span className="text-sm font-bold text-primary-foreground">
              {token.symbol.charAt(0)}
            </span>
          </div>
          <div>
            <h3 className="font-semibold text-foreground">{token.symbol}</h3>
            <p className="text-sm text-muted-foreground">{token.name}</p>
          </div>
        </div>
        <div className="text-right">
          <div className="text-lg font-bold text-foreground">
            ${token.price >= 0.001 ? token.price.toFixed(4) : token.price.toFixed(8)}
          </div>
          <div className={`text-sm flex items-center gap-1 ${
            token.change24h >= 0 ? 'text-emerald-400' : 'text-red-400'
          }`}>
            {token.change24h >= 0 ? (
              <TrendingUp className="h-3 w-3" />
            ) : (
              <TrendingDown className="h-3 w-3" />
            )}
            {Math.abs(token.change24h).toFixed(1)}%
          </div>
        </div>
      </div>

      {/* Risk Assessment */}
      <Card className="border-border/50">
        <CardContent className="p-3">
          <div className="flex items-center justify-between text-sm">
            <div className="flex items-center gap-2">
              <span className="text-muted-foreground">Risk Level:</span>
              <Badge 
                variant="outline" 
                className={`${getRiskColor(token.riskLevel)} border-current`}
              >
                {getRiskIcon(token.riskLevel)}
                <span className="ml-1 capitalize">{token.riskLevel}</span>
              </Badge>
            </div>
            <div className="flex items-center gap-4 text-muted-foreground">
              <span>Buy Tax: {token.buyTax}%</span>
              <span>Sell Tax: {token.sellTax}%</span>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Trading Tabs */}
      <div className="flex rounded-lg bg-muted p-1">
        <Button
          variant={activeTab === "buy" ? "default" : "ghost"}
          className="flex-1 h-9"
          onClick={() => setActiveTab("buy")}
        >
          <ArrowUpRight className="h-4 w-4 mr-2" />
          Buy
        </Button>
        <Button
          variant={activeTab === "sell" ? "default" : "ghost"}
          className="flex-1 h-9"
          onClick={() => setActiveTab("sell")}
        >
          <ArrowDownRight className="h-4 w-4 mr-2" />
          Sell
        </Button>
      </div>

      {/* Balance Display */}
      <div className="text-sm text-muted-foreground">
        Balance: <span className="font-medium text-foreground">{balance.toFixed(4)} SOL</span>
      </div>

      {/* Amount Input */}
      <div className="space-y-2">
        <label className="text-sm font-medium text-foreground">
          Amount (SOL)
        </label>
        <div className="relative">
          <input
            type="number"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            placeholder="0.00"
            className="w-full px-3 py-2 bg-background border border-border rounded-md text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent"
            step="0.01"
            min="0"
          />
          <div className="absolute right-3 top-2 text-sm text-muted-foreground">
            SOL
          </div>
        </div>

        {/* Quick Amount Buttons */}
        <div className="grid grid-cols-4 gap-2">
          {quickAmounts.map((quickAmount) => (
            <Button
              key={quickAmount}
              variant="outline"
              size="sm"
              className="text-xs h-8"
              onClick={() => setAmount(quickAmount.toString())}
            >
              {quickAmount}
            </Button>
          ))}
        </div>
      </div>

      {/* Trade Summary */}
      {amountFloat > 0 && (
        <Card className="border-border/50">
          <CardContent className="p-3 space-y-2">
            <div className="flex justify-between text-sm">
              <span className="text-muted-foreground">You pay:</span>
              <span className="text-foreground">{amountFloat} SOL</span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-muted-foreground">You get:</span>
              <span className="text-foreground">
                {token.price > 0 ? (amountFloat / token.price).toLocaleString(undefined, { maximumFractionDigits: 0 }) : '0'} {token.symbol}
              </span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-muted-foreground">Estimated price:</span>
              <span className="text-foreground">${token.price.toFixed(8)}</span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-muted-foreground">Slippage tolerance:</span>
              <span className="text-foreground">5.0%</span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-muted-foreground">Tax ({activeTab}):</span>
              <span className="text-foreground">
                {activeTab === "buy" ? token.buyTax : token.sellTax}%
              </span>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Trade Button */}
      <Button
        className="w-full h-12 text-lg font-semibold"
        disabled={amountFloat <= 0 || isProcessing || (activeTab === "buy" && amountFloat > balance)}
        onClick={() => handleTrade(activeTab)}
      >
        {isProcessing ? (
          <>
            <div className="animate-spin h-4 w-4 mr-2 border-2 border-foreground border-t-transparent rounded-full" />
            Processing...
          </>
        ) : (
          <>
            {activeTab === "buy" ? (
              <ArrowUpRight className="h-5 w-5 mr-2" />
            ) : (
              <ArrowDownRight className="h-5 w-5 mr-2" />
            )}
            {activeTab === "buy" ? "Buy" : "Sell"} {token.symbol}
          </>
        )}
      </Button>

      {/* Warning */}
      <div className="text-xs text-muted-foreground bg-muted/50 p-2 rounded">
        ⚠️ High-risk memecoin. Always do your own research before investing.
      </div>
    </div>
  )
}