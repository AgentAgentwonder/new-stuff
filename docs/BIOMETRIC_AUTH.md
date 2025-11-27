# Biometric Authentication System

This document describes the biometric authentication system implementation for Eclipse Market Pro.

## Overview

The biometric authentication system provides platform-specific biometric unlock capabilities:
- **Windows**: Windows Hello (fingerprint, facial recognition, or PIN)
- **macOS**: Touch ID
- **Linux**: Password-only fallback (biometric not supported)

## Features

### Core Capabilities
- ✅ Platform-specific biometric enrollment
- ✅ Biometric verification on app launch
- ✅ Fallback password authentication
- ✅ Secure credential storage using OS keychains
- ✅ Automatic lock screen on app resume
- ✅ Graceful degradation on unsupported platforms

### Security
- Password hashing using Argon2
- Secure token storage in OS-native credential managers
- No biometric data stored or logged
- Zeroize sensitive data in memory
- Platform-specific secure enclaves (Windows Credential Manager, macOS Keychain)

## Architecture

### Backend (Rust/Tauri)

#### Files
- `src-tauri/src/auth/biometric.rs` - Core biometric implementation
- `src-tauri/src/auth/mod.rs` - Tauri command exports

#### Tauri Commands
- `biometric_get_status()` - Check availability and enrollment status
- `biometric_enroll(fallbackPassword)` - Enroll biometric with fallback
- `biometric_verify()` - Trigger biometric authentication
- `biometric_disable()` - Disable and remove biometric data
- `biometric_verify_fallback(password)` - Verify using fallback password

### Frontend (React/TypeScript)

#### Components
- `src/components/auth/LockScreen.tsx` - Lock screen modal with biometric/password UI
- `src/pages/Settings.tsx` - Biometric settings configuration page

#### State Management
- App-level lock state tracking
- Automatic lock on visibility change
- Status synchronization with backend

## Usage

### For Users

#### Enabling Biometric Authentication
1. Navigate to Settings page
2. In the Security section, enter a fallback password (min. 8 characters)
3. Confirm the password
4. Click "Enable Windows Hello" or "Enable Touch ID"
5. Follow the platform-specific biometric prompt

#### Using Biometric Authentication
- On app launch, the lock screen will appear automatically
- Click "Unlock with Windows Hello/Touch ID" to authenticate
- Use "Use password instead" to authenticate with fallback password
- After 3 failed attempts, fallback to password is automatic

#### Disabling Biometric Authentication
1. Navigate to Settings page
2. Click "Disable Biometric Authentication"
3. Confirm the action

### For Developers

#### Testing

**Windows:**
```bash
# Requires Windows 10+ with biometric hardware or PIN set up
cargo test --features custom-protocol
```

**macOS:**
```bash
# Requires macOS with Touch ID
cargo test --features custom-protocol
```

**Linux:**
```bash
# Will show "Password Only" mode
cargo test --features custom-protocol
```

#### Adding New Platform Support

To add support for additional platforms:

1. Update `platform_kind()` in `biometric.rs`:
```rust
fn platform_kind() -> BiometricPlatform {
    #[cfg(target_os = "your_os")]
    {
        return BiometricPlatform::YourAuth;
    }
    // ...
}
```

2. Implement availability check:
```rust
#[cfg(target_os = "your_os")]
fn your_os_available() -> Result<bool, BiometricError> {
    // Check if biometric hardware is available
}
```

3. Implement verification:
```rust
#[cfg(target_os = "your_os")]
async fn your_os_verify() -> Result<(), BiometricError> {
    // Trigger biometric prompt
}
```

4. Add platform-specific dependencies in `Cargo.toml`:
```toml
[target.'cfg(target_os = "your_os")'.dependencies]
your-biometric-crate = "x.y.z"
```

## Platform-Specific Notes

### Windows Hello
- Requires Windows 10 (build 1903) or later
- Supports fingerprint readers, IR cameras, or PIN
- Uses Windows.Security.Credentials.UI.UserConsentVerifier
- Credentials stored in Windows Credential Manager

### Touch ID (macOS)
- Requires macOS 10.12.1 or later with Touch ID sensor
- MacBook Pro, MacBook Air, iMac, or Magic Keyboard with Touch ID
- Uses LocalAuthentication framework
- Credentials stored in macOS Keychain
- **Note**: Current implementation is a stub that always returns success. Full Touch ID integration requires objc2/LocalAuthentication bindings.

### Linux
- Biometric authentication not available
- Password-only mode enabled by default
- May add PAM integration in future for fingerprint readers

## Security Considerations

1. **Password Storage**: Fallback passwords are hashed with Argon2 and stored in OS keychains
2. **Token Generation**: Random 32-byte tokens are generated for enrollment validation
3. **Memory Safety**: Sensitive strings are zeroized after use
4. **No Biometric Data**: The app never stores or transmits biometric data - only uses OS APIs
5. **Secure Enclaves**: Platform-specific secure storage (Credential Manager, Keychain)

## Troubleshooting

### Windows: "Windows Hello not available"
- Ensure Windows Hello is set up in Windows Settings > Accounts > Sign-in options
- Verify biometric hardware is functioning
- Check that no Group Policy is blocking Windows Hello

### macOS: Touch ID prompts don't appear
- Ensure Touch ID is configured in System Preferences > Touch ID
- Verify the app has required permissions
- Note: Current implementation requires full LocalAuthentication framework integration

### Linux: Biometric option missing
- This is expected behavior
- Linux currently only supports password-based authentication

### Fallback Password Not Working
- Ensure you're entering the password set during enrollment
- Password is case-sensitive
- If lost, disable and re-enable biometric authentication

## Future Enhancements

- [ ] Support for YubiKey/FIDO2 hardware tokens
- [ ] Biometric timeout/auto-lock settings
- [ ] Multiple fallback password options
- [ ] Linux PAM integration for fingerprint readers
- [ ] Complete macOS Touch ID implementation using LocalAuthentication
- [ ] Android/iOS biometric support (if mobile targets added)
- [ ] Face recognition on supported hardware
- [ ] Encrypted data protection tied to biometric verification

## Dependencies

### Rust Crates
- `argon2` - Password hashing
- `keyring` - Cross-platform credential storage
- `zeroize` - Memory safety for sensitive data
- `windows` (Windows only) - Windows Hello APIs
- `security-framework` (macOS only) - Keychain access
- `core-foundation` (macOS only) - Core Foundation bindings

### Frontend
- `@tauri-apps/api` - Tauri IPC
- `framer-motion` - Lock screen animations
- `lucide-react` - Icons

## License

Follows the main project license.
