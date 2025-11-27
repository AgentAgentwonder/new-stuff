import { HashRouter, Navigate, Route, Routes } from 'react-router-dom';
import { lazy, Suspense } from 'react';
import { AppErrorBoundary } from '@/components';
import { AccessibilityProvider } from '@/components/providers/AccessibilityProvider';
import ClientLayout from '@/layouts/ClientLayout';

// Lazy load all pages to prevent blocking at app startup
const Dashboard = lazy(() => import('@/pages/Dashboard').catch(() => ({ default: () => <LoadingFallback /> })));
const Portfolio = lazy(() => import('@/pages/Portfolio').catch(() => ({ default: () => <LoadingFallback /> })));
const Trading = lazy(() => import('@/pages/Trading').catch(() => ({ default: () => <LoadingFallback /> })));
const AnalyticsPage = lazy(() => import('@/pages/analytics/page'));
const AIPage = lazy(() => import('@/pages/ai/page'));
const AIAssistantPage = lazy(() => import('@/pages/ai/assistant/page'));
const AIPredictionsPage = lazy(() => import('@/pages/ai/predictions/page'));
const AIRiskPage = lazy(() => import('@/pages/ai/risk/page'));
const MarketPage = lazy(() => import('@/pages/market/page'));
const MarketTrendsPage = lazy(() => import('@/pages/market/trends/page'));
const FreshCoinsPage = lazy(() => import('@/pages/market/fresh-coins/page'));
const FreshBuyersPage = lazy(() => import('@/pages/market/fresh-buyers/page'));
const MarketSentimentPage = lazy(() => import('@/pages/market/sentiment/page'));
const MarketWatchlistPage = lazy(() => import('@/pages/market/watchlist/page'));
const GovernancePage = lazy(() => import('@/pages/governance/page'));
const GovernanceProposalsPage = lazy(() => import('@/pages/governance/proposals/page'));
const GovernanceAlertsPage = lazy(() => import('@/pages/governance/alerts/page'));
const GovernanceVoicePage = lazy(() => import('@/pages/governance/voice/page'));
const PortfolioHoldingsPage = lazy(() => import('@/pages/portfolio/holdings/page'));
const PortfolioPositionsPage = lazy(() => import('@/pages/portfolio/positions/page'));
const PortfolioPerformancePage = lazy(() => import('@/pages/portfolio/performance/page'));
const PortfolioHistoryPage = lazy(() => import('@/pages/portfolio/history/page'));
const PortfolioWalletsPage = lazy(() => import('@/pages/portfolio/wallets/page'));
const TradingSpotPage = lazy(() => import('@/pages/trading/spot/page'));
const TradingFuturesPage = lazy(() => import('@/pages/trading/futures/page'));
const TradingP2PPage = lazy(() => import('@/pages/trading/p2p/page'));
const TradingPaperPage = lazy(() => import('@/pages/trading/paper/page'));
const TradingOrderbookPage = lazy(() => import('@/pages/trading/orderbook/page'));
const LearningPage = lazy(() => import('@/pages/learning/page'));
const SettingsPage = lazy(() => import('@/pages/settings/page'));
const WorkspacesPage = lazy(() => import('@/pages/workspaces/page'));
const CoinDetailPage = lazy(() => import('@/pages/coin/[symbol]/page'));

const LoadingFallback = () => <div style={{ padding: '20px', color: 'white' }}>Loading...</div>;

function App() {
  return (
    <AppErrorBoundary>
      <AccessibilityProvider>
        <HashRouter>
          <ClientLayout>
            <Suspense fallback={<LoadingFallback />}>
              <Routes>
              <Route path="/" element={<Navigate to="/settings" replace />} />

              {/* Main Pages */}
              <Route path="/dashboard" element={<Dashboard />} />
              <Route path="/portfolio" element={<Portfolio />} />
              <Route path="/trading" element={<Trading />} />
              <Route path="/analytics" element={<AnalyticsPage />} />

              {/* AI Pages */}
              <Route path="/ai" element={<AIPage />} />
              <Route path="/ai/assistant" element={<AIAssistantPage />} />
              <Route path="/ai/predictions" element={<AIPredictionsPage />} />
              <Route path="/ai/risk" element={<AIRiskPage />} />

              {/* Market Pages */}
              <Route path="/market" element={<MarketPage />} />
              <Route path="/market/trends" element={<MarketTrendsPage />} />
              <Route path="/market/fresh-coins" element={<FreshCoinsPage />} />
              <Route path="/market/fresh-buyers" element={<FreshBuyersPage />} />
              <Route path="/market/sentiment" element={<MarketSentimentPage />} />
              <Route path="/market/watchlist" element={<MarketWatchlistPage />} />

              {/* Governance Pages */}
              <Route path="/governance" element={<GovernancePage />} />
              <Route path="/governance/proposals" element={<GovernanceProposalsPage />} />
              <Route path="/governance/alerts" element={<GovernanceAlertsPage />} />
              <Route path="/governance/voice" element={<GovernanceVoicePage />} />

              {/* Portfolio Sub-pages */}
              <Route path="/portfolio/holdings" element={<PortfolioHoldingsPage />} />
              <Route path="/portfolio/positions" element={<PortfolioPositionsPage />} />
              <Route path="/portfolio/performance" element={<PortfolioPerformancePage />} />
              <Route path="/portfolio/history" element={<PortfolioHistoryPage />} />
              <Route path="/portfolio/wallets" element={<PortfolioWalletsPage />} />

              {/* Trading Sub-pages */}
              <Route path="/trading/spot" element={<TradingSpotPage />} />
              <Route path="/trading/futures" element={<TradingFuturesPage />} />
              <Route path="/trading/p2p" element={<TradingP2PPage />} />
              <Route path="/trading/paper" element={<TradingPaperPage />} />
              <Route path="/trading/orderbook" element={<TradingOrderbookPage />} />

              {/* Other Pages */}
              <Route path="/learning" element={<LearningPage />} />
              <Route path="/settings" element={<SettingsPage />} />
              <Route path="/workspaces" element={<WorkspacesPage />} />
              <Route path="/coin/:symbol" element={<CoinDetailPage />} />

              {/* Catch-all - redirect to dashboard */}
              <Route path="*" element={<Navigate to="/dashboard" replace />} />
              </Routes>
            </Suspense>
          </ClientLayout>
        </HashRouter>
      </AccessibilityProvider>
    </AppErrorBoundary>
  );
}

export default App;
