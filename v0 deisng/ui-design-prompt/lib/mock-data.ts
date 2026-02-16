export interface CandleData {
  time: number
  open: number
  high: number
  low: number
  close: number
  volume: number
}

export interface MemecoinToken {
  id: string
  symbol: string
  name: string
  price: number
  marketCap: number
  liquidity: number
  holders: number
  buyTax: number
  sellTax: number
  riskLevel: "low" | "medium" | "high"
  age: number // seconds since launch
  launchedAt: number // timestamp ms
  change24h: number
  isNew: boolean
  whaleAlert: boolean
  volume24h: number
}

export interface Trade {
  id: number
  type: "buy" | "sell"
  token: string
  amount: number
  price: number
  total: number
  time: string
  status: "filled" | "pending" | "failed"
}

export interface Holding {
  token: string
  symbol: string
  amount: number
  avgPrice: number
  currentPrice: number
  value: number
  pnl: number
  pnlPercent: number
}

// Generate OHLC candlestick data
export function generateCandleData(basePrice: number, count: number, intervalSec: number = 60): CandleData[] {
  const candles: CandleData[] = []
  let currentPrice = basePrice
  const now = Math.floor(Date.now() / 1000)

  for (let i = 0; i < count; i++) {
    const volatility = (Math.random() - 0.5) * 0.06 * currentPrice
    const open = currentPrice
    const close = currentPrice + volatility
    const high = Math.max(open, close) * (1 + Math.random() * 0.02)
    const low = Math.min(open, close) * (1 - Math.random() * 0.02)
    const volume = Math.floor(50000 + Math.random() * 500000)

    candles.push({
      time: now - (count - i) * intervalSec,
      open: Number(open.toFixed(6)),
      high: Number(high.toFixed(6)),
      low: Number(low.toFixed(6)),
      close: Number(close.toFixed(6)),
      volume,
    })

    currentPrice = close
  }

  return candles
}

// Generate a new price tick
export function generateNewPrice(currentPrice: number) {
  const randomPercentage = (Math.random() - 0.5) * 6
  const newPrice = currentPrice * (1 + randomPercentage / 100)
  const finalPrice = Math.max(newPrice, currentPrice * 0.5)
  const percentageChange = Math.abs(((finalPrice - currentPrice) / currentPrice) * 100)
  const direction = finalPrice >= currentPrice ? "up" : "down"

  return {
    newPrice: finalPrice,
    newChange: percentageChange,
    direction: direction as "up" | "down",
  }
}

// Generate new candle from price update
export function generateNewCandle(lastCandle: CandleData, newPrice: number, intervalSec: number = 60): CandleData {
  return {
    time: lastCandle.time + intervalSec,
    open: lastCandle.close,
    high: Math.max(lastCandle.close, newPrice),
    low: Math.min(lastCandle.close, newPrice),
    close: Number(newPrice.toFixed(6)),
    volume: Math.floor(50000 + Math.random() * 500000),
  }
}

// Generate initial memecoin tokens with varying ages
export function generateMemecoins(): MemecoinToken[] {
  const names = [
    { symbol: "BONK", name: "Bonk" },
    { symbol: "WIF", name: "dogwifhat" },
    { symbol: "POPCAT", name: "Popcat" },
    { symbol: "MEW", name: "cat in a dogs world" },
    { symbol: "BOME", name: "BOOK OF MEME" },
    { symbol: "MYRO", name: "Myro" },
    { symbol: "SLERF", name: "Slerf" },
    { symbol: "WEN", name: "Wen" },
    { symbol: "SAMO", name: "Samoyedcoin" },
    { symbol: "PONKE", name: "Ponke" },
    { symbol: "MUMU", name: "Mumu the Bull" },
    { symbol: "GIGA", name: "Gigachad" },
    { symbol: "PENG", name: "Peng" },
    { symbol: "TREMP", name: "Tremp" },
    { symbol: "BODEN", name: "Boden" },
  ]

  const now = Date.now()

  return names.map((t, i) => {
    const ageSeconds = i < 3 ? Math.floor(Math.random() * 30) + 5 : Math.floor(Math.random() * 7200) + 60
    const price = i < 5 ? Math.random() * 0.01 + 0.0001 : Math.random() * 2 + 0.001
    const mc = price * (Math.random() * 50000000 + 100000)
    const liq = mc * (Math.random() * 0.08 + 0.02)
    const holders = Math.floor(Math.random() * 8000) + 50
    const buyTax = Math.floor(Math.random() * 5)
    const sellTax = Math.floor(Math.random() * 8)
    const totalFee = buyTax + sellTax
    const risk: "low" | "medium" | "high" = totalFee <= 5 && holders > 200 && liq > 5000 ? "low" : totalFee <= 10 && holders > 50 ? "medium" : "high"

    return {
      id: `${t.symbol}-${i}`,
      symbol: t.symbol,
      name: t.name,
      price,
      marketCap: mc,
      liquidity: liq,
      holders,
      buyTax,
      sellTax,
      riskLevel: risk,
      age: ageSeconds,
      launchedAt: now - ageSeconds * 1000,
      change24h: (Math.random() - 0.4) * 60,
      isNew: ageSeconds < 60,
      whaleAlert: Math.random() > 0.75,
      volume24h: Math.floor(Math.random() * 500000) + 1000,
    }
  })
}

// Pre-seeded sample trades
export const sampleTrades: Trade[] = [
  {
    id: 1,
    type: "buy",
    token: "BONK",
    amount: 0.5,
    price: 0.000042,
    total: 0.5,
    time: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
    status: "filled",
  },
  {
    id: 2,
    type: "sell",
    token: "WIF",
    amount: 0.2,
    price: 2.34,
    total: 0.468,
    time: new Date(Date.now() - 1 * 60 * 60 * 1000).toISOString(),
    status: "filled",
  },
  {
    id: 3,
    type: "buy",
    token: "POPCAT",
    amount: 0.3,
    price: 0.72,
    total: 0.216,
    time: new Date(Date.now() - 30 * 60 * 1000).toISOString(),
    status: "filled",
  },
]

// Sample holdings
export const sampleHoldings: Holding[] = [
  { token: "Bonk", symbol: "BONK", amount: 12500000, avgPrice: 0.000038, currentPrice: 0.000045, value: 562.5, pnl: 87.5, pnlPercent: 18.4 },
  { token: "dogwifhat", symbol: "WIF", amount: 150, avgPrice: 2.1, currentPrice: 2.34, value: 351, pnl: 36, pnlPercent: 11.4 },
  { token: "Popcat", symbol: "POPCAT", amount: 800, avgPrice: 0.55, currentPrice: 0.72, value: 576, pnl: 136, pnlPercent: 30.9 },
]
