"use client"

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { TrendingUp, TrendingDown, DollarSign, PieChart, Wallet } from "lucide-react"
import { type Holding } from "@/store/dashboardStore"

interface PortfolioViewProps {
  holdings: Holding[]
  solBalance: number
}

export default function PortfolioView({ holdings, solBalance }: PortfolioViewProps) {
  const totalValue = solBalance + holdings.reduce((sum, h) => sum + h.value, 0)
  const totalPnl = holdings.reduce((sum, h) => sum + h.pnl, 0)
  const totalChangePercent = totalValue > 0 ? (totalPnl / (totalValue - totalPnl)) * 100 : 0
  const isPositive = totalPnl >= 0

  const formatPrice = (price: number) => {
    if (price >= 1) return `$${price.toFixed(2)}`
    if (price >= 0.01) return `$${price.toFixed(4)}`
    return `$${price.toFixed(8)}`
  }

  const formatPercentage = (value: number) => {
    const sign = value >= 0 ? '+' : ''
    return `${sign}${value.toFixed(2)}%`
  }

  const formatValue = (value: number) => {
    if (value >= 1000000) return `$${(value / 1000000).toFixed(2)}M`
    if (value >= 1000) return `$${(value / 1000).toFixed(2)}K`
    return `$${value.toFixed(2)}`
  }

  return (
    <div className="space-y-6">
      {/* Portfolio Summary */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground flex items-center gap-2">
              <Wallet className="h-4 w-4" />
              Total Value
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-foreground">{formatValue(totalValue)}</div>
            <div className="text-sm text-muted-foreground">{totalValue.toFixed(4)} SOL</div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground flex items-center gap-2">
              <DollarSign className="h-4 w-4" />
              SOL Balance
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-foreground">{solBalance.toFixed(4)}</div>
            <div className="text-sm text-muted-foreground">{formatValue(solBalance)}</div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground flex items-center gap-2">
              <PieChart className="h-4 w-4" />
              Total PnL
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className={`text-2xl font-bold ${isPositive ? 'text-emerald-400' : 'text-red-400'}`}>
              {formatValue(Math.abs(totalPnl))}
            </div>
            <div className={`text-sm flex items-center gap-1 ${isPositive ? 'text-emerald-400' : 'text-red-400'}`}>
              {isPositive ? <TrendingUp className="h-3 w-3" /> : <TrendingDown className="h-3 w-3" />}
              {formatPercentage(totalChangePercent)}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Holdings
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-foreground">{holdings.length}</div>
            <div className="text-sm text-muted-foreground">tokens</div>
          </CardContent>
        </Card>
      </div>

      {/* Holdings List */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg font-semibold">Your Holdings</CardTitle>
        </CardHeader>
        <CardContent>
          {holdings.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-32 text-muted-foreground">
              <PieChart className="h-8 w-8 mb-2 opacity-50" />
              <p>No holdings yet</p>
              <p className="text-sm">Start trading to build your portfolio</p>
            </div>
          ) : (
            <div className="space-y-3">
              {holdings.map((holding) => (
                <div
                  key={holding.id}
                  className="flex items-center justify-between p-3 rounded-lg border border-border hover:bg-muted/50 transition-colors"
                >
                  <div className="flex items-center gap-3">
                    <div className="w-10 h-10 rounded-full bg-gradient-to-br from-primary to-primary/60 flex items-center justify-center">
                      <span className="text-sm font-bold text-primary-foreground">
                        {holding.symbol.charAt(0)}
                      </span>
                    </div>
                    <div>
                      <h3 className="font-semibold text-foreground">{holding.symbol}</h3>
                      <p className="text-sm text-muted-foreground">{holding.name}</p>
                    </div>
                  </div>

                  <div className="text-right">
                    <div className="font-semibold text-foreground">
                      {holding.amount.toLocaleString(undefined, { maximumFractionDigits: 0 })} tokens
                    </div>
                    <div className="text-sm text-muted-foreground">
                      {formatValue(holding.value)}
                    </div>
                  </div>

                  <div className="text-right">
                    <div className="font-semibold text-foreground">
                      {formatPrice(holding.price)}
                    </div>
                    <div className={`text-sm flex items-center gap-1 ${
                      holding.change24h >= 0 ? 'text-emerald-400' : 'text-red-400'
                    }`}>
                      {holding.change24h >= 0 ? <TrendingUp className="h-3 w-3" /> : <TrendingDown className="h-3 w-3" />}
                      {Math.abs(holding.change24h).toFixed(1)}%
                    </div>
                  </div>

                  <div className="text-right">
                    <div className={`font-semibold ${
                      holding.pnl >= 0 ? 'text-emerald-400' : 'text-red-400'
                    }`}>
                      {holding.pnl >= 0 ? '+' : ''}{formatValue(holding.pnl)}
                    </div>
                    <div className="text-sm text-muted-foreground">
                      {formatPercentage((holding.pnl / holding.value) * 100)}
                    </div>
                  </div>

                  <div className="flex items-center gap-2">
                    <Badge 
                      variant={holding.change24h >= 0 ? "default" : "destructive"}
                      className="text-xs"
                    >
                      {holding.change24h >= 0 ? "Profit" : "Loss"}
                    </Badge>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Portfolio Allocation */}
      {holdings.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="text-lg font-semibold">Portfolio Allocation</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">SOL</span>
                <span className="text-sm font-medium text-foreground">
                  {((solBalance / totalValue) * 100).toFixed(1)}%
                </span>
              </div>
              <div className="h-2 bg-muted rounded-full overflow-hidden">
                <div 
                  className="h-full bg-primary rounded-full transition-all duration-300"
                  style={{ width: `${(solBalance / totalValue) * 100}%` }}
                />
              </div>

              {holdings.map((holding, index) => (
                <div key={holding.id}>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">{holding.symbol}</span>
                    <span className="text-sm font-medium text-foreground">
                      {((holding.value / totalValue) * 100).toFixed(1)}%
                    </span>
                  </div>
                  <div className="h-2 bg-muted rounded-full overflow-hidden">
                    <div 
                      className="h-full rounded-full transition-all duration-300"
                      style={{ 
                        width: `${(holding.value / totalValue) * 100}%`,
                        backgroundColor: `hsl(${220 + index * 30}, 70%, 50%)`
                      }}
                    />
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Portfolio Actions */}
      <Card>
        <CardContent className="p-4">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="font-semibold text-foreground">Portfolio Actions</h3>
              <p className="text-sm text-muted-foreground">Manage your portfolio</p>
            </div>
            <div className="flex items-center gap-2">
              <Button variant="outline" size="sm">
                Export Data
              </Button>
              <Button variant="outline" size="sm">
                Rebalance
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}