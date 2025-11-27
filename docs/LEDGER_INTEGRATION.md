# Ledger Hardware Wallet Integration

This document describes the Ledger hardware wallet integration implementation for Eclipse Market Pro.

## Overview

The Ledger integration enables secure transaction signing using Ledger hardware wallets (Nano S, Nano S Plus, Nano X) via the WebHID protocol. This implementation provides full device management, address derivation, and transaction signing capabilities.

## Architecture

### Backend (Rust/Tauri)

#### Module: `src-tauri/src/wallet/ledger.rs`

The backend provides state management and validation for Ledger operations:

**Key Components:**

- `LedgerState`: Manages connected devices and active device state
- `LedgerDevice`: Device information structure
- Validation functions for derivation paths and transactions

**Tauri Commands:**

1. `ledger_register_device`: Register a new Ledger device
2. `ledger_list_devices`: List all registered devices
3. `ledger_get_device`: Get specific device by ID
4. `ledger_connect_device`: Mark device as connected
5. `ledger_disconnect_device`: Disconnect device
6. `ledger_update_device_address`: Update device address info
7. `ledger_validate_transaction`: Validate transaction before signing
8. `ledger_get_active_device`: Get currently active device
9. `ledger_remove_device`: Remove device from registry
10. `ledger_clear_devices`: Clear all devices

### Frontend (TypeScript/React)

#### Module: `src/utils/ledger.ts`

Provides the core Ledger wallet service using `@ledgerhq/hw-transport-webhid` and `@ledgerhq/hw-app-solana`.

**Key Classes:**

- `LedgerWalletService`: Main service for Ledger operations
  - `isSupported()`: Check WebHID support
  - `requestDevice()`: Request device access via WebHID
  - `getAddress(derivationPath, display)`: Get address from device
  - `signTransaction(transaction, derivationPath)`: Sign transaction
  - `signMessage(message, derivationPath)`: Sign off-chain message
  - `disconnect()`: Disconnect from device

**Helper Functions:**

- `listLedgerDevices()`: List registered devices
- `getLedgerDevice(deviceId)`: Get device by ID
- `getActiveLedgerDevice()`: Get active device
- `removeLedgerDevice(deviceId)`: Remove device
- `clearAllLedgerDevices()`: Clear all devices

#### Hook: `src/hooks/useLedger.ts`

React hook for managing Ledger connections:

```typescript
const {
  device,           // Current connected device
  devices,          // List of all devices
  isConnecting,     // Connection status
  isConnected,      // Connected state
  error,            // Error message
  connect,          // Connect to device
  disconnect,       // Disconnect from device
  getAddress,       // Get wallet address
  signTransaction,  // Sign transaction
  signMessage,      // Sign message
  refreshDevices,   // Refresh device list
  clearError,       // Clear error state
} = useLedger();
```

#### Component: `src/components/wallet/LedgerConnect.tsx`

UI component for connecting to Ledger devices with step-by-step wizard:

1. Instructions
2. Connecting
3. Address retrieval
4. Complete

## Usage

### Basic Connection

```typescript
import { useLedger } from '../hooks/useLedger';

function MyComponent() {
  const { connect, device, error } = useLedger();

  const handleConnect = async () => {
    const connectedDevice = await connect();
    if (connectedDevice) {
      console.log('Connected:', connectedDevice);
    }
  };

  return (
    <button onClick={handleConnect}>
      Connect Ledger
    </button>
  );
}
```

### Getting Address

```typescript
const { getAddress } = useLedger();

// Get address without displaying on device
const result = await getAddress("44'/501'/0'/0'", false);
console.log('Address:', result.address);

// Get address and verify on device
const verified = await getAddress("44'/501'/0'/0'", true);
```

### Signing Transactions

```typescript
import { Transaction } from '@solana/web3.js';
import { useLedger } from '../hooks/useLedger';

const { signTransaction } = useLedger();

// Create your transaction
const transaction = new Transaction().add(/* instructions */);

// Sign with Ledger
const signature = await signTransaction(transaction, "44'/501'/0'/0'");

// Add signature to transaction
transaction.addSignature(publicKey, signature);
```

### Using the LedgerConnect Component

