# Eclipse Market Pro

A Vite-powered React 18 + TypeScript desktop application with Tauri backend for tracking cryptocurrency markets, connecting to Solana wallets, and analyzing market data.

## Features

- **Phantom Wallet Integration**: First-class Phantom wallet connectivity with persistent sessions
- **Ledger Hardware Wallet Support**: Secure Solana transaction signing via Ledger devices using WebHID
- **Real-time Market Data**: Live price tickers, charts, and order book data
- **Trading Tools**: Swap form, automation rules, and risk indicators
- **Multi-Module Dashboard**: Navigate between Coins, Stocks, Insiders, Trading, and Settings
- **Sentiment Analysis**: AI-powered text sentiment and risk assessment
- **Tauri Backend**: Rust-based commands for wallet management, market data, and API integrations

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (v18 or higher)
- [Rust](https://www.rust-lang.org/) (latest stable)
- [Tauri CLI](https://tauri.app/)
- [Phantom Wallet](https://phantom.app/) browser extension (for wallet connectivity)

### Installation

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd eclipse-market-pro
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Configure environment variables:
   ```bash
   cp .env.example .env
   ```
   The default SQLite database path for SQLx compile-time checking is `./database.db`. Update any optional settings in `.env` as needed:
   ```env
   # Optional: Solana configuration
   VITE_SOLANA_NETWORK=mainnet-beta
   VITE_SOLANA_RPC_ENDPOINT=https://api.mainnet-beta.solana.com
   SOLANA_RPC_ENDPOINT=https://api.mainnet-beta.solana.com
   ```

### Development

Run the app in development mode:
```bash
npm run tauri dev
```

### Formatting & Linting

The project uses **Prettier** and **ESLint** to maintain a consistent code style.

- Format code automatically:
  ```bash
  npm run format
  ```
- Check formatting without modifying files:
  ```bash
  npm run format:check
  ```
- Run ESLint:
  ```bash
  npm run lint
  ```

### Build

Build the application for production:
```bash
npm run tauri build
```

## Phantom Wallet Integration

### Overview

The application provides seamless Phantom wallet integration with the following features:

- **Auto-Reconnect**: Automatically reconnects to your previously connected wallet on app launch
- **Persistent Sessions**: Wallet state is persisted across app restarts using both localStorage and Tauri backend storage
- **Network Support**: Supports Mainnet, Devnet, and Testnet
- **Balance Tracking**: Real-time SOL balance updates every 15 seconds
- **Error Handling**: Graceful handling of user rejections, network errors, and missing wallet providers

### Usage

1. **Connect Wallet**: Click the "Connect Wallet" button in the top-right corner
2. **Approve Connection**: Approve the connection request in your Phantom wallet
3. **View Balance**: Your wallet address and SOL balance will be displayed
4. **Disconnect**: Click the "Disconnect" button to end the session

### Architecture

The Phantom integration consists of:

- **Frontend Store** (`src/store/walletStore.ts`): Zustand store managing wallet state with persistence
- **UI Component** (`src/components/wallet/PhantomConnect.tsx`): React component for wallet connection UI
- **Wallet Provider** (`src/providers/SolanaWalletProvider.tsx`): Solana wallet adapter context provider
- **Backend Commands** (`src-tauri/src/wallet/phantom.rs`): Rust Tauri commands for session management and balance queries

### Backend Commands

The following Tauri commands are available:

- `phantom_connect`: Connect and persist a wallet session
- `phantom_disconnect`: Disconnect and clear the session
- `phantom_session`: Retrieve the current session
- `phantom_balance`: Fetch the SOL balance for an address
- `phantom_sign_message`: Verify a signed message (client-side signing via adapter)
- `phantom_sign_transaction`: Verify a signed transaction (client-side signing via adapter)

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `VITE_SOLANA_NETWORK` | Solana network (mainnet-beta, devnet, testnet) | `devnet` |
| `VITE_SOLANA_RPC_ENDPOINT` | Custom RPC endpoint URL | Network default |
| `SOLANA_RPC_ENDPOINT` | Backend RPC endpoint URL | Network default |

## Ledger Hardware Wallet Integration

### Requirements

- Chrome/Chromium, Edge, or Opera with WebHID support
- Ledger firmware 2.1.0+
- Solana app installed and opened on the device

### Key Features

- Secure transaction signing through Ledger hardware wallets using WebHID
- Address derivation with on-device verification
- Transaction validation and error handling
- Tauri backend state management for connected devices

### Workflow

1. Open the Hardware Wallet Manager from Settings
2. Connect the Ledger device via USB and unlock it
3. Open the Solana app on the Ledger device
4. Use the "Connect Ledger" flow to pair the device
5. Derive and verify addresses on the device screen
6. Sign transactions through the Ledger device during trading

### Core Components

- **Service** (`src/utils/ledger.ts`): Wraps Ledger WebHID and Solana app APIs
- **Hook** (`src/hooks/useLedger.ts`): React hook for managing connection state
- **UI** (`src/components/wallet/LedgerConnect.tsx`): Guided connection and signing UI
- **Backend** (`src-tauri/src/wallet/ledger.rs`): Device registry and validation commands

For implementation details see [LEDGER_INTEGRATION.md](./LEDGER_INTEGRATION.md).

## Animation System

Eclipse Market Pro features a comprehensive animation system with lunar/eclipse-inspired motion primitives built on Framer Motion. The system includes:

- **Reusable Animation Primitives**: Orbital transitions, constellation effects, corona glows
- **Shared Components**: Eclipse loaders, moon phase indicators, progress bars, constellation backgrounds
- **Parallax Effects**: Scroll-based depth and ambient motion
- **Accessibility First**: Full support for reduced motion preferences
- **Performance Optimized**: Memoized components and GPU-accelerated transforms

See the [Animation Guide](./ANIMATION_GUIDE.md) for detailed documentation on:
- Animation tokens (durations, easings)
- Component usage examples
- Accessibility implementation
- Performance guidelines
- Testing strategies

## Project Structure

```
eclipse-market-pro/
├── src/
│   ├── components/
│   │   ├── wallet/
│   │   │   ├── PhantomConnect.tsx
│   │   │   ├── LedgerConnect.tsx
│   │   │   └── HardwareWalletManager.tsx
│   │   ├── ApiSettings.tsx
│   │   ├── LiveChart.tsx
│   │   └── ...
│   ├── hooks/
│   │   ├── useWallet.ts
│   │   ├── useLedger.ts
│   │   └── ...
│   ├── utils/
│   │   ├── ledger.ts
│   │   └── ...
│   ├── pages/
│   │   ├── Dashboard.tsx
│   │   └── ...
│   ├── providers/
│   │   └── SolanaWalletProvider.tsx
│   ├── store/
│   │   └── walletStore.ts
│   ├── App.tsx
│   └── main.tsx
├── src-tauri/
│   ├── src/
│   │   ├── wallet/
│   │   │   ├── mod.rs
│   │   │   ├── phantom.rs
│   │   │   ├── ledger.rs
│   │   │   └── hardware_wallet.rs
│   │   ├── lib.rs
│   │   └── ...
│   └── Cargo.toml
├── package.json
├── LEDGER_INTEGRATION.md
└── README.md
```

## Dependencies

### Frontend
- React 18
- TypeScript
- Vite
- Tailwind CSS
- Framer Motion
- Lucide React (icons)
- @solana/wallet-adapter-react
- @solana/wallet-adapter-phantom
- @solana/web3.js
- @ledgerhq/hw-transport-webhid
- @ledgerhq/hw-app-solana
- Zustand (state management)
- Recharts / Lightweight Charts


### Backend (Rust)
- Tauri
- solana-client
- solana-sdk
- serde / serde_json
- tokio
- reqwest
- base64
- bincode
- chrono

## License

[Your License Here]

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.
