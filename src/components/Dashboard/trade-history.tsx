"use client"

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { ArrowUpRight, ArrowDownRight, Clock, CheckCircle, XCircle, Loader2, ExternalLink } from "lucide-react"
import { type Trade } from "@/store/dashboardStore"

interface TradeHistoryPanelProps {
  trades: Trade[]
}

export default function TradeHistoryPanel({ trades }: TradeHistoryPanelProps) {
  const formatTime = (isoString: string) => {
    const date = new Date(isoString)
    const now = new Date()
    const diff = now.getTime() - date.getTime()
    
    const minutes = Math.floor(diff / 60000)
    const hours = Math.floor(diff / 3600000)
    const days = Math.floor(diff / 86400000)
    
    if (minutes < 1) return 'Just now'
    if (minutes < 60) return `${minutes}m ago`
    if (hours < 24) return `${hours}h ago`
    if (days < 7) return `${days}d ago`
    
    return date.toLocaleDateString()
  }

  const formatPrice = (price: number) => {
    if (price >= 1) return `$${price.toFixed(2)}`
    if (price >= 0.01) return `$${price.toFixed(4)}`
    return `$${price.toFixed(8)}`
  }

  const formatAmount = (amount: number) => {
    return amount.toFixed(4)
  }

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'filled':
        return <CheckCircle className="h-4 w-4 text-emerald-400" />
      case 'pending':
        return <Loader2 className="h-4 w-4 text-yellow-400 animate-spin" />
      case 'cancelled':
        return <XCircle className="h-4 w-4 text-red-400" />
      case 'failed':
        return <XCircle className="h-4 w-4 text-red-400" />
      default:
        return null
    }
  }

  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'filled':
        return <Badge variant="default" className="bg-emerald-500/20 text-emerald-400 border-emerald-500/30">Filled</Badge>
      case 'pending':
        return <Badge variant="secondary" className="bg-yellow-500/20 text-yellow-400 border-yellow-500/30">Pending</Badge>
      case 'cancelled':
        return <Badge variant="destructive" className="bg-red-500/20 text-red-400 border-red-500/30">Cancelled</Badge>
      case 'failed':
        return <Badge variant="destructive" className="bg-red-500/20 text-red-400 border-red-500/30">Failed</Badge>
      default:
        return <Badge variant="outline">{status}</Badge>
    }
  }

  // Calculate stats
  const buyTrades = trades.filter(t => t.type === 'buy')
  const sellTrades = trades.filter(t => t.type === 'sell')
  const totalBuyVolume = buyTrades.reduce((sum, t) => sum + t.amount, 0)
  const totalSellVolume = sellTrades.reduce((sum, t) => sum + t.amount, 0)
  const filledTrades = trades.filter(t => t.status === 'filled')
  const pendingTrades = trades.filter(t => t.status === 'pending')

  return (
    <div className="space-y-6">
      {/* Stats Summary */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Total Trades
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-foreground">{trades.length}</div>
            <div className="text-xs text-muted-foreground">
              {filledTrades.length} filled, {pendingTrades.length} pending
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground flex items-center gap-2">
              <ArrowUpRight className="h-4 w-4 text-emerald-400" />
              Buy Volume
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-foreground">{formatAmount(totalBuyVolume)}</div>
            <div className="text-xs text-muted-foreground">{buyTrades.length} trades</div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground flex items-center gap-2">
              <ArrowDownRight className="h-4 w-4 text-red-400" />
              Sell Volume
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-foreground">{formatAmount(totalSellVolume)}</div>
            <div className="text-xs text-muted-foreground">{sellTrades.length} trades</div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Net Volume
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className={`text-2xl font-bold ${totalBuyVolume - totalSellVolume >= 0 ? 'text-emerald-400' : 'text-red-400'}`}>
              {formatAmount(Math.abs(totalBuyVolume - totalSellVolume))}
            </div>
            <div className="text-xs text-muted-foreground">
              {totalBuyVolume - totalSellVolume >= 0 ? 'Net Buy' : 'Net Sell'}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Trade History */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg font-semibold flex items-center gap-2">
            <Clock className="h-5 w-5" />
            Trade History
          </CardTitle>
        </CardHeader>
        <CardContent>
          {trades.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-32 text-muted-foreground">
              <Clock className="h-8 w-8 mb-2 opacity-50" />
              <p>No trades yet</p>
              <p className="text-sm">Start trading to see your history</p>
            </div>
          ) : (
            <div className="space-y-2">
              {trades.map((trade) => (
                <div
                  key={trade.id}
                  className="flex items-center justify-between p-4 rounded-lg border border-border hover:bg-muted/50 transition-colors"
                >
                  <div className="flex items-center gap-4">
                    {/* Trade Type Icon */}
                    <div className={`w-10 h-10 rounded-full flex items-center justify-center ${
                      trade.type === 'buy' 
                        ? 'bg-emerald-500/20 text-emerald-400' 
                        : 'bg-red-500/20 text-red-400'
                    }`}>
                      {trade.type === 'buy' ? (
                        <ArrowUpRight className="h-5 w-5" />
                      ) : (
                        <ArrowDownRight className="h-5 w-5" />
                      )}
                    </div>

                    {/* Token Info */}
                    <div>
                      <div className="flex items-center gap-2">
                        <h4 className="font-semibold text-foreground uppercase">{trade.token}</h4>
                        <span className={`text-sm font-medium ${
                          trade.type === 'buy' ? 'text-emerald-400' : 'text-red-400'
                        }`}>
                          {trade.type.toUpperCase()}
                        </span>
                        {getStatusBadge(trade.status)}
                      </div>
                      <div className="text-sm text-muted-foreground">
                        {formatTime(trade.time)}
                      </div>
                    </div>
                  </div>

                  {/* Trade Details */}
                  <div className="flex items-center gap-8">
                    <div className="text-right">
                      <div className="text-sm text-muted-foreground">Amount</div>
                      <div className="font-medium text-foreground">{formatAmount(trade.amount)} SOL</div>
                    </div>

                    <div className="text-right">
                      <div className="text-sm text-muted-foreground">Price</div>
                      <div className="font-medium text-foreground">{formatPrice(trade.price)}</div>
                    </div>

                    <div className="text-right">
                      <div className="text-sm text-muted-foreground">Total</div>
                      <div className="font-medium text-foreground">{formatAmount(trade.total)} SOL</div>
                    </div>

                    <div className="text-right">
                      {getStatusIcon(trade.status)}
                      {trade.txSignature && (
                        <a
                          href={`https://solscan.io/tx/${trade.txSignature}`}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="ml-2 text-muted-foreground hover:text-foreground"
                        >
                          <ExternalLink className="h-4 w-4" />
                        </a>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Filters */}
      <Card>
        <CardContent className="p-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <span className="text-sm text-muted-foreground">Filter by:</span>
              <div className="flex gap-2">
                <Badge variant="outline" className="cursor-pointer hover:bg-muted">All</Badge>
                <Badge variant="outline" className="cursor-pointer hover:bg-muted">Buy</Badge>
                <Badge variant="outline" className="cursor-pointer hover:bg-muted">Sell</Badge>
                <Badge variant="outline" className="cursor-pointer hover:bg-muted">Filled</Badge>
                <Badge variant="outline" className="cursor-pointer hover:bg-muted">Pending</Badge>
              </div>
            </div>
            <Button variant="outline" size="sm">
              Export History
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}