import type React from 'react';

import { Link, useLocation } from 'react-router-dom';
import { useState } from 'react';

interface SidebarProps {
  isOpen: boolean;
}

interface MenuItem {
  label: string;
  href?: string;
  icon: React.ReactNode;
  submenu?: MenuItem[];
}

export default function Sidebar({ isOpen }: SidebarProps) {
  const location = useLocation();
  const pathname = location.pathname;
  const [expandedMenus, setExpandedMenus] = useState<string[]>([]);

  const toggleSubmenu = (label: string) => {
    setExpandedMenus(prev =>
      prev.includes(label) ? prev.filter(m => m !== label) : [...prev, label]
    );
  };

  const menuItems: MenuItem[] = [
    {
      label: 'Dashboard',
      href: '/dashboard',
      icon: <span className="text-lg">📊</span>,
    },
    {
      label: 'Trading',
      href: '/trading',
      icon: <span className="text-lg">📈</span>,
      submenu: [
        { label: 'Spot Trading', href: '/trading/spot', icon: <span>▸</span> },
        { label: 'Futures', href: '/trading/futures', icon: <span>▸</span> },
        { label: 'Paper Trading', href: '/trading/paper', icon: <span>▸</span> },
        { label: 'Order Book', href: '/trading/orderbook', icon: <span>▸</span> },
        { label: 'Contract Risk', href: '/trading/contract-risk', icon: <span>▸</span> },
        { label: 'P2P Marketplace', href: '/trading/p2p', icon: <span>▸</span> },
      ],
    },
    {
      label: 'Portfolio',
      href: '/portfolio',
      icon: <span className="text-lg">💼</span>,
      submenu: [
        { label: 'Holdings', href: '/portfolio/holdings', icon: <span>▸</span> },
        { label: 'Positions', href: '/portfolio/positions', icon: <span>▸</span> },
        { label: 'Performance', href: '/portfolio/performance', icon: <span>▸</span> },
        { label: 'Trading History', href: '/portfolio/history', icon: <span>▸</span> },
        { label: 'Wallets', href: '/portfolio/wallets', icon: <span>▸</span> },
      ],
    },
    {
      label: 'Market Surveillance',
      href: '/market',
      icon: <span className="text-lg">👁️</span>,
      submenu: [
        { label: 'Market Trends', href: '/market/trends', icon: <span>▸</span> },
        { label: 'Fresh Coins', href: '/market/fresh-coins', icon: <span>▸</span> },
        { label: 'Fresh Buyers', href: '/market/fresh-buyers', icon: <span>▸</span> },
        { label: 'Sentiment', href: '/market/sentiment', icon: <span>▸</span> },
        { label: 'Watchlist', href: '/market/watchlist', icon: <span>▸</span> },
      ],
    },
    {
      label: 'AI Analysis',
      href: '/ai',
      icon: <span className="text-lg">🧠</span>,
      submenu: [
        { label: 'Predictions', href: '/ai/predictions', icon: <span>▸</span> },
        { label: 'AI Assistant', href: '/ai/assistant', icon: <span>▸</span> },
        { label: 'Risk Scores', href: '/ai/risk', icon: <span>▸</span> },
      ],
    },
    {
      label: 'Governance & Alerts',
      href: '/governance',
      icon: <span className="text-lg">🔔</span>,
      submenu: [
        { label: 'Proposals', href: '/governance/proposals', icon: <span>▸</span> },
        { label: 'Alerts', href: '/governance/alerts', icon: <span>▸</span> },
        { label: 'Voice Control', href: '/governance/voice', icon: <span>▸</span> },
      ],
    },
  ];

  const bottomMenuItems: MenuItem[] = [
    {
      label: 'Workspaces',
      href: '/workspaces',
      icon: <span className="text-lg">👥</span>,
    },
    {
      label: 'Learning',
      href: '/learning',
      icon: <span className="text-lg">📚</span>,
    },
    {
      label: 'Settings',
      href: '/settings',
      icon: <span className="text-lg">⚙️</span>,
    },
  ];

  const isMenuItemActive = (item: MenuItem) => {
    return pathname === item.href || item.submenu?.some(sub => pathname.startsWith(sub.href!));
  };

  return (
    <aside
      className={`bg-sidebar text-sidebar-foreground border-r border-sidebar-border transition-all duration-300 ease-out overflow-hidden flex flex-col ${
        isOpen ? 'w-64' : 'w-0'
      }`}
    >
      {/* Top padding */}
      <div className="h-16 border-b border-sidebar-border flex items-center px-6">
        <span className="font-bold text-lg tracking-wide">Menu</span>
      </div>

      {/* Main menu items */}
      <nav className="flex-1 overflow-y-auto py-4">
        <ul className="space-y-1 px-3">
          {menuItems.map(item => (
            <li key={item.label}>
              <div
                className={`flex items-center gap-3 px-3 py-2 rounded-lg transition-colors ${
                  isMenuItemActive(item)
                    ? 'bg-sidebar-primary text-sidebar-primary-foreground'
                    : 'text-sidebar-foreground hover:bg-sidebar-accent/50'
                } ${item.submenu ? 'cursor-pointer' : ''}`}
                onClick={() => item.submenu && toggleSubmenu(item.label)}
              >
                {item.submenu ? (
                  <>
                    {item.icon}
                    <span className="text-sm font-medium flex-1">{item.label}</span>
                    <span
                      className={`transition-transform inline-block ${
                        expandedMenus.includes(item.label) ? 'rotate-180' : ''
                      }`}
                    >
                      ▼
                    </span>
                  </>
                ) : (
                  <>
                    <Link to={item.href!} className="flex items-center gap-3 flex-1">
                      {item.icon}
                      <span className="text-sm font-medium">{item.label}</span>
                    </Link>
                  </>
                )}
              </div>
              {item.submenu && expandedMenus.includes(item.label) && (
                <ul className="ml-6 mt-2 space-y-1">
                  {item.submenu.map(subitem => (
                    <li key={subitem.href}>
                      <Link
                        to={subitem.href!}
                        className={`flex items-center gap-2 px-3 py-2 rounded text-xs transition-colors ${
                          pathname === subitem.href
                            ? 'bg-sidebar-primary/60 text-sidebar-primary-foreground'
                            : 'text-sidebar-foreground/80 hover:bg-sidebar-accent/40'
                        }`}
                      >
                        {subitem.icon}
                        {subitem.label}
                      </Link>
                    </li>
                  ))}
                </ul>
              )}
            </li>
          ))}
        </ul>
      </nav>

      {/* Bottom menu items */}
      <div className="border-t border-sidebar-border p-4 space-y-2">
        <ul className="space-y-1">
          {bottomMenuItems.map(item => (
            <li key={item.label}>
              <Link
                to={item.href!}
                className={`flex items-center gap-3 px-3 py-2 rounded-lg transition-colors text-sm ${
                  pathname === item.href
                    ? 'bg-sidebar-primary text-sidebar-primary-foreground'
                    : 'text-sidebar-foreground hover:bg-sidebar-accent/50'
                }`}
              >
                {item.icon}
                {item.label}
              </Link>
            </li>
          ))}
        </ul>
        <button className="w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sidebar-foreground hover:bg-destructive/20 hover:text-destructive transition-colors text-sm mt-4">
          <span>🚪</span>
          Logout
        </button>
      </div>
    </aside>
  );
}
