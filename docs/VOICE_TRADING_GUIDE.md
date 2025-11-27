# Voice Trading Guide

## Overview

Voice Trading brings hands-free control to your trading experience. Issue commands, query your portfolio, set alerts, and execute trades using natural language voice commands. Designed with safety in mind, the system includes confirmations, multi-factor authentication for sensitive operations, and driving-mode optimizations for on-the-go trading.

## Features

### Core Capabilities

- **Voice-Activated Trading**: Buy and sell assets using voice commands
- **Portfolio Queries**: Check balances, positions, P&L with voice
- **Price Monitoring**: Query current prices and market data
- **Alert Management**: Create and manage price alerts via voice
- **AI Assistant Integration**: Ask questions and get market insights
- **Multi-Factor Authentication**: Secure sensitive trades with voice MFA
- **Risk Assessment**: Real-time risk scoring and warnings
- **Driving Mode**: Simplified, safety-focused voice interactions

### Safety Features

- **Confirmation Flow**: All trading commands require explicit confirmation
- **Risk Warnings**: Automatic detection of high-impact trades
- **MFA for Large Trades**: Trades over $1,000 require PIN authentication
- **Rate Limiting**: Prevents accidental rapid command execution
- **Error Recovery**: Clear error messages with recovery suggestions
- **Cancel Anytime**: Voice commands can be cancelled before execution

## Getting Started

### Prerequisites

- Modern browser with Web Speech API support (Chrome, Edge, Safari)
- Microphone access permission
- Active internet connection
- Trading account with available balance

### Enabling Voice Trading

1. Click the voice icon in the bottom-right corner of the app
2. Grant microphone permission when prompted
3. The icon will turn purple when listening
4. Speak your command clearly

### First Commands

Try these simple commands to get started:

- "What's the price of SOL?"
- "Show my balance"
- "Buy 1 SOL"
- "Alert me when Bitcoin goes above 50000"

## Voice Commands Reference

### Trading Commands

#### Buy Orders
```
"Buy [amount] [symbol]"
"Purchase [amount] [symbol]"
"Long [amount] [symbol]"
```

Examples:
- "Buy 10 SOL"
- "Purchase 0.5 Bitcoin"
- "Long 1000 BONK"

#### Sell Orders
```
"Sell [amount] [symbol]"
"Exit [amount] [symbol]"
"Short [amount] [symbol]"
```

Examples:
- "Sell 5 SOL"
- "Exit 0.25 ETH"

#### Cancel Orders
```
"Cancel order [order ID]"
"Abort order [order ID]"
```

### Portfolio Queries

```
"What's my balance?"
"Show my position in [symbol]"
"How much [symbol] do I have?"
"What's my profit?"
"Portfolio summary"
```

### Price Queries

```
"What's the price of [symbol]?"
"Price of [symbol]"
"How much is [symbol]?"
```

### Alert Commands

#### Create Alerts
```
"Alert me when [symbol] goes above [price]"
"Notify me if [symbol] goes below [price]"
```

Examples:
- "Alert me when SOL goes above 200"
- "Notify me if Bitcoin goes below 40000"

#### List Alerts
```
"List my alerts"
"Show my alerts"
"What alerts do I have?"
```

### Market Data

```
"Market summary"
"What's happening in the market?"
"Market overview"
```

## Confirmation Flow

### Standard Confirmation

When you issue a trading command:

1. **Command Parsed**: System identifies your intent
2. **Risk Assessment**: Automatic risk scoring
3. **Confirmation Prompt**: Voice summary of the trade
4. **User Response**: Say "yes" to confirm or "no" to cancel
5. **Execution**: Trade is executed if confirmed

### Multi-Factor Authentication (MFA)

For sensitive trades (>$1,000 or high risk):

1. **MFA Challenge**: System prompts for PIN
2. **PIN Entry**: Enter 4-6 digit PIN
3. **Verification**: System validates credentials
4. **Execution**: Trade proceeds if authenticated

## Driving Mode

Driving Mode optimizes voice trading for vehicle use with safety-focused features:

### Features

- **Simplified Responses**: Shorter, clearer audio feedback
- **Reduced Notifications**: Only critical updates spoken
- **Larger Touch Targets**: Easier manual interaction if needed
- **High-Contrast UI**: Better visibility in varied lighting

### Enabling Driving Mode

```javascript
// Start session in driving mode
voiceStore.startSession('en-US', true);
```

Or enable from Settings → Voice → Driving Mode