```typescript
import LedgerConnect from '../components/wallet/LedgerConnect';

function MyPage() {
  const [showLedger, setShowLedger] = useState(false);

  return (
    <>
      <button onClick={() => setShowLedger(true)}>
        Connect Ledger
      </button>
      
      {showLedger && (
        <LedgerConnect
          onClose={() => setShowLedger(false)}
          onConnected={(address, publicKey) => {
            console.log('Connected:', address);
            setShowLedger(false);
          }}
        />
      )}
    </>
  );
}
```

## Error Handling

The integration provides comprehensive error handling:

### Error Types

- `WEBHID_NOT_SUPPORTED`: Browser doesn't support WebHID
- `USER_CANCELLED`: User cancelled device selection
- `APP_NOT_OPENED`: Solana app not opened on device
- `WRONG_APP`: Wrong app opened on device
- `USER_REJECTED`: User rejected on device
- `NOT_CONNECTED`: Device not connected
- `CONNECTION_FAILED`: Failed to connect
- `GET_ADDRESS_FAILED`: Failed to get address
- `SIGN_FAILED`: Failed to sign transaction
- `INVALID_DATA`: Invalid transaction data

### Error Handling Example

```typescript
const { connect, error, clearError } = useLedger();

const handleConnect = async () => {
  clearError();
  const device = await connect();
  
  if (!device && error) {
    switch (error) {
      case 'WebHID is not supported':
        // Show browser compatibility message
        break;
      case 'User cancelled device selection':
        // User action, no need for error message
        break;
      default:
        // Show generic error
        console.error(error);
    }
  }
};
```

## Browser Compatibility

The Ledger integration requires WebHID support:

- ✅ Chrome/Chromium (version 89+)
- ✅ Edge (version 89+)
- ✅ Opera (version 75+)
- ❌ Firefox (not supported)
- ❌ Safari (not supported)

## Security Considerations

1. **Device Verification**: Always verify addresses on the device screen
2. **Transaction Review**: Review all transaction details on device before confirming
3. **Firmware Updates**: Keep device firmware up to date
4. **Transport Security**: WebHID provides secure communication channel
5. **State Management**: Device state is managed securely in Rust backend

## Derivation Paths

Standard Solana derivation path format:
```
m/44'/501'/[account]'/[change]'
```

Examples:
- `m/44'/501'/0'/0'` - First account
- `m/44'/501'/1'/0'` - Second account
- `m/44'/501'/2'/0'` - Third account

## Transaction Signing Flow

1. User initiates transaction
2. Check if hardware signing is enabled
3. Validate transaction format and size
4. Send transaction to Ledger device
5. User reviews on device screen
6. User confirms on device
7. Signature returned to application
8. Transaction broadcast to network

## Testing

### Unit Tests

Run Rust tests:
```bash
cd src-tauri
cargo test ledger
```

Run TypeScript tests:
```bash
npm test ledger
```

### Integration Testing

The integration includes tests for:
- Device detection and connection
- Address derivation
- Transaction validation
- Error handling
- State management

## Troubleshooting

### Device Not Detected

1. Ensure device is connected via USB
2. Check if device is unlocked
3. Verify Solana app is open
4. Try reconnecting the device

### Transaction Signing Fails

1. Check transaction size (max 65535 bytes)
2. Verify derivation path format
3. Ensure device is not in screensaver mode
4. Check firmware version compatibility

### WebHID Permission Denied

1. Check browser permissions
2. Clear browser cache
3. Try in incognito/private window
4. Check for browser extensions blocking WebHID

## Future Enhancements

Potential improvements:
- Multi-signature support via Ledger
- Blind signing for custom programs
- Extended public key derivation
- Multiple account management
- Firmware version checking and warnings
- Transaction history on device

## Dependencies

### NPM Packages

- `@ledgerhq/hw-transport-webhid`: WebHID transport layer
- `@ledgerhq/hw-app-solana`: Solana app interface

### Rust Crates

- `tauri`: Application framework
- `serde`: Serialization
- `base64`: Encoding support
- `thiserror`: Error handling

## References

- [Ledger Developer Documentation](https://developers.ledger.com/)
- [WebHID API](https://wicg.github.io/webhid/)
- [Solana Derivation Paths](https://docs.solana.com/wallet-guide/paper-wallet)
- [BIP-44](https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki)
