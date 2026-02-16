"use client"

import { useMemo } from "react"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent } from "@/components/ui/card"
import { ArrowUpRight, ArrowDownRight, Clock, Users, Zap, AlertTriangle } from "lucide-react"
import { type MemecoinToken } from "@/store/dashboardStore"

interface MemecoinFeedProps {
  coins: MemecoinToken[]
  selectedId: string | null
  onSelect: (coin: MemecoinToken) => void
}

export default function MemecoinFeed({ coins, selectedId, onSelect }: MemecoinFeedProps) {
  const sortedCoins = useMemo(() => {
    return [...coins].sort((a, b) => {
      // Sort by new coins first, then by volume, then by recent activity
      if (a.isNew !== b.isNew) return b.isNew ? 1 : -1
      if (a.whaleAlert !== b.whaleAlert) return b.whaleAlert ? 1 : -1
      return b.volume24h - a.volume24h
    })
  }, [coins])

  const formatPrice = (price: number) => {
    if (price >= 0.001) return price.toFixed(4)
    if (price >= 0.0001) return price.toFixed(6)
    return price.toFixed(8)
  }

  const formatMarketCap = (mc: number) => {
    if (mc >= 1000000) return `$${(mc / 1000000).toFixed(1)}M`
    if (mc >= 1000) return `$${(mc / 1000).toFixed(1)}K`
    return mc.toFixed(0)
  }

  const formatAge = (seconds: number) => {
    if (seconds < 60) return `${seconds}s`
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m`
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}h`
    return `${Math.floor(seconds / 86400)}d`
  }

  const getRiskColor = (risk: string) => {
    switch (risk) {
      case 'low': return 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30'
      case 'medium': return 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30'
      case 'high': return 'bg-red-500/20 text-red-400 border-red-500/30'
      default: return 'bg-gray-500/20 text-gray-400 border-gray-500/30'
    }
  }

  const getRiskIcon = (risk: string) => {
    switch (risk) {
      case 'low': return <Zap className="h-3 w-3" />
      case 'medium': return <AlertTriangle className="h-3 w-3" />
      case 'high': return <AlertTriangle className="h-3 w-3" />
      default: return null
    }
  }

  return (
    <div className="h-full flex flex-col">
      <div className="flex items-center justify-between p-2 border-b">
        <h3 className="text-sm font-semibold text-foreground">Memecoin Feed</h3>
        <span className="text-xs text-muted-foreground">{coins.length} tokens</span>
      </div>
      
      <div className="flex-1 overflow-y-auto">
        {sortedCoins.length === 0 ? (
          <div className="flex items-center justify-center h-32 text-sm text-muted-foreground">
            No tokens available
          </div>
        ) : (
          <div className="space-y-1 p-2">
            {sortedCoins.map((coin) => (
              <Card
                key={coin.id}
                className={`cursor-pointer transition-all hover:shadow-md ${
                  selectedId === coin.id ? 'border-primary shadow-md' : ''
                } ${coin.isNew ? 'ring-1 ring-emerald-500/30' : ''} ${
                  coin.whaleAlert ? 'ring-1 ring-yellow-500/30' : ''
                }`}
                onClick={() => onSelect(coin)}
              >
                <CardContent className="p-3">
                  <div className="flex items-start justify-between mb-2">
                    <div className="flex items-center gap-2 min-w-0 flex-1">
                      <div className="flex-shrink-0">
                        <div className="w-6 h-6 rounded-full bg-gradient-to-br from-primary to-primary/60 flex items-center justify-center">
                          <span className="text-xs font-bold text-primary-foreground">
                            {coin.symbol.charAt(0)}
                          </span>
                        </div>
                      </div>
                      <div className="min-w-0 flex-1">
                        <div className="flex items-center gap-2">
                          <h4 className="text-sm font-semibold text-foreground truncate">
                            {coin.symbol}
                          </h4>
                          {coin.isNew && (
                            <Badge variant="secondary" className="text-[10px] px-1 py-0 h-4 bg-emerald-500/20 text-emerald-400 border-emerald-500/30">
                              NEW
                            </Badge>
                          )}
                          {coin.whaleAlert && (
                            <Badge variant="secondary" className="text-[10px] px-1 py-0 h-4 bg-yellow-500/20 text-yellow-400 border-yellow-500/30">
                              WHALE
                            </Badge>
                          )}
                        </div>
                        <p className="text-[11px] text-muted-foreground truncate">{coin.name}</p>
                      </div>
                    </div>
                    <div className="text-right flex-shrink-0">
                      <div className="text-sm font-bold text-foreground">
                        ${formatPrice(coin.price)}
                      </div>
                      <div className={`text-xs flex items-center gap-1 ${
                        coin.change24h >= 0 ? 'text-emerald-400' : 'text-red-400'
                      }`}>
                        {coin.change24h >= 0 ? (
                          <ArrowUpRight className="h-3 w-3" />
                        ) : (
                          <ArrowDownRight className="h-3 w-3" />
                        )}
                        {Math.abs(coin.change24h).toFixed(1)}%
                      </div>
                    </div>
                  </div>

                  <div className="grid grid-cols-3 gap-2 text-[11px]">
                    <div>
                      <div className="text-muted-foreground">MC</div>
                      <div className="font-medium text-foreground">{formatMarketCap(coin.marketCap)}</div>
                    </div>
                    <div>
                      <div className="text-muted-foreground flex items-center gap-1">
                        <Users className="h-3 w-3" />
                        Holders
                      </div>
                      <div className="font-medium text-foreground">{coin.holders}</div>
                    </div>
                    <div>
                      <div className="text-muted-foreground flex items-center gap-1">
                        <Clock className="h-3 w-3" />
                        Age
                      </div>
                      <div className="font-medium text-foreground">{formatAge(coin.age)}</div>
                    </div>
                  </div>

                  <div className="flex items-center justify-between mt-2">
                    <div className="flex items-center gap-2">
                      <Badge 
                        variant="outline" 
                        className={`text-[10px] px-1 py-0 h-4 ${getRiskColor(coin.riskLevel)}`}
                      >
                        {getRiskIcon(coin.riskLevel)}
                        <span className="ml-1 capitalize">{coin.riskLevel}</span>
                      </Badge>
                    </div>
                    <div className="text-[10px] text-muted-foreground">
                      {coin.buyTax}%/{coin.sellTax}% tax
                    </div>
                  </div>

                  {/* Volume indicator */}
                  {coin.volume24h > 0 && (
                    <div className="mt-2 pt-2 border-t border-border/50">
                      <div className="flex items-center justify-between text-[10px] text-muted-foreground">
                        <span>24h Volume</span>
                        <span className="font-medium">{formatMarketCap(coin.volume24h)}</span>
                      </div>
                      <div className="mt-1 h-1 bg-muted rounded-full overflow-hidden">
                        <div 
                          className="h-full bg-primary rounded-full transition-all duration-300"
                          style={{ 
                            width: `${Math.min(coin.volume24h / 100000 * 100, 100)}%` 
                          }}
                        />
                      </div>
                    </div>
                  )}
                </CardContent>
              </Card>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}