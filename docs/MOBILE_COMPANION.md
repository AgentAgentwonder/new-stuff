# Mobile Companion App Guide

## Overview

The Mobile Companion App provides authenticated, view-only access to your trading dashboard from mobile devices (iOS and Android). It includes push notifications, quick trades with biometric security, widgets, and Apple Watch support.

## Features

### 🔐 Authentication & Security

- **Biometric Authentication**: Secure login using Face ID, Touch ID, or fingerprint
- **Session Management**: Device-specific sessions with automatic expiry
- **View-Only Mode**: Read-only access to portfolios and market data
- **Device Management**: Register and manage multiple devices

### 📱 Core Features

1. **Dashboard Sync**: Real-time synchronization with desktop app
2. **Push Notifications**: Alerts for price changes, trades, and portfolio updates
3. **Quick Trades**: Execute trades with biometric confirmation
4. **Widgets**: Home screen widgets for market data and portfolio overview
5. **Apple Watch Support**: Glanceable portfolio data on your wrist

## Backend Architecture

### Mobile Module Structure

```
src-tauri/src/mobile/
├── mod.rs         # Module definitions and shared types
├── auth.rs        # Authentication and biometric security
├── push.rs        # Push notification management
├── sync.rs        # Data synchronization with reduced payloads
├── trades.rs      # Quick trade execution with safety checks
└── widgets.rs     # Widget data endpoints
```

### Key Components

#### 1. Mobile Authentication Manager

Handles device registration, biometric challenges, and session management.

**Device Registration:**
```rust
#[tauri::command]
pub async fn mobile_register_device(
    req: MobileAuthRequest,
    mobile_auth: SharedMobileAuthManager,
) -> Result<MobileDevice, String>
```

**Biometric Flow:**
1. Create challenge: `mobile_create_biometric_challenge(device_id)`
2. Verify signature: `mobile_verify_biometric(challenge_id, signature)`
3. Receive session token for authenticated requests

#### 2. Push Notification Manager

Manages notification queuing and delivery to mobile devices.

**Notification Categories:**
- `Alert`: Price alerts and threshold notifications
- `QuickTrade`: Trade execution confirmations
- `Portfolio`: Portfolio updates and performance
- `Watch`: Watchlist changes
- `System`: System messages and updates

#### 3. Mobile Sync Manager

Provides reduced data payloads optimized for mobile bandwidth.

**Reduced Data Types:**
- `ReducedMarketData`: Essential price and volume data
- `ReducedPortfolioData`: Portfolio summary with top holdings
- `ReducedAlert`: Active alerts and triggers

#### 4. Mobile Trade Engine

Executes quick trades with biometric confirmation and safety checks.

**Safety Features:**
- Maximum trade amount limits
- Order size restrictions
- Biometric signature verification
- Session authentication

#### 5. Widget Manager

Generates data for home screen widgets.

**Widget Types:**
- `PriceWatch`: Price monitoring for selected assets
- `PortfolioSummary`: Total value and 24h change
- `Alerts`: Active alerts and recent triggers
- `TopMovers`: Biggest gainers and losers
- `QuickActions`: Common actions shortcuts

## API Reference

### Authentication Commands

#### Register Device
```typescript
invoke('mobile_register_device', {
  req: {
    device_id: string,
    device_name: string,
    platform: 'ios' | 'android',
    biometric_public_key?: string
  }
}): Promise<MobileDevice>
```

#### Create Biometric Challenge
```typescript
invoke('mobile_create_biometric_challenge', {
  device_id: string
}): Promise<BiometricChallenge>
```

#### Verify Biometric
```typescript
invoke('mobile_verify_biometric', {
  challenge_id: string,
  signature: string
}): Promise<MobileAuthResponse>
```

#### Authenticate Session
```typescript
invoke('mobile_authenticate_session', {
  session_token: string
}): Promise<MobileSession>
```

### Push Notification Commands

#### Queue Notification
```typescript
invoke('mobile_queue_notification', {
  device_id: string,
  category: NotificationCategory,
  title: string,
  body: string,
  payload: any
}): Promise<PushNotification>
```

#### Get Pending Notifications
```typescript
invoke('mobile_get_pending_notifications', {
  device_id: string
}): Promise<PushNotification[]>
```

### Sync Commands

#### Sync Data
```typescript
invoke('mobile_sync_data', {
  device_id: string
}): Promise<MobileSyncData>
```

