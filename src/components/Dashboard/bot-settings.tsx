"use client"

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Switch } from "@/components/ui/switch"
import { Label } from "@/components/ui/label"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { 
  Settings, 
  Zap, 
  Shield, 
  TrendingUp, 
  Activity,
  DollarSign,
  BarChart3,
  AlertTriangle
} from "lucide-react"

interface BotSettingsPanelProps {
  isRunning: boolean
  setIsRunning: () => void
  isSimulation: boolean
  setIsSimulation: () => void
}

export default function BotSettingsPanel({
  isRunning,
  setIsRunning,
  isSimulation,
  setIsSimulation
}: BotSettingsPanelProps) {
  const [settings, setSettings] = useState({
    minProfitThreshold: 1.0,
    maxSlippage: 5.0,
    autoCompound: false,
    riskLevel: 'medium',
    tradeAmount: 0.05,
    maxPositionSize: 0.5,
    stopLoss: 10,
    takeProfit: 50,
  })

  return (
    <div className="space-y-6">
      {/* Bot Status */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Activity className="h-5 w-5" />
            Bot Status
          </CardTitle>
          <CardDescription>
            Control the trading bot and simulation mode
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between p-4 rounded-lg border border-border">
            <div className="flex items-center gap-3">
              <div className={`w-3 h-3 rounded-full ${isRunning ? 'bg-emerald-500 animate-pulse' : 'bg-red-500'}`} />
              <div>
                <h4 className="font-medium text-foreground">Trading Bot</h4>
                <p className="text-sm text-muted-foreground">
                  {isRunning ? 'Active and trading' : 'Stopped'}
                </p>
              </div>
            </div>
            <Button 
              variant={isRunning ? "destructive" : "default"}
              onClick={setIsRunning}
            >
              {isRunning ? "Stop Bot" : "Start Bot"}
            </Button>
          </div>

          <div className="flex items-center justify-between p-4 rounded-lg border border-border">
            <div className="flex items-center gap-3">
              <Zap className={`h-5 w-5 ${isSimulation ? 'text-yellow-400' : 'text-muted-foreground'}`} />
              <div>
                <h4 className="font-medium text-foreground">Simulation Mode</h4>
                <p className="text-sm text-muted-foreground">
                  {isSimulation ? 'Paper trading - no real funds' : 'Live trading with real funds'}
                </p>
              </div>
            </div>
            <Switch checked={isSimulation} onCheckedChange={setIsSimulation} />
          </div>
        </CardContent>
      </Card>

      {/* Trading Parameters */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Settings className="h-5 w-5" />
            Trading Parameters
          </CardTitle>
          <CardDescription>
            Configure your bot's trading behavior
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="tradeAmount">Trade Amount (SOL)</Label>
              <div className="flex items-center gap-2">
                <DollarSign className="h-4 w-4 text-muted-foreground" />
                <input
                  type="number"
                  id="tradeAmount"
                  value={settings.tradeAmount}
                  onChange={(e) => setSettings({ ...settings, tradeAmount: parseFloat(e.target.value) })}
                  step="0.01"
                  min="0.01"
                  className="flex-1 px-3 py-2 bg-background border border-border rounded-md text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
                />
              </div>
            </div>

            <div className="space-y-2">
              <Label htmlFor="maxPosition">Max Position Size (SOL)</Label>
              <div className="flex items-center gap-2">
                <TrendingUp className="h-4 w-4 text-muted-foreground" />
                <input
                  type="number"
                  id="maxPosition"
                  value={settings.maxPositionSize}
                  onChange={(e) => setSettings({ ...settings, maxPositionSize: parseFloat(e.target.value) })}
                  step="0.1"
                  min="0.1"
                  className="flex-1 px-3 py-2 bg-background border border-border rounded-md text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
                />
              </div>
            </div>

            <div className="space-y-2">
              <Label htmlFor="minProfit">Min Profit Threshold (%)</Label>
              <div className="flex items-center gap-2">
                <BarChart3 className="h-4 w-4 text-muted-foreground" />
                <input
                  type="number"
                  id="minProfit"
                  value={settings.minProfitThreshold}
                  onChange={(e) => setSettings({ ...settings, minProfitThreshold: parseFloat(e.target.value) })}
                  step="0.1"
                  min="0.1"
                  className="flex-1 px-3 py-2 bg-background border border-border rounded-md text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
                />
              </div>
            </div>

            <div className="space-y-2">
              <Label htmlFor="maxSlippage">Max Slippage (%)</Label>
              <div className="flex items-center gap-2">
                <AlertTriangle className="h-4 w-4 text-muted-foreground" />
                <input
                  type="number"
                  id="maxSlippage"
                  value={settings.maxSlippage}
                  onChange={(e) => setSettings({ ...settings, maxSlippage: parseFloat(e.target.value) })}
                  step="0.5"
                  min="0.5"
                  max="50"
                  className="flex-1 px-3 py-2 bg-background border border-border rounded-md text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
                />
              </div>
            </div>
          </div>

          <div className="flex items-center justify-between p-4 rounded-lg border border-border">
            <div className="flex items-center gap-3">
              <Shield className="h-5 w-5 text-muted-foreground" />
              <div>
                <h4 className="font-medium text-foreground">Auto-Compound</h4>
                <p className="text-sm text-muted-foreground">
                  Automatically reinvest profits
                </p>
              </div>
            </div>
            <Switch 
              checked={settings.autoCompound} 
              onCheckedChange={(checked) => setSettings({ ...settings, autoCompound: checked })} 
            />
          </div>
        </CardContent>
      </Card>

      {/* Risk Management */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            Risk Management
          </CardTitle>
          <CardDescription>
            Configure stop loss and take profit levels
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="stopLoss">Stop Loss (%)</Label>
              <div className="flex items-center gap-2">
                <AlertTriangle className="h-4 w-4 text-red-400" />
                <input
                  type="number"
                  id="stopLoss"
                  value={settings.stopLoss}
                  onChange={(e) => setSettings({ ...settings, stopLoss: parseFloat(e.target.value) })}
                  step="1"
                  min="1"
                  max="50"
                  className="flex-1 px-3 py-2 bg-background border border-border rounded-md text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
                />
                <span className="text-sm text-muted-foreground">%</span>
              </div>
              <p className="text-xs text-muted-foreground">
                Sell when position drops by this %
              </p>
            </div>

            <div className="space-y-2">
              <Label htmlFor="takeProfit">Take Profit (%)</Label>
              <div className="flex items-center gap-2">
                <TrendingUp className="h-4 w-4 text-emerald-400" />
                <input
                  type="number"
                  id="takeProfit"
                  value={settings.takeProfit}
                  onChange={(e) => setSettings({ ...settings, takeProfit: parseFloat(e.target.value) })}
                  step="1"
                  min="1"
                  max="100"
                  className="flex-1 px-3 py-2 bg-background border border-border rounded-md text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
                />
                <span className="text-sm text-muted-foreground">%</span>
              </div>
              <p className="text-xs text-muted-foreground">
                Sell when position gains by this %
              </p>
            </div>
          </div>

          <div className="space-y-2">
            <Label>Risk Level</Label>
            <div className="grid grid-cols-3 gap-2">
              {['low', 'medium', 'high'].map((level) => (
                <Button
                  key={level}
                  variant={settings.riskLevel === level ? 'default' : 'outline'}
                  size="sm"
                  onClick={() => setSettings({ ...settings, riskLevel: level as any })}
                  className={`${
                    level === 'low' ? 'text-emerald-400' :
                    level === 'medium' ? 'text-yellow-400' :
                    'text-red-400'
                  }`}
                >
                  {level.charAt(0).toUpperCase() + level.slice(1)}
                </Button>
              ))}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Bot Stats */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <BarChart3 className="h-5 w-5" />
            Bot Statistics
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="text-center p-4 rounded-lg bg-muted/50">
              <div className="text-2xl font-bold text-foreground">0</div>
              <div className="text-sm text-muted-foreground">Total Trades</div>
            </div>
            <div className="text-center p-4 rounded-lg bg-muted/50">
              <div className="text-2xl font-bold text-emerald-400">0%</div>
              <div className="text-sm text-muted-foreground">Win Rate</div>
            </div>
            <div className="text-center p-4 rounded-lg bg-muted/50">
              <div className="text-2xl font-bold text-foreground">0.00</div>
              <div className="text-sm text-muted-foreground">Total PnL (SOL)</div>
            </div>
            <div className="text-center p-4 rounded-lg bg-muted/50">
              <div className="text-2xl font-bold text-foreground">0.00</div>
              <div className="text-sm text-muted-foreground">Total PnL ($)</div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* API Configuration */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Settings className="h-5 w-5" />
            API Configuration
          </CardTitle>
          <CardDescription>
            Configure API keys for real-time data
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="heliusKey">Helius API Key</Label>
            <input
              type="password"
              id="heliusKey"
              placeholder="Enter your Helius API key"
              className="w-full px-3 py-2 bg-background border border-border rounded-md text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary"
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="dexKey">DexScreener API Key (Optional)</Label>
            <input
              type="password"
              id="dexKey"
              placeholder="Enter your DexScreener API key"
              className="w-full px-3 py-2 bg-background border border-border rounded-md text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary"
            />
          </div>
          <Button variant="outline" className="w-full">
            Save API Configuration
          </Button>
        </CardContent>
      </Card>
    </div>
  )
}

import { useState } from 'react'