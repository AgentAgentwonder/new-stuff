"use client"

import type React from "react"
import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import type { MemecoinToken } from "@/lib/mock-data"

interface TradingInterfaceProps {
  token: MemecoinToken | null
  balance: number
  onTrade: (trade: { type: "buy" | "sell"; token: string; amount: number; price: number }) => void
}

const SLIPPAGE_OPTIONS = [0.5, 1, 2, 5]

export default function TradingInterface({ token, balance, onTrade }: TradingInterfaceProps) {
  const [amount, setAmount] = useState("")
  const [side, setSide] = useState<"buy" | "sell">("buy")
  const [slippage, setSlippage] = useState(1)

  if (!token) {
    return (
      <div className="flex items-center justify-center h-full text-sm text-muted-foreground p-6">
        Select a token from the feed to trade
      </div>
    )
  }

  const numericAmount = Number.parseFloat(amount) || 0
  const estimatedReceive = numericAmount > 0 && token.price > 0 ? numericAmount / token.price : 0
  const canExecute = side === "buy" ? numericAmount > 0 && numericAmount <= balance : numericAmount > 0

  const handleExecute = () => {
    if (!canExecute) return
    onTrade({
      type: side,
      token: token.symbol,
      amount: numericAmount,
      price: token.price,
    })
    setAmount("")
  }

  const handlePercentClick = (pct: number) => {
    setAmount((balance * pct / 100).toFixed(4))
  }

  return (
    <div className="space-y-3 p-1">
      {/* Token header */}
      <div className="flex items-center justify-between">
        <div>
          <span className="text-sm font-bold text-foreground">{token.symbol}</span>
          <span className="text-xs text-muted-foreground ml-2">{token.name}</span>
        </div>
        <span className="text-sm font-mono font-bold text-foreground">
          ${token.price < 0.01 ? token.price.toFixed(6) : token.price.toFixed(4)}
        </span>
      </div>

      {/* Buy/Sell Toggle */}
      <div className="grid grid-cols-2 gap-1 p-0.5 rounded-lg bg-muted">
        <button
          onClick={() => setSide("buy")}
          className={`py-2 text-sm font-semibold rounded-md transition-colors ${
            side === "buy" ? "bg-emerald-500/20 text-emerald-400" : "text-muted-foreground hover:text-foreground"
          }`}
        >
          Buy
        </button>
        <button
          onClick={() => setSide("sell")}
          className={`py-2 text-sm font-semibold rounded-md transition-colors ${
            side === "sell" ? "bg-red-500/20 text-red-400" : "text-muted-foreground hover:text-foreground"
          }`}
        >
          Sell
        </button>
      </div>

      {/* Amount */}
      <div className="space-y-1.5">
        <Label htmlFor="trade-amount" className="text-xs text-muted-foreground">
          Amount (SOL)
        </Label>
        <Input
          id="trade-amount"
          type="number"
          placeholder="0.00"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          className="bg-muted border-border font-mono"
        />
        {/* Quick percent buttons */}
        <div className="flex gap-1">
          {[25, 50, 75, 100].map((pct) => (
            <button
              key={pct}
              onClick={() => handlePercentClick(pct)}
              className="flex-1 text-[10px] py-1 rounded bg-muted hover:bg-muted/80 text-muted-foreground hover:text-foreground transition-colors font-medium"
            >
              {pct}%
            </button>
          ))}
        </div>
      </div>

      {/* Preview */}
      {numericAmount > 0 && (
        <div className="rounded-lg bg-muted/50 p-2.5 space-y-1">
          <div className="flex justify-between text-xs">
            <span className="text-muted-foreground">
              {side === "buy" ? "Receive" : "Send"} ~
            </span>
            <span className="font-mono text-foreground">
              {estimatedReceive.toLocaleString(undefined, { maximumFractionDigits: 2 })} {token.symbol}
            </span>
          </div>
          <div className="flex justify-between text-xs">
            <span className="text-muted-foreground">Price Impact</span>
            <span className="font-mono text-yellow-400">~{(numericAmount * 0.3).toFixed(2)}%</span>
          </div>
        </div>
      )}

      {/* Slippage */}
      <div className="space-y-1.5">
        <Label className="text-xs text-muted-foreground">Slippage Tolerance</Label>
        <div className="flex gap-1">
          {SLIPPAGE_OPTIONS.map((s) => (
            <button
              key={s}
              onClick={() => setSlippage(s)}
              className={`flex-1 text-xs py-1.5 rounded font-medium transition-colors ${
                slippage === s
                  ? "bg-emerald-500/20 text-emerald-400"
                  : "bg-muted text-muted-foreground hover:text-foreground"
              }`}
            >
              {s}%
            </button>
          ))}
        </div>
      </div>

      {/* Balance */}
      <div className="flex justify-between text-xs text-muted-foreground pt-1">
        <span>Balance</span>
        <span className="font-mono text-foreground">{balance.toFixed(4)} SOL</span>
      </div>

      {/* Swap Button */}
      <Button
        className={`w-full font-bold text-sm py-5 ${
          side === "buy"
            ? "bg-emerald-600 hover:bg-emerald-700 text-white"
            : "bg-red-600 hover:bg-red-700 text-white"
        }`}
        onClick={handleExecute}
        disabled={!canExecute}
      >
        {side === "buy" ? "BUY" : "SELL"} {token.symbol}
      </Button>
    </div>
  )
}