#### Get Last Sync Time
```typescript
invoke('mobile_get_last_sync', {
  device_id: string
}): Promise<number | null>
```

### Quick Trade Commands

#### Execute Quick Trade
```typescript
invoke('mobile_execute_quick_trade', {
  trade: {
    session_token: string,
    symbol: string,
    side: 'buy' | 'sell',
    amount: number,
    biometric_signature: string
  }
}): Promise<QuickTradeConfirmation>
```

#### Get Safety Checks
```typescript
invoke('mobile_safety_checks'): Promise<string[]>
```

### Widget Commands

#### Get Widget Data
```typescript
invoke('mobile_get_widget_data', {
  widget_type: WidgetType
}): Promise<WidgetData>
```

#### Get All Widgets
```typescript
invoke('mobile_get_all_widgets'): Promise<WidgetData[]>
```

## Mobile App Implementation

### Tauri Mobile Setup

1. **Add Mobile Platforms:**
```bash
# iOS
cargo tauri ios init

# Android
cargo tauri android init
```

2. **Configure tauri.conf.json:**
```json
{
  "tauri": {
    "bundle": {
      "iOS": {
        "frameworks": ["LocalAuthentication"]
      },
      "android": {
        "permissions": [
          "android.permission.USE_BIOMETRIC",
          "android.permission.INTERNET"
        ]
      }
    }
  }
}
```

### React Native Integration (Alternative)

If using React Native instead of Tauri Mobile:

1. **Install Dependencies:**
```bash
npm install @tauri-apps/api
npm install react-native-biometrics
npm install @notifee/react-native
```

2. **Connect to Backend:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

// Example: Register device
const registerDevice = async () => {
  const device = await invoke('mobile_register_device', {
    req: {
      device_id: getDeviceId(),
      device_name: getDeviceName(),
      platform: Platform.OS,
      biometric_public_key: await getBiometricKey()
    }
  });
  return device;
};
```

### Biometric Integration

#### iOS (Face ID / Touch ID)
```swift
import LocalAuthentication

func authenticateWithBiometrics() -> Promise<String> {
    let context = LAContext()
    var error: NSError?
    
    if context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &error) {
        context.evaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, 
                             localizedReason: "Authenticate to execute trade") { success, _ in
            if success {
                // Generate signature
                let signature = generateSignature()
                resolve(signature)
            }
        }
    }
}
```

#### Android (Fingerprint / Face Unlock)
```kotlin
import androidx.biometric.BiometricPrompt
import androidx.biometric.BiometricManager

fun authenticateWithBiometrics(): Promise<String> {
    val biometricPrompt = BiometricPrompt(activity, executor,
        object : BiometricPrompt.AuthenticationCallback() {
            override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
                val signature = generateSignature()
                resolve(signature)
            }
        })
    
    val promptInfo = BiometricPrompt.PromptInfo.Builder()
        .setTitle("Biometric Authentication")
        .setSubtitle("Authenticate to execute trade")
        .setNegativeButtonText("Cancel")
        .build()
    
    biometricPrompt.authenticate(promptInfo)
}
```

## Apple Watch Support

### WatchOS Extension Setup

1. **Create Watch App Target** in Xcode
2. **Add Complications:**
```swift
struct PortfolioComplication: TimelineProvider {
    func getTimeline(for complication: CLKComplication, withHandler handler: @escaping (CLKComplicationTimeline?) -> Void) {
        // Fetch portfolio data from backend
        let data = await fetchPortfolioData()
        
        // Create timeline entries
        let entry = createTimelineEntry(data: data)
        let timeline = CLKComplicationTimeline(entries: [entry], policy: .afterDate(Date(), paused: false))
        handler(timeline)
    }
}
```

3. **Watch App UI:**
```swift
struct WatchPortfolioView: View {
    @State private var portfolio: ReducedPortfolioData?
    
    var body: some View {
        VStack {
            Text("Portfolio")
                .font(.headline)
            
            if let portfolio = portfolio {
                Text(formatCurrency(portfolio.total_value))
                    .font(.title)
                
                Text("\(portfolio.total_change_pct, specifier: "%.2f")%")
                    .foregroundColor(portfolio.total_change_pct >= 0 ? .green : .red)
            }
        }
        .onAppear {
            loadPortfolioData()
        }
    }
    