### Safety Guidelines for Driving Mode

⚠️ **IMPORTANT SAFETY WARNINGS**:

- Always prioritize road safety over trading
- Pull over for complex transactions
- Use voice commands only when safe to do so
- Disable voice trading in high-traffic situations
- Check your state/country laws regarding device use while driving

## Configuration

### Voice Settings

Access voice settings from the voice overlay or Settings page:

#### Speech Recognition
- **Language**: Select voice recognition language
- **Continuous Mode**: Keep listening after commands
- **Interim Results**: Show real-time transcription

#### Speech Synthesis
- **Voice**: Choose TTS voice
- **Rate**: Speed of speech (0.5 - 2.0)
- **Pitch**: Voice pitch (0.0 - 2.0)
- **Volume**: Audio volume (0.0 - 1.0)

#### Notifications
- **Frequency**: All / Important / Critical
- **Max Per Minute**: Rate limiting (1-10)
- **Driving Mode**: Enable/disable driving optimizations

### Example Configuration

```typescript
voiceStore.updateNotificationSettings({
  enabled: true,
  frequency: 'important',
  rate: 1.0,
  pitch: 1.0,
  volume: 0.8,
  drivingMode: false,
  maxNotificationsPerMinute: 5,
});
```

## Risk Management

### Risk Levels

Commands are automatically assigned risk levels:

- **Low**: Portfolio queries, price checks
- **Medium**: Alert creation, small trades
- **High**: Moderate-value trades, cancellations
- **Critical**: Large trades, high-impact operations

### Risk Indicators

- **Transaction Size**: Larger trades = higher risk
- **Price Impact**: Slippage and market impact
- **Volatility**: Current market volatility
- **Position Size**: Impact on portfolio

### Risk Warnings

The system warns about:
- High price impact (>2%)
- Large transaction amounts (>$10,000)
- High slippage tolerance (>1%)
- Volatile market conditions
- Unusual trading patterns

## Error Handling and Recovery

### Common Errors

#### "Command Not Recognized"
- Speak more clearly
- Use standard command patterns
- Check supported commands list

#### "Insufficient Balance"
- Verify available funds
- Reduce order size
- Check connected wallet

#### "MFA Failed"
- Re-enter correct PIN
- Check PIN configuration
- Contact support after 3 failed attempts

#### "Network Error"
- Check internet connection
- Retry command
- Switch to manual trading if persistent

### Error Recovery

All errors include:
- Clear error message (voice + visual)
- Suggested actions
- Option to retry or cancel
- Error code for troubleshooting

## API Integration

### Using Voice in Your Application

#### Initialize Voice Store

```typescript
import { useVoiceStore } from './store/voiceStore';

// Enable voice trading
voiceStore.setEnabled(true);

// Start a session
voiceStore.startSession('en-US', false);
```

#### Use Voice Hook

```typescript
import { useVoice } from './hooks/useVoice';

function MyComponent() {
  const {
    enabled,
    listening,
    currentCommand,
    speak,
    startListening,
    stopListening,
  } = useVoice({
    autoStart: true,
    onCommand: (intent) => console.log('Command:', intent),
    onError: (error) => console.error('Error:', error),
  });

  return (
    <button onClick={listening ? stopListening : startListening}>
      {listening ? 'Stop' : 'Start'} Listening
    </button>
  );
}
```

#### Voice Command Service

```typescript
import { voiceCommandService } from './utils/voiceCommandService';

// Parse voice input
const intent = voiceCommandService.parseIntent('buy 10 SOL');

// Execute command
const result = await voiceCommandService.executeCommand(intent);

// Calculate risk
const riskScore = voiceCommandService.calculateRiskScore(intent, marketData);
```

## UI Components

### Voice Trading Overlay

The main voice interface component:

```tsx
import { VoiceTradingOverlay } from './components/voice/VoiceTradingOverlay';

<VoiceTradingOverlay
  position="bottom-right"
  drivingMode={false}
/>
```

### Voice Confirmation Modal

Confirmation UI for voice commands:

```tsx
import { VoiceConfirmationModal } from './components/voice/VoiceConfirmationModal';

<VoiceConfirmationModal
  command={currentCommand}
  onConfirm={handleConfirm}
  onCancel={handleCancel}
  onMFASubmit={handleMFA}
/>
```

### Voice Notification Router

Handles speech synthesis for notifications:

```tsx
import { VoiceNotificationRouter } from './components/voice/VoiceNotificationRouter';

<VoiceNotificationRouter enabled={true} />
```

