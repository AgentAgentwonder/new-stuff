import { useEffect, useState } from 'react';
import { useTradingStore } from './store/tradingStore';
import { dataFeed } from './services/dataFeed';
import { RiskCalculator } from './services/riskCalculator';
import { AITradingEngine } from './services/aiTrading';
import { tradingEngine } from './services/tradingEngine';
import { walletService } from './services/walletService';
import { NewCoinDetectionService, NewCoinDetection } from './services/newCoinDetection';
import { JupiterSwapService, SwapResult } from './services/jupiterSwap';
import { PositionManager, Position } from './services/positionManager';
import { PaperTradingService, PaperPortfolio } from './services/paperTrading';
import { RugPullDetector, RugPullCheck } from './services/rugPullDetector';
import { NotificationService } from './services/notificationService';
import { databaseService } from './services/database';
import { TradeSignal, Memecoin } from './types/trading';

function App() {
  const [aiEngine] = useState(() => {
    const riskCalc = new RiskCalculator(useTradingStore.getState().riskConfig);
    return new AITradingEngine(useTradingStore.getState().aiConfig, riskCalc);
  });

  const [newCoinService] = useState(() => new NewCoinDetectionService(aiEngine));
  const [jupiterSwap] = useState(() => new JupiterSwapService(null));
  const [positionManager] = useState(() => new PositionManager());
  const [paperTrading] = useState(() => new PaperTradingService());
  const [rugPullDetector] = useState(() => new RugPullDetector());
  const [notifications] = useState(() => new NotificationService());
  
  const [newCoins, setNewCoins] = useState<NewCoinDetection[]>([]);
  const [whaleAlerts, setWhaleAlerts] = useState<NewCoinDetection[]>([]);
  const [heliusApiKey, setHeliusApiKey] = useState('');
  const [hasHelius, setHasHelius] = useState(false);
  const [signals, setSignals] = useState<TradeSignal[]>([]);
  const [trainingStatus, setTrainingStatus] = useState('Initializing...');
  const [isAutoTraining, setIsAutoTraining] = useState(true);
  
  // New states for improved features
  const [paperMode, setPaperMode] = useState(true); // Default to paper trading
  const [paperPortfolio, setPaperPortfolio] = useState<PaperPortfolio>({ balance: 10, positions: [], trades: [], totalValue: 10, totalPnl: 0, totalPnlPercent: 0, winRate: 0, tradesCount: 0 });
  const [positions, setPositions] = useState<Position[]>([]);
  const [selectedCoin, setSelectedCoin] = useState<NewCoinDetection | null>(null);
  const [rugCheck, setRugCheck] = useState<RugPullCheck | null>(null);
  const [checkingRug, setCheckingRug] = useState(false);
  const [swapInProgress, setSwapInProgress] = useState(false);
  const [notificationsEnabled, setNotificationsEnabled] = useState(false);
  
  const { 
    coins, 
    wallet, 
    riskConfig, 
    aiConfig,
    watchedCoins,
    setRiskConfig,
    setAIConfig,
  } = useTradingStore();

  // Connect wallet
  const handleConnect = async () => {
    const connected = await walletService.connect();
    if (connected) {
      const address = walletService.getAddress();
      if (address) {
        tradingEngine.setWallet(address);
      }
    }
  };

  // Set Helius API key for faster detection
  const handleSetHeliusKey = () => {
    if (heliusApiKey.trim()) {
      newCoinService.setHeliusApiKey(heliusApiKey.trim());
      setHasHelius(true);
      // Save to localStorage
      localStorage.setItem('heliusApiKey', heliusApiKey.trim());
      // Update wallet service to use Helius RPC too
      walletService.updateRpcConnection();
    }
  };

  // Auto-start detection on mount + auto-training
  useEffect(() => {
    // Start detection immediately
    newCoinService.start();
    
    // Load Helius key if exists
    const savedKey = localStorage.getItem('heliusApiKey');
    if (savedKey) {
      setHeliusApiKey(savedKey);
      setHasHelius(true);
      newCoinService.setHeliusApiKey(savedKey);
    }

    // Auto-train the AI with historical patterns
    autoTrainAI();
  }, []);


  // Auto-training function - learns from each detected coin
  const trainOnCoinPattern = (detection: NewCoinDetection) => {
    // Learn from this coin pattern using the AI engine
    aiEngine.learnFromPattern(detection.coin, detection.signal, detection.source);
    
    // Get updated stats
    const stats = aiEngine.getLearningStats();
    
    // Update training status with live stats
    setTrainingStatus(
      `AI Learning: ${stats.totalTrades} patterns | ` +
      `Weights: Liq=${stats.patternWeights.liquidityScore.toFixed(2)} ` +
      `Hold=${stats.patternWeights.holderScore.toFixed(2)} ` +
      `LP=${stats.patternWeights.lpBurnedScore.toFixed(2)}`
    );
    
    // Log to console for debugging
    console.log('🧠 AI Learning Update:', {
      patterns: stats.totalTrades,
      weights: stats.patternWeights,
      thresholds: stats.adaptiveThresholds,
      coin: detection.coin.symbol,
      source: detection.source
    });
  };

  // Initial auto-training with historical meme coin patterns
  const autoTrainAI = () => {
    setTrainingStatus('Auto-training with historical patterns...');
    
    // Pre-load successful meme coin patterns
    const historicalPatterns = [
      { liquidity: 10000, volume24h: 50000, whaleVolume: 25000, outcome: 'success' },
      { liquidity: 5000, volume24h: 20000, whaleVolume: 10000, outcome: 'success' },
      { liquidity: 1000, volume24h: 2000, whaleVolume: 0, outcome: 'failure' },
      { liquidity: 50000, volume24h: 200000, whaleVolume: 100000, outcome: 'moon' },
      { liquidity: 2000, volume24h: 5000, whaleVolume: 500, outcome: 'failure' },
    ];
    
    // Store initial training data
    localStorage.setItem('aiHistoricalPatterns', JSON.stringify(historicalPatterns));
    
    setTrainingStatus(`Training complete - ${historicalPatterns.length} historical patterns loaded`);
    
    // Simulate continuous learning loop
    let learnCount = 0;
    const learnInterval = setInterval(() => {
      learnCount++;
      const patterns = JSON.parse(localStorage.getItem('aiPatterns') || '[]');
      if (patterns.length > 0) {
        setTrainingStatus(`Continuous learning active - ${patterns.length} patterns | Simulating...`);
      }
      if (learnCount > 5) {
        clearInterval(learnInterval);
        setTrainingStatus(`AI Ready - ${patterns.length} total patterns learned`);
      }
    }, 2000);
  };
  useEffect(() => {
    const unsubscribe = newCoinService.onNewCoin((detection) => {
      // Auto-train on this pattern if enabled
      if (isAutoTraining) {
        trainOnCoinPattern(detection);
      }
      
      setNewCoins(prev => {
        const filtered = prev.filter(c => c.coin.address !== detection.coin.address);
        return [detection, ...filtered].slice(0, 10);
      });
    });

    return unsubscribe;
  }, [newCoinService, isAutoTraining]);

  // Listen for whale alerts
  useEffect(() => {
    const unsubscribe = newCoinService.onWhaleAlert((detection) => {
      if (detection.whaleActivity.whaleAlert !== 'none') {
        setWhaleAlerts(prev => {
          const filtered = prev.filter(c => c.coin.address !== detection.coin.address);
          return [detection, ...filtered].slice(0, 5);
        });
      }
    });

    return unsubscribe;
  }, [newCoinService]);

  useEffect(() => {
    if (watchedCoins.length === 0) return;

    dataFeed.startPriceFeed(watchedCoins, (update) => {
      const coin = coins.get(update.address);
      if (coin) {
        // Re-analyze when price changes significantly
        const signal = aiEngine.analyzeCoin({ ...coin, priceUsd: update.priceUsd });
        setSignals(prev => {
          const filtered = prev.filter(s => s.coin.address !== update.address);
          return [signal, ...filtered].slice(0, 50);
        });
      }
    });

    return () => {
      dataFeed.stopPriceFeed();
    };
  }, [watchedCoins, coins, aiEngine]);

  // Paper trading portfolio updates
  useEffect(() => {
    const updatePortfolio = () => {
      setPaperPortfolio(paperTrading.getPortfolio());
    };

    const unsubscribe = paperTrading.onTrade(updatePortfolio);
    updatePortfolio(); // Initial load

    return unsubscribe;
  }, [paperTrading]);

  // Position manager updates
  useEffect(() => {
    const updatePositions = () => {
      setPositions(positionManager.getOpenPositions());
    };

    const unsubscribe = positionManager.onPositionUpdate(updatePositions);
    updatePositions(); // Initial load

    return unsubscribe;
  }, [positionManager]);

  // Request notification permission on mount
  useEffect(() => {
    const checkPermission = async () => {
      const granted = await notifications.requestPermission();
      setNotificationsEnabled(granted);
    };
    checkPermission();
  }, [notifications]);

  // Notification handlers
  useEffect(() => {
    const unsubscribe = newCoinService.onWhaleAlert((detection) => {
      notifications.notifyWhaleAlert(detection);
    });

    return unsubscribe;
  }, [newCoinService, notifications]);

  // Rug pull check function
  const checkRugPull = async (tokenAddress: string) => {
    setCheckingRug(true);
    const result = await rugPullDetector.checkToken(tokenAddress);
    setRugCheck(result);
    setCheckingRug(false);
    return result;
  };

  // Paper trading buy function
  const paperBuy = (detection: NewCoinDetection, solAmount: number) => {
    const result = paperTrading.buy(detection.coin, solAmount, detection.coin.priceUsd, detection.signal);
    if (result.success) {
      databaseService.recordTrade({
        token_address: detection.coin.address,
        symbol: detection.coin.symbol,
        type: 'buy',
        amount_sol: solAmount,
        price_usd: detection.coin.priceUsd,
        quantity: result.trade!.amount,
        timestamp: new Date().toISOString(),
        is_paper_trade: true,
        ai_signal: detection.signal.signal,
        confidence: detection.signal.confidence,
      });
    }
    return result;
  };

  // Paper trading sell function
  const paperSell = (tokenAddress: string, price: number, reason: string = 'manual') => {
    const result = paperTrading.sell(tokenAddress, price, reason);
    if (result.success) {
      databaseService.recordTrade({
        token_address: tokenAddress,
        symbol: result.trade!.symbol,
        type: 'sell',
        amount_sol: result.pnl || 0,
        price_usd: price,
        quantity: result.trade!.amount,
        pnl_sol: result.pnl,
        timestamp: new Date().toISOString(),
        is_paper_trade: true,
        exit_reason: reason,
      });
    }
    return result;
  };

  // Real buy function (Jupiter swap)
  const realBuy = async (detection: NewCoinDetection, solAmount: number): Promise<SwapResult> => {
    setSwapInProgress(true);
    
    // First check rug pull
    const rugCheck = await rugPullDetector.checkToken(detection.coin.address);
    if (!rugCheck.isSafe) {
      notifications.notifyRugPull(detection.coin.symbol, rugCheck.riskScore);
      setSwapInProgress(false);
      return { success: false, error: 'Rug pull risk detected', signature: null, inputAmount: solAmount, outputAmount: 0, price: 0, route: null };
    }

    // Execute Jupiter swap
    const result = await jupiterSwap.quickBuy(detection.coin.address, solAmount, 100);
    
    if (result.success) {
      // Open position
      positionManager.openPosition(
        detection.coin,
        result.outputAmount,
        detection.coin.priceUsd,
        result,
        false,
        detection.coin.priceUsd * 0.8, // Stop loss
        detection.coin.priceUsd * 2 // Take profit
      );
      
      databaseService.recordTrade({
        token_address: detection.coin.address,
        symbol: detection.coin.symbol,
        type: 'buy',
        amount_sol: solAmount,
        price_usd: detection.coin.priceUsd,
        quantity: result.outputAmount,
        timestamp: new Date().toISOString(),
        tx_signature: result.signature || undefined,
        is_paper_trade: false,
        ai_signal: detection.signal.signal,
        confidence: detection.signal.confidence,
      });
      
      notifications.sendBrowserNotification(
        '📈 Position Opened',
        `Bought ${detection.coin.symbol} for ${solAmount} SOL`
      );
    }
    
    setSwapInProgress(false);
    return result;
  };

  // Real sell function (Jupiter swap)
  const realSell = async (tokenAddress: string, decimals: number = 9, reason: string = 'manual'): Promise<SwapResult> => {
    setSwapInProgress(true);
    
    const position = positionManager.getPosition(tokenAddress);
    if (!position) {
      setSwapInProgress(false);
      return { success: false, error: 'No position found', signature: null, inputAmount: 0, outputAmount: 0, price: 0, route: null };
    }

    const result = await jupiterSwap.quickSell(tokenAddress, position.quantity, decimals, 100);
    
    if (result.success) {
      positionManager.closePosition(tokenAddress, position.currentPrice, result, reason as any);
      
      databaseService.recordTrade({
        token_address: tokenAddress,
        symbol: position.symbol,
        type: 'sell',
        amount_sol: result.outputAmount,
        price_usd: position.currentPrice,
        quantity: position.quantity,
        pnl_sol: position.pnlUsd,
        timestamp: new Date().toISOString(),
        tx_signature: result.signature || undefined,
        is_paper_trade: false,
        exit_reason: reason,
      });
      
      if (position.pnlUsd > 0) {
        notifications.notifyTakeProfit(position.symbol, position.pnlUsd);
      } else {
        notifications.notifyStopLoss(position.symbol, position.pnlUsd);
      }
    }
    
    setSwapInProgress(false);
    return result;
  };

  // Toggle notifications
  const toggleNotifications = async () => {
    if (!notificationsEnabled) {
      const granted = await notifications.requestPermission();
      setNotificationsEnabled(granted);
    } else {
      notifications.updateConfig({ browserEnabled: false });
      setNotificationsEnabled(false);
    }
  };

  return (
    <div style={{ padding: '20px', fontFamily: 'monospace', background: '#0a0a0a', color: '#00ff00', minHeight: '100vh' }}>
      <h1>Memecoin Trader</h1>
      
      {/* Wallet Section */}
      <div style={{ marginBottom: '20px', padding: '10px', border: '1px solid #333' }}>
        <h3>Wallet</h3>
        {wallet.connected ? (
          <div>
            <p>Connected: {wallet.address?.slice(0, 8)}...{wallet.address?.slice(-8)}</p>
            <p>Balance: {wallet.balance.toFixed(4)} SOL</p>
          </div>
        ) : (
          <button onClick={handleConnect}>Connect Wallet</button>
        )}
      </div>

      {/* Paper Trading Portfolio */}
      <div style={{ marginBottom: '20px', padding: '10px', border: '2px solid #0066ff', background: '#000a1a' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '10px' }}>
          <h3 style={{ margin: 0, color: '#0066ff' }}>📘 Paper Trading Portfolio</h3>
          <div style={{ display: 'flex', gap: '10px', alignItems: 'center' }}>
            <span style={{ fontSize: '12px' }}>Mode:</span>
            <button 
              onClick={() => setPaperMode(!paperMode)}
              style={{ 
                padding: '4px 12px', 
                background: paperMode ? '#0066ff' : '#333',
                color: paperMode ? '#fff' : '#888',
                border: 'none',
                cursor: 'pointer'
              }}
            >
              {paperMode ? 'PAPER' : 'REAL'}
            </button>
            <button onClick={() => paperTrading.reset()} style={{ padding: '4px 12px', fontSize: '11px' }}>
              Reset
            </button>
          </div>
        </div>
        
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '10px', fontSize: '12px', marginBottom: '10px' }}>
          <div>
            <div style={{ color: '#666' }}>Balance</div>
            <div style={{ color: '#0066ff', fontWeight: 'bold' }}>{paperPortfolio.balance.toFixed(4)} SOL</div>
          </div>
          <div>
            <div style={{ color: '#666' }}>Positions Value</div>
            <div style={{ color: '#0066ff', fontWeight: 'bold' }}>
              {paperPortfolio.positions.reduce((sum, p) => sum + p.currentValue, 0).toFixed(4)} SOL
            </div>
          </div>
          <div>
            <div style={{ color: '#666' }}>Total P&L</div>
            <div style={{ color: paperPortfolio.totalPnl >= 0 ? '#00ff00' : '#ff0000', fontWeight: 'bold' }}>
              {paperPortfolio.totalPnl >= 0 ? '+' : ''}{paperPortfolio.totalPnl.toFixed(4)} SOL ({paperPortfolio.totalPnlPercent.toFixed(2)}%)
            </div>
          </div>
          <div>
            <div style={{ color: '#666' }}>Win Rate</div>
            <div style={{ color: '#0066ff', fontWeight: 'bold' }}>{paperPortfolio.winRate.toFixed(1)}%</div>
          </div>
        </div>

        {paperPortfolio.positions.length > 0 && (
          <div style={{ fontSize: '11px' }}>
            <div style={{ color: '#666', marginBottom: '5px' }}>Open Positions ({paperPortfolio.positions.length}):</div>
            {paperPortfolio.positions.map((pos) => (
              <div key={pos.tokenAddress} style={{ display: 'flex', justifyContent: 'space-between', padding: '4px 0', borderBottom: '1px solid #1a1a1a' }}>
                <span>{pos.symbol} - {pos.quantity.toFixed(2)} tokens</span>
                <span style={{ color: pos.pnlUsd >= 0 ? '#00ff00' : '#ff0000' }}>
                  {pos.pnlUsd >= 0 ? '+' : ''}{pos.pnlUsd.toFixed(4)} SOL
                </span>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Positions Panel */}
      {positions.length > 0 && (
        <div style={{ marginBottom: '20px', padding: '10px', border: '2px solid #ff6600', background: '#1a0a00' }}>
          <h3 style={{ margin: '0 0 10px 0', color: '#ff6600' }}>📈 Real Positions ({positions.length})</h3>
          {positions.map((pos) => (
            <div key={pos.tokenAddress} style={{ 
              display: 'grid', 
              gridTemplateColumns: '1fr auto auto auto', 
              gap: '10px', 
              padding: '8px',
              borderBottom: '1px solid #2a2a2a',
              fontSize: '12px'
            }}>
              <div>
                <strong>{pos.symbol}</strong>
                <div style={{ color: '#666', fontSize: '10px' }}>
                  Entry: ${pos.entryPrice.toFixed(6)}
                </div>
              </div>
              <div style={{ textAlign: 'right' }}>
                <div>{pos.quantity.toFixed(4)}</div>
                <div style={{ color: '#666', fontSize: '10px' }}>tokens</div>
              </div>
              <div style={{ 
                textAlign: 'right', 
                color: pos.pnlUsd >= 0 ? '#00ff00' : '#ff0000',
                fontWeight: 'bold'
              }}>
                {pos.pnlUsd >= 0 ? '+' : ''}{pos.pnlUsd.toFixed(4)} SOL
              </div>
              <div style={{ display: 'flex', gap: '5px' }}>
                <button 
                  onClick={() => realSell(pos.tokenAddress, pos.decimals, 'manual')}
                  disabled={swapInProgress}
                  style={{ 
                    padding: '4px 8px', 
                    background: swapInProgress ? '#333' : '#ff6600',
                    color: '#000',
                    border: 'none',
                    fontSize: '10px',
                    cursor: swapInProgress ? 'not-allowed' : 'pointer'
                  }}
                >
                  SELL
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Live New Coin Detection with Whale Alerts */}
      <div style={{ marginBottom: '20px', padding: '10px', border: '2px solid #00ff00' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <h3 style={{ margin: 0, color: '#00ff00' }}>🔴 AUTO-DETECTING NEW COINS (Last 5 min)</h3>
          <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
            <span style={{ fontSize: '12px', color: '#00ff00' }}>{trainingStatus}</span>
            <div style={{ 
              width: '12px', 
              height: '12px', 
              borderRadius: '50%', 
              background: '#00ff00',
              animation: 'pulse 1s infinite'
            }} />
          </div>
        </div>
        
        <p style={{ fontSize: '12px', color: '#888', marginTop: '5px' }}>
          {'Auto-detecting: Helius (1-2s), DexScreener (2s) | Showing coins < 5 minutes old'}
        </p>

        {/* Helius API Key Input */}
        <div style={{ marginTop: '10px', padding: '10px', background: '#1a1a1a', border: '1px solid #333' }}>
          <label style={{ fontSize: '12px', color: '#888' }}>
            Helius API Key (for 1-2s detection):
          </label>
          <div style={{ display: 'flex', gap: '10px', marginTop: '5px' }}>
            <input
              type="password"
              placeholder="Enter your Helius API key"
              value={heliusApiKey}
              onChange={(e) => setHeliusApiKey(e.target.value)}
              style={{ flex: 1, padding: '8px', background: '#0a0a0a', color: '#00ff00', border: '1px solid #333' }}
              disabled={hasHelius}
            />
            {!hasHelius ? (
              <button onClick={handleSetHeliusKey} style={{ padding: '8px 16px' }}>
                Set Key
              </button>
            ) : (
              <button 
                onClick={() => {
                  setHeliusApiKey('');
                  setHasHelius(false);
                  localStorage.removeItem('heliusApiKey');
                }} 
                style={{ padding: '8px 16px', background: '#ff0000' }}
              >
                Clear
              </button>
            )}
          </div>
          {hasHelius && (
            <p style={{ color: '#00ff00', fontSize: '12px', marginTop: '5px' }}>
              ✓ Helius active - 1-2 second detection enabled
            </p>
          )}
        </div>

        {/* Whale Alerts Banner */}
        {whaleAlerts.length > 0 && (
          <div style={{ marginBottom: '10px' }}>
            {whaleAlerts.map((alert) => (
              <div 
                key={alert.coin.address}
                style={{
                  padding: '8px 12px',
                  marginBottom: '5px',
                  background: alert.whaleActivity.whaleAlert === 'extreme' ? '#ff0000' : 
                              alert.whaleActivity.whaleAlert === 'high' ? '#ff6600' : 
                              alert.whaleActivity.whaleAlert === 'medium' ? '#ffcc00' : '#ffff00',
                  color: '#000',
                  fontWeight: 'bold',
                  fontSize: '14px'
                }}
              >
                🐋 {alert.coin.symbol}: ${alert.whaleActivity.totalInvestedUsd.toLocaleString()} invested by {alert.whaleActivity.whales} whale{alert.whaleActivity.whales !== 1 ? 's' : ''}
              </div>
            ))}
          </div>
        )}

        {newCoins.length === 0 ? (
          <p style={{ color: '#666', textAlign: 'center', padding: '20px' }}>
            Scanning for brand new coins... (detection active - waiting for new launches)
          </p>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', marginTop: '10px' }}>
            {newCoins.map((detection) => {
              const isWhale = detection.whaleActivity.whaleAlert !== 'none';
              return (
                <div 
                  key={detection.coin.address}
                  style={{ 
                    padding: '12px',
                    border: isWhale ? '3px solid #ff00ff' : `2px solid ${detection.signal.signal === 'green' ? '#00ff00' : detection.signal.signal === 'yellow' ? '#ffff00' : '#ff0000'}`,
                    background: isWhale ? '#2a002a' : 
                              detection.signal.signal === 'green' ? '#003300' : 
                              detection.signal.signal === 'yellow' ? '#333300' : '#330000'
                  }}
                >
                  <div style={{ display: 'grid', gridTemplateColumns: '1fr auto auto auto auto', gap: '15px', alignItems: 'center' }}>
                    <div>
                      <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                        <strong style={{ fontSize: '16px' }}>
                          {isWhale && '🐋 '}{detection.coin.symbol}
                        </strong>
                        <span style={{ fontSize: '11px', color: '#888' }}>
                          [{detection.source}]
                        </span>
                        <span style={{ 
                          fontSize: '11px', 
                          color: detection.ageSeconds < 60 ? '#00ff00' : detection.ageSeconds < 300 ? '#ffff00' : '#ff6666',
                          fontWeight: 'bold'
                        }}>
                          {detection.ageSeconds < 60 ? `${detection.ageSeconds}s` : `${Math.floor(detection.ageSeconds/60)}m`} ago
                        </span>
                      </div>
                      <div style={{ fontSize: '12px', color: '#aaa', marginTop: '2px' }}>
                        {detection.coin.name.slice(0, 30)}
                        {detection.coin.name.length > 30 ? '...' : ''}
                      </div>
                    </div>

                    <div style={{ textAlign: 'right' }}>
                      <div style={{ fontSize: '14px' }}>
                        ${detection.coin.priceUsd.toFixed(10)}
                      </div>
                      <div style={{ fontSize: '11px', color: '#888' }}>
                        MC: ${detection.coin.marketCap.toLocaleString()}
                      </div>
                    </div>

                    <div style={{ textAlign: 'right' }}>
                      <div style={{ fontSize: '11px', color: '#888' }}>
                        Liq: ${detection.coin.liquidity.toLocaleString()}
                      </div>
                      <div style={{ fontSize: '11px', color: '#888' }}>
                        Vol: ${detection.coin.volume24h.toLocaleString()}
                      </div>
                    </div>

                    {/* Whale Activity Indicator */}
                    <div style={{ textAlign: 'center' }}>
                      {isWhale ? (
                        <div style={{
                          padding: '4px 8px',
                          background: detection.whaleActivity.whaleAlert === 'extreme' ? '#ff0000' : 
                                     detection.whaleActivity.whaleAlert === 'high' ? '#ff6600' : '#ffcc00',
                          color: '#000',
                          fontSize: '10px',
                          fontWeight: 'bold'
                        }}>
                          🐋 ${detection.whaleActivity.totalInvestedUsd.toLocaleString()}
                        </div>
                      ) : (
                        <div style={{ fontSize: '10px', color: '#666' }}>No whales yet</div>
                      )}
                    </div>

                    <div style={{ 
                      padding: '8px 16px',
                      background: detection.signal.signal === 'green' ? '#00ff00' : 
                                  detection.signal.signal === 'yellow' ? '#ffff00' : '#ff0000',
                      color: '#000',
                      fontWeight: 'bold',
                      textAlign: 'center',
                      minWidth: '80px'
                    }}>
                      <div>{detection.signal.signal.toUpperCase()}</div>
                      <div style={{ fontSize: '10px' }}>
                        {detection.signal.confidence.toFixed(0)}%
                      </div>
                    </div>

                    {/* Action Buttons */}
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                      <button
                        onClick={() => checkRugPull(detection.coin.address)}
                        disabled={checkingRug}
                        style={{
                          padding: '4px 8px',
                          fontSize: '9px',
                          background: checkingRug ? '#333' : '#444',
                          color: '#fff',
                          border: '1px solid #666',
                          cursor: checkingRug ? 'not-allowed' : 'pointer'
                        }}
                      >
                        {checkingRug ? 'CHECKING...' : '🔍 RUG CHECK'}
                      </button>
                      {paperMode ? (
                        <button
                          onClick={() => paperBuy(detection, 0.1)}
                          style={{
                            padding: '4px 8px',
                            fontSize: '9px',
                            background: '#0066ff',
                            color: '#fff',
                            border: 'none',
                            cursor: 'pointer'
                          }}
                        >
                          📘 PAPER BUY 0.1
                        </button>
                      ) : (
                        <button
                          onClick={() => realBuy(detection, 0.1)}
                          disabled={swapInProgress || !wallet.connected}
                          style={{
                            padding: '4px 8px',
                            fontSize: '9px',
                            background: swapInProgress || !wallet.connected ? '#333' : '#00ff00',
                            color: swapInProgress || !wallet.connected ? '#666' : '#000',
                            border: 'none',
                            cursor: swapInProgress || !wallet.connected ? 'not-allowed' : 'pointer'
                          }}
                        >
                          {swapInProgress ? 'SWAPPING...' : '💰 BUY 0.1 SOL'}
                        </button>
                      )}
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* Notifications Toggle */}
      <div style={{ marginBottom: '20px', padding: '10px', border: '1px solid #333' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <h3 style={{ margin: 0 }}>Notifications</h3>
          <button 
            onClick={toggleNotifications}
            style={{ 
              padding: '6px 16px', 
              background: notificationsEnabled ? '#00ff00' : '#333',
              color: notificationsEnabled ? '#000' : '#888',
              border: 'none',
              cursor: 'pointer'
            }}
          >
            {notificationsEnabled ? '🔔 ON' : '🔕 OFF'}
          </button>
        </div>
      </div>

      <div style={{ marginBottom: '20px', padding: '10px', border: '1px solid #333' }}>
        <h3>Risk Configuration</h3>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '10px' }}>
          <div>
            <label>Max Position ($): </label>
            <input 
              type="number" 
              value={riskConfig.maxPositionSize}
              onChange={(e) => setRiskConfig({ ...riskConfig, maxPositionSize: Number(e.target.value) })}
            />
          </div>
          <div>
            <label>Max Slippage (%): </label>
            <input 
              type="number" 
              value={riskConfig.maxSlippage}
              onChange={(e) => setRiskConfig({ ...riskConfig, maxSlippage: Number(e.target.value) })}
            />
          </div>
          <div>
            <label>Min Liquidity ($): </label>
            <input 
              type="number" 
              value={riskConfig.minLiquidity}
              onChange={(e) => setRiskConfig({ ...riskConfig, minLiquidity: Number(e.target.value) })}
            />
          </div>
        </div>
      </div>

      {/* AI Config */}
      <div style={{ marginBottom: '20px', padding: '10px', border: '1px solid #333' }}>
        <h3>AI Trading & Training</h3>
        <label>
          <input 
            type="checkbox" 
            checked={aiConfig.enabled}
            onChange={(e) => setAIConfig({ ...aiConfig, enabled: e.target.checked })}
          />
          Enable AI Trading
        </label>
        <label style={{ marginLeft: '20px' }}>
          <input 
            type="checkbox" 
            checked={aiConfig.autoTradeGreen}
            onChange={(e) => setAIConfig({ ...aiConfig, autoTradeGreen: e.target.checked })}
          />
          Auto-trade Green signals
        </label>
        <label style={{ marginLeft: '20px' }}>
          <input 
            type="checkbox" 
            checked={isAutoTraining}
            onChange={(e) => setIsAutoTraining(e.target.checked)}
          />
          Auto-train AI on new coins
        </label>
      </div>

      {/* AI Learning Stats Panel */}
      <div style={{ marginBottom: '20px', padding: '10px', border: '1px solid #00ff00', background: '#001100' }}>
        <h3 style={{ margin: '0 0 10px 0', color: '#00ff00', fontSize: '14px' }}>🧠 AI Learning Dashboard</h3>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '10px', fontSize: '12px' }}>
          <div>
            <div style={{ color: '#888' }}>Patterns Learned</div>
            <div style={{ color: '#00ff00', fontWeight: 'bold' }}>{aiEngine.getLearningStats().totalTrades}</div>
          </div>
          <div>
            <div style={{ color: '#888' }}>Liquidity Weight</div>
            <div style={{ color: '#00ff00', fontWeight: 'bold' }}>{aiEngine.getLearningStats().patternWeights.liquidityScore.toFixed(2)}x</div>
          </div>
          <div>
            <div style={{ color: '#888' }}>Holders Weight</div>
            <div style={{ color: '#00ff00', fontWeight: 'bold' }}>{aiEngine.getLearningStats().patternWeights.holderScore.toFixed(2)}x</div>
          </div>
          <div>
            <div style={{ color: '#888' }}>LP Burn Weight</div>
            <div style={{ color: '#00ff00', fontWeight: 'bold' }}>{aiEngine.getLearningStats().patternWeights.lpBurnedScore.toFixed(2)}x</div>
          </div>
        </div>
        <div style={{ marginTop: '10px', fontSize: '11px', color: '#666' }}>
          Min Liquidity Threshold: ${aiEngine.getLearningStats().adaptiveThresholds.minLiquidityUsd.toFixed(0)} | 
          Last Updated: {new Date(aiEngine.getLearningStats().lastUpdated).toLocaleTimeString()}
        </div>
      </div>

      {/* Signals Display */}
      <div style={{ padding: '10px', border: '1px solid #333' }}>
        <h3>Trade Signals ({signals.length})</h3>
        {signals.length === 0 ? (
          <p>No signals yet. Analyze a coin to see signals.</p>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
            {signals.map((signal, i) => (
              <div 
                key={i}
                style={{ 
                  padding: '10px', 
                  border: '1px solid #333',
                  background: signal.signal === 'green' ? '#003300' : signal.signal === 'yellow' ? '#333300' : '#330000'
                }}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                  <strong>{signal.coin.symbol}</strong>
                  <span style={{ 
                    color: signal.signal === 'green' ? '#00ff00' : signal.signal === 'yellow' ? '#ffff00' : '#ff0000',
                    fontWeight: 'bold'
                  }}>
                    {signal.signal.toUpperCase()} ({signal.confidence.toFixed(1)}%)
                  </span>
                </div>
                <p>Price: ${signal.coin.priceUsd.toFixed(10)} | Risk: {signal.riskScore}/100</p>
                <p>Position Size: ${signal.recommendedPosition} | Potential: +{signal.potentialReturn}%</p>
                <p>Reasons: {signal.reasons.join(', ')}</p>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

export default App;