    func loadPortfolioData() {
        // Fetch from backend via iOS app or direct API
    }
}
```

## Widgets Implementation

### iOS Widget (SwiftUI)

```swift
import WidgetKit
import SwiftUI

struct PortfolioWidget: Widget {
    var body: some WidgetConfiguration {
        StaticConfiguration(kind: "PortfolioWidget", provider: Provider()) { entry in
            PortfolioWidgetView(entry: entry)
        }
        .configurationDisplayName("Portfolio")
        .description("View your portfolio at a glance")
        .supportedFamilies([.systemSmall, .systemMedium])
    }
}

struct PortfolioWidgetView: View {
    var entry: Provider.Entry
    
    var body: some View {
        VStack(alignment: .leading) {
            Text("Portfolio")
                .font(.caption)
                .foregroundColor(.secondary)
            
            Text(entry.totalValue)
                .font(.title2)
                .bold()
            
            HStack {
                Text(entry.change24h)
                    .font(.subheadline)
                Image(systemName: entry.isPositive ? "arrow.up" : "arrow.down")
            }
            .foregroundColor(entry.isPositive ? .green : .red)
        }
        .padding()
    }
}
```

### Android Widget (Jetpack Glance)

```kotlin
class PortfolioWidget : GlanceAppWidget() {
    override suspend fun provideGlance(context: Context, id: GlanceId) {
        provideContent {
            PortfolioWidgetContent()
        }
    }
}

@Composable
fun PortfolioWidgetContent() {
    val portfolio = remember { fetchPortfolioData() }
    
    Column(
        modifier = GlanceModifier
            .fillMaxSize()
            .padding(16.dp)
    ) {
        Text(
            text = "Portfolio",
            style = TextStyle(fontSize = 12.sp, color = ColorProvider(Color.Gray))
        )
        
        Text(
            text = formatCurrency(portfolio.totalValue),
            style = TextStyle(fontSize = 20.sp, fontWeight = FontWeight.Bold)
        )
        
        Row {
            Text(
                text = "${portfolio.change24h}%",
                style = TextStyle(
                    fontSize = 14.sp,
                    color = if (portfolio.change24h >= 0) ColorProvider(Color.Green) else ColorProvider(Color.Red)
                )
            )
        }
    }
}
```

## Testing

### Mobile Test Setup

Create mobile-specific test directory:

```bash
mkdir -p mobile-tests
cd mobile-tests
```

#### Detox Tests (React Native)

1. **Install Detox:**
```bash
npm install --save-dev detox
```

2. **Configure `.detoxrc.json`:**
```json
{
  "testRunner": "jest",
  "runnerConfig": "mobile-tests/config.json",
  "apps": {
    "ios": {
      "type": "ios.app",
      "binaryPath": "ios/build/Build/Products/Debug-iphonesimulator/App.app",
      "build": "xcodebuild -workspace ios/App.xcworkspace -scheme App -configuration Debug -sdk iphonesimulator"
    },
    "android": {
      "type": "android.apk",
      "binaryPath": "android/app/build/outputs/apk/debug/app-debug.apk",
      "build": "cd android && ./gradlew assembleDebug"
    }
  }
}
```

3. **Example Test:**
```typescript
// mobile-tests/auth.test.ts
describe('Mobile Authentication', () => {
  beforeAll(async () => {
    await device.launchApp();
  });

  it('should register device successfully', async () => {
    await element(by.id('register-button')).tap();
    await expect(element(by.id('device-registered'))).toBeVisible();
  });

  it('should authenticate with biometrics', async () => {
    await element(by.id('biometric-auth')).tap();
    // Simulate biometric success
    await device.setBiometricEnrollment(true);
    await expect(element(by.id('authenticated'))).toBeVisible();
  });

  it('should execute quick trade with biometric', async () => {
    await element(by.id('quick-trade-btn')).tap();
    await element(by.id('trade-amount')).typeText('100');
    await element(by.id('confirm-trade')).tap();
    // Simulate biometric
    await device.setBiometricEnrollment(true);
    await expect(element(by.id('trade-confirmed'))).toBeVisible();
  });
});
```

#### Appium Tests (Cross-Platform)

1. **Install Appium:**
```bash
npm install --save-dev appium
npm install --save-dev webdriverio
```

2. **Example Test:**
```typescript
// mobile-tests/appium/sync.test.ts
import { remote } from 'webdriverio';

