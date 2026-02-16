"use client"

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import type { Trade } from "@/lib/mock-data"

interface TradeHistoryProps {
  trades: Trade[]
}

export default function TradeHistory({ trades }: TradeHistoryProps) {
  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-sm">Trade History</CardTitle>
      </CardHeader>
      <CardContent>
        {trades.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground text-sm">
            No trades yet. Start trading to see your history.
          </div>
        ) : (
          <div className="space-y-0">
            {/* Header */}
            <div className="grid grid-cols-[60px_60px_50px_70px_70px_70px_56px] gap-2 text-[10px] uppercase tracking-wider font-semibold text-muted-foreground pb-2 border-b border-border">
              <span>Time</span>
              <span>Token</span>
              <span>Type</span>
              <span className="text-right">Amount</span>
              <span className="text-right">Price</span>
              <span className="text-right">Total</span>
              <span className="text-right">Status</span>
            </div>

            {trades
              .slice()
              .reverse()
              .map((trade) => (
                <div
                  key={trade.id}
                  className="grid grid-cols-[60px_60px_50px_70px_70px_70px_56px] gap-2 items-center py-2 border-b border-border/30 last:border-0"
                >
                  <span className="text-[11px] text-muted-foreground font-mono">
                    {new Date(trade.time).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
                  </span>
                  <span className="text-xs font-semibold text-foreground">{trade.token}</span>
                  <span
                    className={`text-[11px] font-semibold ${
                      trade.type === "buy" ? "text-emerald-400" : "text-red-400"
                    }`}
                  >
                    {trade.type.toUpperCase()}
                  </span>
                  <span className="text-xs text-right font-mono text-foreground">
                    {trade.amount.toFixed(4)}
                  </span>
                  <span className="text-xs text-right font-mono text-foreground">
                    ${trade.price < 0.01 ? trade.price.toFixed(6) : trade.price.toFixed(4)}
                  </span>
                  <span className="text-xs text-right font-mono text-foreground">
                    {trade.total.toFixed(4)}
                  </span>
                  <span
                    className={`text-[10px] text-right font-medium ${
                      trade.status === "filled"
                        ? "text-emerald-400"
                        : trade.status === "pending"
                          ? "text-yellow-400"
                          : "text-red-400"
                    }`}
                  >
                    {trade.status}
                  </span>
                </div>
              ))}
          </div>
        )}
      </CardContent>
    </Card>
  )
}
