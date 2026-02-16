import { NewCoinDetection } from './newCoinDetection';

export interface NotificationConfig {
  browserEnabled: boolean;
  whaleAlertsEnabled: boolean;
  newCoinAlertsEnabled: boolean;
  minWhaleAlertLevel: 'low' | 'medium' | 'high' | 'extreme';
  soundEnabled: boolean;
}

export class NotificationService {
  private config: NotificationConfig;
  private hasPermission: boolean = false;

  constructor() {
    this.config = {
      browserEnabled: false,
      whaleAlertsEnabled: true,
      newCoinAlertsEnabled: true,
      minWhaleAlertLevel: 'medium',
      soundEnabled: true,
    };
    this.loadConfig();
  }

  // Request browser notification permission
  async requestPermission(): Promise<boolean> {
    if (!('Notification' in window)) {
      console.log('Browser notifications not supported');
      return false;
    }

    const permission = await Notification.requestPermission();
    this.hasPermission = permission === 'granted';
    
    if (this.hasPermission) {
      this.config.browserEnabled = true;
      this.saveConfig();
    }

    return this.hasPermission;
  }

  // Send browser notification
  sendBrowserNotification(title: string, body: string, icon?: string): void {
    if (!this.config.browserEnabled || !this.hasPermission) return;

    try {
      new Notification(title, {
        body,
        icon: icon || '/icon.png',
        badge: '/badge.png',
        tag: title,
        requireInteraction: true,
      });
    } catch (error) {
      console.error('Failed to send notification:', error);
    }
  }

  // Play alert sound
  playSound(type: 'new_coin' | 'whale' | 'alert' | 'error'): void {
    if (!this.config.soundEnabled) return;

    const sounds: Record<string, string> = {
      new_coin: 'https://assets.mixkit.co/active_storage/sfx/2869/2869-preview.mp3',
      whale: 'https://assets.mixkit.co/active_storage/sfx/2866/2866-preview.mp3',
      alert: 'https://assets.mixkit.co/active_storage/sfx/2868/2868-preview.mp3',
      error: 'https://assets.mixkit.co/active_storage/sfx/2861/2861-preview.mp3',
    };

    try {
      const audio = new Audio(sounds[type]);
      audio.volume = 0.5;
      audio.play().catch(() => {
        // Ignore autoplay errors
      });
    } catch (error) {
      console.error('Failed to play sound:', error);
    }
  }

  // Notify new coin detection
  notifyNewCoin(detection: NewCoinDetection): void {
    if (!this.config.newCoinAlertsEnabled) return;

    const { coin, signal, source } = detection;
    
    // Only notify for high-confidence green signals
    if (signal.signal !== 'green' || signal.confidence < 70) return;

    this.sendBrowserNotification(
      '🚀 New Memecoin Detected!',
      `${coin.symbol} - ${coin.name} (${source}) | Signal: ${signal.signal.toUpperCase()} ${signal.confidence.toFixed(0)}%`
    );

    this.playSound('new_coin');
  }

  // Notify whale activity
  notifyWhaleAlert(detection: NewCoinDetection): void {
    if (!this.config.whaleAlertsEnabled) return;

    const { whaleActivity, coin } = detection;
    const alertLevel = whaleActivity.whaleAlert;

    // Check minimum alert level
    const levels = ['none', 'low', 'medium', 'high', 'extreme'];
    const configIndex = levels.indexOf(this.config.minWhaleAlertLevel);
    const alertIndex = levels.indexOf(alertLevel);

    if (alertIndex < configIndex) return;

    const emoji = alertLevel === 'extreme' ? '🚨' : alertLevel === 'high' ? '🐋' : '💰';
    
    this.sendBrowserNotification(
      `${emoji} Whale Alert: ${coin.symbol}`,
      `${whaleActivity.whales} whales invested $${whaleActivity.totalInvestedUsd.toLocaleString()}`,
    );

    this.playSound('whale');
  }

  // Notify stop loss triggered
  notifyStopLoss(symbol: string, loss: number): void {
    this.sendBrowserNotification(
      '🔴 Stop Loss Triggered',
      `${symbol} position closed with $${Math.abs(loss).toFixed(2)} loss`
    );
    this.playSound('alert');
  }

  // Notify take profit
  notifyTakeProfit(symbol: string, profit: number): void {
    this.sendBrowserNotification(
      '🟢 Take Profit Hit',
      `${symbol} position closed with $${profit.toFixed(2)} profit`
    );
    this.playSound('new_coin');
  }

  // Notify rug pull detected
  notifyRugPull(symbol: string, riskScore: number): void {
    this.sendBrowserNotification(
      '⚠️ RUG PULL WARNING',
      `${symbol} has ${riskScore}/100 risk score - DO NOT BUY`
    );
    this.playSound('error');
  }

  // Update config
  updateConfig(config: Partial<NotificationConfig>): void {
    this.config = { ...this.config, ...config };
    this.saveConfig();
  }

  getConfig(): NotificationConfig {
    return { ...this.config };
  }

  private saveConfig(): void {
    localStorage.setItem('notificationConfig', JSON.stringify(this.config));
  }

  private loadConfig(): void {
    try {
      const saved = localStorage.getItem('notificationConfig');
      if (saved) {
        this.config = { ...this.config, ...JSON.parse(saved) };
      }
    } catch (error) {
      console.error('Failed to load notification config:', error);
    }
  }
}

export const notificationService = new NotificationService();