describe('Data Sync', () => {
  let driver;

  beforeAll(async () => {
    driver = await remote({
      capabilities: {
        platformName: 'iOS',
        'appium:deviceName': 'iPhone 14',
        'appium:app': '/path/to/app.app',
      }
    });
  });

  it('should sync data from backend', async () => {
    const syncButton = await driver.$('~sync-button');
    await syncButton.click();
    
    const syncStatus = await driver.$('~sync-status');
    await syncStatus.waitForDisplayed({ timeout: 5000 });
    
    const statusText = await syncStatus.getText();
    expect(statusText).toContain('Synced');
  });

  afterAll(async () => {
    await driver.deleteSession();
  });
});
```

### Running Tests

```bash
# Detox (iOS)
detox test --configuration ios

# Detox (Android)
detox test --configuration android

# Appium
npm run test:mobile
```

## Build Process

### iOS Build

1. **Prerequisites:**
   - Xcode 14+
   - Apple Developer Account
   - iOS device or simulator

2. **Build Steps:**
```bash
# Development build
cargo tauri ios dev

# Production build
cargo tauri ios build --release

# Generate IPA
xcodebuild -archivePath ./build/App.xcarchive \
           -exportArchive \
           -exportPath ./build \
           -exportOptionsPlist ExportOptions.plist
```

3. **TestFlight Distribution:**
```bash
# Upload to App Store Connect
xcrun altool --upload-app \
  --type ios \
  --file ./build/App.ipa \
  --username "your@email.com" \
  --password "@keychain:AC_PASSWORD"
```

### Android Build

1. **Prerequisites:**
   - Android Studio
   - Android SDK 28+
   - Java 11+

2. **Build Steps:**
```bash
# Development build
cargo tauri android dev

# Production build (APK)
cargo tauri android build --release

# Production build (AAB for Play Store)
cd src-tauri/gen/android
./gradlew bundleRelease
```

3. **Signing Configuration:**
```gradle
// android/app/build.gradle
android {
    signingConfigs {
        release {
            storeFile file("../keystore.jks")
            storePassword System.getenv("KEYSTORE_PASSWORD")
            keyAlias System.getenv("KEY_ALIAS")
            keyPassword System.getenv("KEY_PASSWORD")
        }
    }
}
```

4. **Play Store Upload:**
```bash
# Upload AAB
bundletool upload-bundle \
  --bundle=app-release.aab \
  --key=service-account.json \
  --package-name=com.eclipse.market
```

## Configuration

### Environment Variables

```env
# .env.mobile
MOBILE_API_ENDPOINT=https://api.yourdomain.com
MOBILE_WS_ENDPOINT=wss://ws.yourdomain.com
PUSH_NOTIFICATION_KEY=your_fcm_key
APPLE_TEAM_ID=your_team_id
```

### Feature Flags

```rust
// src-tauri/src/mobile/config.rs
pub struct MobileConfig {
    pub biometric_required: bool,
    pub quick_trades_enabled: bool,
    pub max_trade_amount: f64,
    pub session_timeout_seconds: i64,
    pub push_notifications_enabled: bool,
    pub widgets_enabled: bool,
    pub watch_support_enabled: bool,
}
```

## Security Best Practices

1. **Biometric Security:**
   - Always verify signatures server-side
   - Use secure enclave for key storage
   - Implement fallback authentication

2. **Session Management:**
   - Short-lived tokens (24 hours)
   - Refresh token rotation
   - Device-specific sessions

3. **Data Protection:**
   - Encrypt sensitive data at rest
   - Use TLS 1.3 for all network traffic
   - Implement certificate pinning

4. **Trade Safety:**
   - Require biometric for all trades
   - Enforce trade amount limits
   - Implement cooldown periods

## Troubleshooting

### Common Issues

**Issue: Biometric authentication fails**
- Verify device has biometric capability
- Check permissions in manifest
- Ensure secure enclave is available

**Issue: Push notifications not received**
- Verify FCM/APNs configuration
- Check device token registration
- Review notification permissions

**Issue: Sync fails**
- Check network connectivity
- Verify session token validity
- Review backend logs

## Support

For issues or questions:
- GitHub Issues: https://github.com/your-repo/issues
- Documentation: https://docs.yourdomain.com
- Discord: https://discord.gg/your-community
