# Mobile Test Suite

This directory contains end-to-end tests for the Eclipse Market Pro mobile companion applications.

## Prerequisites

- Node.js 18+
- Yarn or npm
- Xcode 14+ (for iOS)
- Android Studio (for Android)
- Detox CLI
- Appium server

## Setup

```bash
# Install dependencies
yarn install

# Install Detox CLI
npm install -g detox-cli

# Build mobile apps
cargo tauri ios dev
cargo tauri android dev

# Run Appium server
appium
```

## Running Tests

```bash
# Detox iOS
detox test --configuration ios

# Detox Android
detox test --configuration android

# Appium
npm run test:mobile
```

## Test IDs

The React Native mobile app should expose elements with the following test IDs to support automation:

| Screen              | TestID                       | Description |
|---------------------|-----------------------------|-------------|
| Registration        | `registration-screen`        | Initial device registration view |
|                     | `device-name-input`          | Input field for device name |
|                     | `register-button`            | Register device action |
| Dashboard           | `dashboard-screen`           | Main dashboard container |
| Quick Trade         | `quick-trade-screen`         | Quick trade view |
|                     | `symbol-selector`            | Asset selector |
|                     | `amount-input`               | Trade amount input |
|                     | `buy-button` / `sell-button` | Trade direction buttons |
|                     | `confirm-trade`              | Confirm trade button |
| Widgets             | `widget-price-watch`         | Price watch widget |
| Notifications       | `notifications-tab`          | Notifications tab button |
|                     | `notification-item-{index}`  | Notification list item |

Ensure the mobile UI app implements these identifiers for automated tests. Adjust the tests if the UI changes.
