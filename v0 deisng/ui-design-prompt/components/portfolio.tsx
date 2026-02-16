"use client"

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import type { Holding } from "@/lib/mock-data"

interface PortfolioProps {
  holdings: Holding[]
  solBalance: number
}

export default function Portfolio({ holdings, solBalance }: PortfolioProps) {
  const totalValue = holdings.reduce((sum, h) => sum + h.value, 0) + solBalance
  const totalPnl = holdings.reduce((sum, h) => sum + h.pnl, 0)
  const totalPnlPercent = totalPnl / (totalValue - totalPnl) * 100

  return (
    <div className="space-y-4">
      {/* Summary */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-xs font-medium text-muted-foreground">Total Value</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-xl font-bold font-mono text-foreground">
              {totalValue.toFixed(2)} SOL
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-xs font-medium text-muted-foreground">Total PnL</CardTitle>
          </CardHeader>
          <CardContent>
            <div className={`text-xl font-bold font-mono ${totalPnl >= 0 ? "text-emerald-400" : "text-red-400"}`}>
              {totalPnl >= 0 ? "+" : ""}{totalPnl.toFixed(2)} SOL
            </div>
            <div className={`text-xs ${totalPnlPercent >= 0 ? "text-emerald-400" : "text-red-400"}`}>
              {totalPnlPercent >= 0 ? "+" : ""}{totalPnlPercent.toFixed(1)}%
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-xs font-medium text-muted-foreground">SOL Balance</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-xl font-bold font-mono text-foreground">
              {solBalance.toFixed(4)} SOL
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Holdings Table */}
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm">Holdings</CardTitle>
        </CardHeader>
        <CardContent>
          {holdings.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground text-sm">
              No holdings yet. Buy some tokens to get started.
            </div>
          ) : (
            <div className="space-y-0">
              {/* Header */}
              <div className="grid grid-cols-[1fr_80px_80px_80px_80px] gap-2 text-[10px] uppercase tracking-wider font-semibold text-muted-foreground pb-2 border-b border-border">
                <span>Token</span>
                <span className="text-right">Amount</span>
                <span className="text-right">Price</span>
                <span className="text-right">Value</span>
                <span className="text-right">PnL</span>
              </div>

              {holdings.map((h) => (
                <div
                  key={h.symbol}
                  className="grid grid-cols-[1fr_80px_80px_80px_80px] gap-2 items-center py-2.5 border-b border-border/30 last:border-0"
                >
                  <div>
                    <span className="text-sm font-semibold text-foreground">{h.symbol}</span>
                    <span className="text-xs text-muted-foreground ml-1.5">{h.token}</span>
                  </div>
                  <span className="text-xs text-right font-mono text-foreground">
                    {h.amount.toLocaleString()}
                  </span>
                  <span className="text-xs text-right font-mono text-foreground">
                    ${h.currentPrice < 0.01 ? h.currentPrice.toFixed(6) : h.currentPrice.toFixed(4)}
                  </span>
                  <span className="text-xs text-right font-mono text-foreground">
                    ${h.value.toFixed(2)}
                  </span>
                  <div className="text-right">
                    <span className={`text-xs font-mono font-medium ${h.pnl >= 0 ? "text-emerald-400" : "text-red-400"}`}>
                      {h.pnl >= 0 ? "+" : ""}${h.pnl.toFixed(2)}
                    </span>
                    <span className={`text-[10px] block ${h.pnlPercent >= 0 ? "text-emerald-400" : "text-red-400"}`}>
                      {h.pnlPercent >= 0 ? "+" : ""}{h.pnlPercent.toFixed(1)}%
                    </span>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
