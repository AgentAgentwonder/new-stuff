"use client"

import type React from "react"
import { useState } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Switch } from "@/components/ui/switch"
import { Label } from "@/components/ui/label"
import { Input } from "@/components/ui/input"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { AlertCircle, Wallet, Key, Bot, Filter } from "lucide-react"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"

interface BotSettingsProps {
  isRunning: boolean
  setIsRunning: React.Dispatch<React.SetStateAction<boolean>>
  isSimulation: boolean
  setIsSimulation: React.Dispatch<React.SetStateAction<boolean>>
}

export default function BotSettings({ isRunning, setIsRunning, isSimulation, setIsSimulation }: BotSettingsProps) {
  const [strategy, setStrategy] = useState("simple")
  const [buyThreshold, setBuyThreshold] = useState("0.5")
  const [sellThreshold, setSellThreshold] = useState("0.5")
  const [maxPosition, setMaxPosition] = useState("1.0")
  const [heliusKey, setHeliusKey] = useState("")
  const [jupiterKey, setJupiterKey] = useState("")
  const [dexScreenerKey, setDexScreenerKey] = useState("")
  const [minMC, setMinMC] = useState("10000")
  const [minLiq, setMinLiq] = useState("5000")
  const [minHolders, setMinHolders] = useState("50")
  const [maxFees, setMaxFees] = useState("10")
  const [maxAge, setMaxAge] = useState("24")

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
      {/* Wallet */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm flex items-center gap-2">
            <Wallet className="h-4 w-4 text-emerald-400" />
            Wallet
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <Button className="w-full bg-[#ab9ff2] hover:bg-[#9b8fe2] text-black font-semibold">
            Connect Phantom
          </Button>
          <div className="text-xs text-muted-foreground rounded bg-muted p-2.5 font-mono break-all">
            Not connected
          </div>
        </CardContent>
      </Card>

      {/* API Keys */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm flex items-center gap-2">
            <Key className="h-4 w-4 text-emerald-400" />
            API Keys
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="space-y-1.5">
            <Label htmlFor="helius" className="text-xs text-muted-foreground">Helius API Key</Label>
            <Input id="helius" type="password" value={heliusKey} onChange={(e) => setHeliusKey(e.target.value)} className="bg-muted border-border text-xs" placeholder="Enter key..." />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="jupiter" className="text-xs text-muted-foreground">Jupiter API Key</Label>
            <Input id="jupiter" type="password" value={jupiterKey} onChange={(e) => setJupiterKey(e.target.value)} className="bg-muted border-border text-xs" placeholder="Enter key..." />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="dex" className="text-xs text-muted-foreground">DexScreener API Key</Label>
            <Input id="dex" type="password" value={dexScreenerKey} onChange={(e) => setDexScreenerKey(e.target.value)} className="bg-muted border-border text-xs" placeholder="Enter key..." />
          </div>
        </CardContent>
      </Card>

      {/* Bot Config */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm flex items-center gap-2">
            <Bot className="h-4 w-4 text-emerald-400" />
            Bot Configuration
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="space-y-1.5">
            <Label htmlFor="strategy" className="text-xs text-muted-foreground">Strategy</Label>
            <Select value={strategy} onValueChange={setStrategy}>
              <SelectTrigger id="strategy" className="bg-muted border-border text-xs">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="simple">Simple Momentum</SelectItem>
                <SelectItem value="macd">MACD Crossover</SelectItem>
                <SelectItem value="rsi">RSI Oversold/Overbought</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="grid grid-cols-2 gap-3">
            <div className="space-y-1.5">
              <Label className="text-xs text-muted-foreground">Buy Threshold %</Label>
              <Input type="number" value={buyThreshold} onChange={(e) => setBuyThreshold(e.target.value)} className="bg-muted border-border text-xs" />
            </div>
            <div className="space-y-1.5">
              <Label className="text-xs text-muted-foreground">Sell Threshold %</Label>
              <Input type="number" value={sellThreshold} onChange={(e) => setSellThreshold(e.target.value)} className="bg-muted border-border text-xs" />
            </div>
          </div>
          <div className="space-y-1.5">
            <Label className="text-xs text-muted-foreground">Max Position (SOL)</Label>
            <Input type="number" value={maxPosition} onChange={(e) => setMaxPosition(e.target.value)} className="bg-muted border-border text-xs" />
          </div>
          <div className="flex items-center justify-between pt-1">
            <Label className="text-xs text-muted-foreground">Simulation Mode</Label>
            <Switch checked={isSimulation} onCheckedChange={setIsSimulation} />
          </div>

          {isSimulation && (
            <Alert className="border-emerald-500/30 bg-emerald-500/5">
              <AlertCircle className="h-3.5 w-3.5 text-emerald-400" />
              <AlertTitle className="text-foreground text-xs">Simulation Active</AlertTitle>
              <AlertDescription className="text-muted-foreground text-xs">
                No real trades will be executed.
              </AlertDescription>
            </Alert>
          )}

          <Button
            className={`w-full font-semibold text-sm ${
              isRunning ? "bg-red-600 hover:bg-red-700 text-white" : "bg-emerald-600 hover:bg-emerald-700 text-white"
            }`}
            onClick={() => setIsRunning((prev) => !prev)}
          >
            {isRunning ? "Stop Bot" : "Start Bot"}
          </Button>
        </CardContent>
      </Card>

      {/* Memecoin Filters */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm flex items-center gap-2">
            <Filter className="h-4 w-4 text-emerald-400" />
            Memecoin Filters
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="grid grid-cols-2 gap-3">
            <div className="space-y-1.5">
              <Label className="text-xs text-muted-foreground">Min Market Cap ($)</Label>
              <Input type="number" value={minMC} onChange={(e) => setMinMC(e.target.value)} className="bg-muted border-border text-xs" />
            </div>
            <div className="space-y-1.5">
              <Label className="text-xs text-muted-foreground">Min Liquidity ($)</Label>
              <Input type="number" value={minLiq} onChange={(e) => setMinLiq(e.target.value)} className="bg-muted border-border text-xs" />
            </div>
          </div>
          <div className="grid grid-cols-2 gap-3">
            <div className="space-y-1.5">
              <Label className="text-xs text-muted-foreground">Min Holders</Label>
              <Input type="number" value={minHolders} onChange={(e) => setMinHolders(e.target.value)} className="bg-muted border-border text-xs" />
            </div>
            <div className="space-y-1.5">
              <Label className="text-xs text-muted-foreground">Max Total Fees %</Label>
              <Input type="number" value={maxFees} onChange={(e) => setMaxFees(e.target.value)} className="bg-muted border-border text-xs" />
            </div>
          </div>
          <div className="space-y-1.5">
            <Label className="text-xs text-muted-foreground">Max Age (hours)</Label>
            <Input type="number" value={maxAge} onChange={(e) => setMaxAge(e.target.value)} className="bg-muted border-border text-xs" />
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
