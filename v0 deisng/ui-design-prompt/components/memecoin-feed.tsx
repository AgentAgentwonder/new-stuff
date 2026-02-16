"use client"

import { useState, useCallback } from "react"
import type { MemecoinToken } from "@/lib/mock-data"
import RiskIndicator from "@/components/risk-indicator"
import { ArrowUp, ArrowDown, AlertTriangle } from "lucide-react"

interface MemecoinFeedProps {
  coins: MemecoinToken[]
  selectedId: string | null
  onSelect: (coin: MemecoinToken) => void
}

type SortKey = "age" | "marketCap" | "liquidity" | "holders" | "fees"
type SortDir = "asc" | "desc"

function formatAge(seconds: number): string {
  if (seconds < 60) return `${seconds}s`
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m`
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h`
  return `${Math.floor(seconds / 86400)}d`
}

function formatMC(n: number): string {
  if (n >= 1e9) return `$${(n / 1e9).toFixed(1)}B`
  if (n >= 1e6) return `$${(n / 1e6).toFixed(1)}M`
  if (n >= 1e3) return `$${(n / 1e3).toFixed(0)}K`
  return `$${n.toFixed(0)}`
}

function formatLiq(n: number): string {
  if (n >= 1e6) return `${(n / 1e6).toFixed(1)}M`
  if (n >= 1e3) return `${(n / 1e3).toFixed(1)}K`
  return n.toFixed(0)
}

export default function MemecoinFeed({ coins, selectedId, onSelect }: MemecoinFeedProps) {
  const [sortKey, setSortKey] = useState<SortKey>("age")
  const [sortDir, setSortDir] = useState<SortDir>("asc")

  const handleSort = useCallback((key: SortKey) => {
    setSortKey((prev) => {
      if (prev === key) {
        setSortDir((d) => (d === "asc" ? "desc" : "asc"))
        return prev
      }
      setSortDir("asc")
      return key
    })
  }, [])

  const sorted = [...coins].sort((a, b) => {
    let diff = 0
    switch (sortKey) {
      case "age":
        diff = a.age - b.age
        break
      case "marketCap":
        diff = a.marketCap - b.marketCap
        break
      case "liquidity":
        diff = a.liquidity - b.liquidity
        break
      case "holders":
        diff = a.holders - b.holders
        break
      case "fees":
        diff = (a.buyTax + a.sellTax) - (b.buyTax + b.sellTax)
        break
    }
    return sortDir === "asc" ? diff : -diff
  })

  const SortIcon = ({ col }: { col: SortKey }) => {
    if (sortKey !== col) return null
    return sortDir === "asc" ? (
      <ArrowUp className="h-3 w-3 inline ml-0.5 text-emerald-400" />
    ) : (
      <ArrowDown className="h-3 w-3 inline ml-0.5 text-emerald-400" />
    )
  }

  const headerBtn = (label: string, col: SortKey, align?: string) => (
    <button
      onClick={() => handleSort(col)}
      className={`text-[10px] uppercase tracking-wider font-semibold text-muted-foreground hover:text-foreground transition-colors ${align || ""}`}
    >
      {label}
      <SortIcon col={col} />
    </button>
  )

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between pb-2 px-1">
        <h3 className="text-xs font-semibold text-foreground tracking-wide uppercase">New Tokens</h3>
        <span className="text-[10px] text-muted-foreground">{coins.length} tokens</span>
      </div>

      {/* Header */}
      <div className="grid grid-cols-[1.2fr_50px_65px_60px_50px_50px_36px] gap-1.5 px-2 py-1.5 border-b border-border/60 items-center">
        <span className="text-[10px] uppercase tracking-wider font-semibold text-muted-foreground">Token</span>
        {headerBtn("Age", "age")}
        {headerBtn("MC", "marketCap")}
        {headerBtn("Liq", "liquidity")}
        {headerBtn("Hld", "holders")}
        {headerBtn("Fee", "fees")}
        <span className="text-[10px] uppercase tracking-wider font-semibold text-muted-foreground text-center">Risk</span>
      </div>

      {/* Rows */}
      <div className="flex-1 overflow-y-auto">
        {sorted.map((coin) => (
          <button
            key={coin.id}
            onClick={() => onSelect(coin)}
            className={`grid grid-cols-[1.2fr_50px_65px_60px_50px_50px_36px] gap-1.5 px-2 py-2 items-center w-full text-left transition-colors border-b border-border/30 ${
              selectedId === coin.id
                ? "bg-emerald-500/10 border-l-2 border-l-emerald-500"
                : "hover:bg-muted/50"
            }`}
          >
            {/* Token */}
            <div className="flex items-center gap-1.5 min-w-0">
              <div className="min-w-0">
                <div className="flex items-center gap-1">
                  <span className="text-xs font-semibold text-foreground truncate">{coin.symbol}</span>
                  {coin.isNew && (
                    <span className="text-[9px] font-bold px-1 py-0 rounded bg-emerald-500/20 text-emerald-400 leading-tight">
                      NEW
                    </span>
                  )}
                  {coin.whaleAlert && (
                    <AlertTriangle className="h-3 w-3 text-yellow-400 flex-shrink-0" />
                  )}
                </div>
                <span className="text-[10px] text-muted-foreground truncate block">{coin.name}</span>
              </div>
            </div>
            {/* Age */}
            <span className={`text-xs font-mono ${coin.age < 60 ? "text-emerald-400 font-bold" : "text-foreground"}`}>
              {formatAge(coin.age)}
            </span>
            {/* MC */}
            <span className="text-xs font-mono text-foreground">{formatMC(coin.marketCap)}</span>
            {/* Liquidity */}
            <span className="text-xs font-mono text-foreground">{formatLiq(coin.liquidity)}</span>
            {/* Holders */}
            <span className="text-xs font-mono text-foreground">{coin.holders.toLocaleString()}</span>
            {/* Fees */}
            <span className="text-xs font-mono text-muted-foreground">{coin.buyTax + coin.sellTax}%</span>
            {/* Risk */}
            <div className="flex justify-center">
              <RiskIndicator level={coin.riskLevel} showLabel={false} />
            </div>
          </button>
        ))}
      </div>
    </div>
  )
}