## Localization

### Supported Languages

- English (US) - en-US
- English (UK) - en-GB
- Spanish - es-ES
- French - fr-FR
- German - de-DE
- Japanese - ja-JP
- Chinese - zh-CN

### Setting Language

```typescript
voiceStore.updateRecognitionConfig({
  lang: 'es-ES'
});
```

### Adding Custom Language Patterns

```typescript
// In voiceCommandService.ts
const LOCALE_PATTERNS: Record<string, Record<VoiceIntentType, RegExp[]>> = {
  'es-ES': {
    trade_buy: [
      /comprar\s+(\d+(?:\.\d+)?)\s+(\w+)/i,
      // Add more patterns
    ],
    // Add more intents
  },
};
```

## Best Practices

### For Users

1. **Speak Clearly**: Use natural but clear pronunciation
2. **Avoid Background Noise**: Find a quiet environment
3. **Start Simple**: Begin with queries before trading
4. **Double-Check**: Always verify confirmations
5. **Use Limit Orders**: For precision, use manual entry
6. **Monitor Sessions**: End sessions when done

### For Developers

1. **Handle Errors Gracefully**: Provide clear recovery paths
2. **Test Edge Cases**: Unusual command variations
3. **Implement Rate Limiting**: Prevent abuse
4. **Log Everything**: Audit trail for debugging
5. **Secure MFA**: Never expose authentication details
6. **Optimize Latency**: Fast response times critical

## Security Considerations

### Authentication

- MFA required for trades >$1,000
- PIN stored securely (never in plain text)
- Session timeout after inactivity
- Device-specific authentication

### Privacy

- Voice data not stored permanently
- Transcripts deleted after processing
- No external voice processing (on-device only)
- User consent required for microphone access

### Safety

- Confirmation required for all trades
- Risk assessment before execution
- Rate limiting to prevent rapid trades
- Emergency cancel functionality

## Troubleshooting

### Voice Not Recognized

**Problem**: Microphone not working or commands not recognized

**Solutions**:
- Check browser permissions
- Verify microphone in system settings
- Test microphone in other apps
- Try different browser
- Reduce background noise

### Commands Not Executing

**Problem**: Commands confirmed but not executing

**Solutions**:
- Check internet connection
- Verify wallet connection
- Check available balance
- Review error messages
- Check backend status

### MFA Always Fails

**Problem**: Correct PIN rejected

**Solutions**:
- Reset PIN in security settings
- Check for caps lock/num lock
- Verify PIN hasn't changed
- Clear browser cache
- Contact support

### Poor Voice Quality

**Problem**: Choppy or delayed audio feedback

**Solutions**:
- Check system audio settings
- Close other audio applications
- Reduce speech rate in settings
- Test with different voice
- Check CPU/memory usage

## Support and Feedback

### Getting Help

- Documentation: [Link to docs]
- Community Forum: [Link to forum]
- Support Email: support@example.com
- Discord: [Link to Discord]

### Reporting Issues

When reporting voice trading issues, include:

1. Browser and version
2. Operating system
3. Exact command spoken
4. Error message received
5. Expected vs actual behavior
6. Steps to reproduce

### Feature Requests

We welcome suggestions for:
- New voice commands
- Additional languages
- UI improvements
- Safety enhancements
- Integration requests

## Changelog

### Version 1.0.0 (Current)

- Initial voice trading release
- Core command set (buy, sell, query, alert)
- MFA for sensitive trades
- Driving mode
- 7 language support
- Risk assessment engine
- Voice confirmation flow
- Error recovery system

## Roadmap

### Upcoming Features

- Voice-activated stop-loss/take-profit
- Advanced order types (trailing stop, OCO)
- Voice portfolio rebalancing
- Multi-language custom commands
- Voice-activated charts
- Social trading via voice
- Voice macros/shortcuts
- Biometric MFA

## Legal and Compliance

### Terms of Use

Voice trading is subject to the same terms as manual trading. Users are responsible for all trades executed via voice commands.

### Liability

Trading carries risk. Voice commands are convenience features and do not eliminate trading risks. Always verify trades before confirmation.

### Privacy Policy

Voice data is processed on-device when possible. See our full Privacy Policy for details on data handling.

### Regulatory Compliance

Voice trading complies with applicable financial regulations. Users must comply with local laws regarding voice-activated trading.

---

**Last Updated**: 2024
**Version**: 1.0.0
**Maintained by**: Voice Trading Team
