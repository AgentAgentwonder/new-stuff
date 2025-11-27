# Security Implementation: Keystore, Sessions & 2FA

This document describes the security features implemented for Eclipse Market Pro.

## Backend Implementation (Rust/Tauri)

### 1. Keystore (`src-tauri/src/security/keystore.rs`)

**Features:**
- AES-256-GCM encryption for all stored secrets
- OS-level secure storage via keyring crate (Keychain on macOS, Credential Manager on Windows, Secret Service on Linux)
- Argon2id key derivation for enhanced security
- Master key automatically generated and stored in OS secure storage
- Per-secret salt and nonce for encryption
- Automatic memory zeroing using zeroize crate
- Import/export functionality with password protection
- Key rotation support

**API:**
- `store_secret(key, data)` - Store encrypted secret
- `retrieve_secret(key)` - Retrieve and decrypt secret
- `remove_secret(key)` - Delete secret
- `export_backup(password)` - Export encrypted backup
- `import_backup(password, backup)` - Import from backup
- `rotate_master_key()` - Rotate encryption keys
- `list_keys()` - List all stored secret keys

### 2. Session Manager (`src-tauri/src/auth/session_manager.rs`)

**Features:**
- JWT-based session tokens
- Configurable session timeout (default: 15 minutes)
- Idle timeout tracking
- Session persistence via keystore (encrypted)
- Automatic session renewal
- Session validation

**API Commands:**
- `session_create({user_id, timeout_minutes?})` - Create new session
- `session_renew()` - Renew current session
- `session_end()` - End/logout session
- `session_status()` - Get session status including expiry
- `session_verify()` - Verify session is valid
- `session_update_activity()` - Update last activity timestamp
- `session_configure_timeout(minutes)` - Change timeout duration

**Session Warning:**
- 60-second warning threshold before expiration

### 3. Two-Factor Authentication (`src-tauri/src/auth/two_factor.rs`)

**Features:**
- TOTP implementation (RFC 6238) using HMAC-SHA1
- QR code generation as SVG for authenticator app setup
- 10 backup codes with SHA-256 hashing
- Time-window tolerance (±1 period = ±30 seconds)
- Manual entry key provided
- Backup code regeneration

**API Commands:**
- `two_factor_enroll(user_id)` - Enroll in 2FA, returns QR code and backup codes
- `two_factor_verify({code})` - Verify TOTP code or backup code
- `two_factor_disable()` - Disable 2FA
- `two_factor_status()` - Get enrollment status
- `two_factor_regenerate_backup_codes()` - Generate new backup codes

**TOTP Spec:**
- Issuer: "EclipseMarketPro"
- Digits: 6
- Period: 30 seconds
- Algorithm: SHA1

### 4. Integration

All modules are initialized in `src-tauri/src/lib.rs`:
- Keystore initialized on app startup
- Session manager hydrates from keystore
- 2FA manager hydrates configuration
- All managed as Tauri state for command access

## Frontend Implementation (React/TypeScript)

### Required Components

1. **Session Monitoring Hook** (`src/hooks/useSessionMonitor.ts`)
   - Track user activity (mouse/keyboard events)
   - Auto-refresh session token on activity
   - Display warning UI when session near expiry
   - Auto-logout on expiration
   - Debounce activity updates

2. **Session Context** (`src/providers/SessionProvider.tsx`)
   - Global session state management
   - Countdown display
   - Lock screen trigger

3. **Settings UI Updates** (`src/pages/Settings.tsx`)
   - Keystore backup/restore section
   - Session timeout configuration (5, 15, 30, 60 minutes)
   - 2FA enrollment section with QR code display
   - Backup codes display and download
   - 2FA verification test

4. **2FA Enrollment Component** (`src/components/auth/TwoFactorEnrollment.tsx`)
   - Display QR code (SVG from backend)
   - Show manual entry key
   - Display backup codes for printing/saving
   - Verification step

5. **Session Warning Modal** (`src/components/auth/SessionWarning.tsx`)
   - Countdown timer
   - "Extend Session" button
   - "Logout" button

## Security Best Practices

1. **Memory Safety:**
   - All sensitive data zeroed on drop using zeroize
   - Secrets wrapped in `Zeroizing<T>`
   - No plaintext secrets in logs

2. **Encryption:**
   - AES-256-GCM authenticated encryption
   - Unique salt and nonce per encryption
   - Argon2id for key derivation (19 MiB memory, 2 iterations)

3. **Storage:**
   - Master key in OS secure storage only
   - No plaintext secrets on disk
   - Session state encrypted in keystore

4. **2FA:**
   - Backup codes hashed (SHA-256)
   - One-time use backup codes
   - Time-window tolerance to handle clock skew

5. **Sessions:**
   - JWT signed with 64-byte secret
   - Activity tracking to prevent idle sessions
   - Auto-expiration and renewal

## Testing

### Manual Testing Checklist:

- [ ] Keystore stores and retrieves secrets correctly
- [ ] Keystore export/import with password works
- [ ] Session creates and persists across app restarts (within timeout)
- [ ] Session expires after timeout period
- [ ] Session renews on activity
- [ ] 2FA enrollment generates valid QR code
- [ ] Authenticator app (Google Authenticator, Authy) accepts QR code
- [ ] TOTP verification works
- [ ] Backup codes work
- [ ] Backup codes are one-time use
- [ ] Settings UI allows all operations

### Automated Test Coverage:
- Unit tests for encryption/decryption
- TOTP generation validation
- Session expiration logic
- Backup code hashing

## Usage Examples

### Backend (Rust):

```rust
// Store a secret
keystore.store_secret("api-key", b"secret-value")?;

// Retrieve a secret
let secret = keystore.retrieve_secret("api-key")?;

// Create session
let session = session_manager.create_session("user123".to_string(), Some(30), &keystore)?;

// Enroll 2FA
let enrollment = two_factor_manager.enroll("user@example.com", &keystore)?;
println!("QR Code: {}", enrollment.qr_code);
println!("Backup Codes: {:?}", enrollment.backup_codes);

// Verify 2FA
let valid = two_factor_manager.verify("123456", &keystore)?;
```

### Frontend (TypeScript):

```typescript
// Create session
await invoke('session_create', { user_id: 'user123', timeout_minutes: 15 });

// Check session status
const status = await invoke<SessionStatus>('session_status');

// Enroll 2FA
const enrollment = await invoke<TwoFactorEnrollment>('two_factor_enroll', { 
  user_id: 'user@example.com' 
});

// Verify code
const valid = await invoke<boolean>('two_factor_verify', { 
  code: '123456' 
});
```

## Smart Contract Audit Module

### Backend (Rust/Tauri)

**Location:** `src-tauri/src/security/audit.rs`

**Features:**
- Heuristic scanner for Solana token programs
- External audit integration (CertiK, Trail of Bits)
- Caching layer for audit results
- Risk scoring and level calculation
- Detection of:
  - Dangerous functions (selfdestruct, delegatecall)
  - Mint authority presence
  - Freeze authority presence
  - Blacklist mechanisms
  - Honeypot patterns
  - Low holder counts
  - Unchecked external calls

**Data Structures:**
- `AuditResult` - Complete audit report with score, findings, and metadata
- `Finding` - Individual security issue with severity and recommendations
- `AuditSource` - External audit provider info
- `AuditMetadata` - Token characteristics (mintable, freeze authority, etc.)
- `RiskLevel` - Low, Medium, High, Critical based on score

**API Commands:**
- `scan_contract(contract_address)` - Perform full audit scan
- `get_cached_audit(contract_address)` - Retrieve cached audit
- `clear_audit_cache()` - Clear all cached audits
- `check_risk_threshold(security_score, user_threshold)` - Check if score below threshold

**Scoring System:**
- Base score: 100
- Critical findings: -30 points each
- High findings: -15 points each
- Medium findings: -8 points each
- Low findings: -3 points each
- Final score aggregated with external audit scores when available

### Frontend (React/TypeScript)

**Components:**

1. **TokenSecurityPanel** (`src/components/security/TokenSecurityPanel.tsx`)
   - Full security analysis dashboard
   - Real-time scanning with refresh
   - Metadata summary (mintable, freeze authority, etc.)
   - External audit sources with report links
   - Detailed findings display

2. **SecurityScoreBadge** (`src/components/security/SecurityScoreBadge.tsx`)
   - Visual score indicator with color coding
   - Risk level badge (Low/Medium/High/Critical)
   - Configurable sizes

3. **AuditFindings** (`src/components/security/AuditFindings.tsx`)
   - Categorized finding cards
   - Severity indicators
   - Recommendations display
   - Source attribution

4. **SecurityRiskAlert** (`src/components/security/SecurityRiskAlert.tsx`)
   - Modal warning for high-risk contracts
   - Override option with explicit confirmation
   - Risk level specific messaging

**Integration:**

The `TradeConfirmationModal` now includes:
- Automatic security scanning of destination tokens
- Real-time security score display
- Inline warnings for high/critical risk tokens
- Mandatory review for flagged contracts
- Override flow requiring explicit user confirmation

**Hook:**

`useSecurityAudit` provides:
- Automatic fetching with caching
- Loading and error states
- Manual refresh capability
- Contract address-based scanning

**Types:**

All audit types defined in `src/types/audit.ts`:
- `AuditResult`, `Finding`, `AuditSource`, `AuditMetadata`
- `RiskLevel`, `Severity`, `AuditStatus`
- `SecurityAlertConfig`

### Testing

**Unit Tests (Rust):**
- Risk level calculation from scores
- Score calculation with various findings
- Honeypot detection
- Mintable token detection
- Low holder count warnings
- Full audit flow

**Sample Contracts:**

Mock data generation for testing includes:
- Various token characteristics (mintable, frozen, blacklist)
- Deterministic scoring based on address
- Simulated external audit responses

**Manual Testing Checklist:**

- [ ] Audit scan completes for valid contract addresses
- [ ] Cache properly stores and retrieves results
- [ ] Security score calculation is accurate
- [ ] Risk levels properly categorized
- [ ] Findings display correct severity and recommendations
- [ ] High-risk contracts trigger trade warnings
- [ ] Override flow works correctly
- [ ] External audit sources display with links
- [ ] Refresh updates audit data
- [ ] Loading states display correctly

### Security Best Practices

1. **Rate Limiting:** Consider implementing rate limits for API calls to external audit providers
2. **Data Validation:** All contract addresses validated before scanning
3. **Error Handling:** Graceful fallbacks if audit services unavailable
4. **User Warnings:** Clear, non-dismissible warnings for critical risks
5. **Audit Trail:** All security overrides logged for accountability

### Usage Examples

**Backend:**
```rust
// Scan a contract
let result = scan_contract("TokenMintAddress123".to_string(), app).await?;

// Check if score below threshold
let is_risky = check_risk_threshold(result.security_score, Some(60))?;
```

**Frontend:**
```typescript
// Use audit hook
const { audit, loading, refresh } = useSecurityAudit({
  contractAddress: 'TokenMintAddress123',
  autoFetch: true,
});

// Display security panel
<TokenSecurityPanel contractAddress={tokenAddress} />

// Show security badge
{audit && (
  <SecurityScoreBadge
    score={audit.securityScore}
    riskLevel={audit.riskLevel}
  />
)}
```

### Configuration

**Cache Duration:** 1 hour (configurable in `AuditCache::new()`)

**Risk Thresholds:**
- Low: 80-100
- Medium: 60-79
- High: 40-59
- Critical: 0-39

**Trade Blocking:**
- By default, high and critical risk tokens trigger warnings
- Users can override with explicit confirmation
- Can be configured to hard-block certain risk levels

## Safety Mode Engine

### Backend Implementation (Rust/Tauri)

**Location:** `src-tauri/src/trading/safety/`

**Features:**
- Policy-based trade validation
- Configurable cooldown periods between trades
- Transaction impact simulation
- MEV risk assessment and protection suggestions
- Optional insurance provider integration
- Risk limit enforcement (trade amount, price impact, slippage)
- Daily trade frequency limits
- High-risk token blocking

**Data Structures:**

- `SafetyPolicy` - Configurable safety rules and thresholds
- `PolicyViolation` - Specific policy rule violations
- `CooldownStatus` - Per-wallet cooldown tracking
- `TransactionSimulation` - Pre-trade simulation results
- `ImpactPreview` - Trade impact estimates
- `InsuranceQuote` - Insurance provider quotes
- `SafetyCheckResult` - Comprehensive safety validation result

**API Commands:**
- `check_trade_safety(request)` - Validate trade against all safety policies
- `approve_trade(wallet_address)` - Record trade and start cooldown
- `get_safety_policy()` - Retrieve current safety configuration
- `update_safety_policy(policy)` - Update safety settings
- `get_cooldown_status(wallet_address)` - Check cooldown status
- `reset_daily_limits()` - Reset daily trade counters
- `get_insurance_quote(provider_id, ...)` - Get insurance quote
- `select_insurance(provider_id, ...)` - Select insurance coverage
- `list_insurance_providers()` - List available insurance providers

**Safety Policies:**

Default Configuration:
- Cooldown: 30 seconds between trades
- Max trade amount: $10,000
- Max daily trades: 100
- Max price impact: 10%
- Max slippage: 5%
- High risk threshold: 40 (blocks tokens with security score below 40)
- Insurance recommendation threshold: $50,000

**Policy Enforcement:**

1. **Cooldown Manager**: Tracks last trade timestamp per wallet, enforces waiting period
2. **Policy Engine**: Validates trades against configured limits and rules
3. **Transaction Simulator**: 
   - Estimates output amounts (expected, minimum, maximum)
   - Calculates price impact and slippage
   - Assesses MEV risk level (low/medium/high/critical)
   - Provides success probability estimate
4. **Insurance Coordinator**:
   - Manages multiple insurance providers
   - Generates quotes based on risk factors
   - Recommends best coverage options

**Violation Severity Levels:**
- `Warning` - Trade allowed but flagged
- `Error` - Trade blocked but can be overridden
- `Critical` - Trade hard-blocked, no override

### Frontend Implementation (React/TypeScript)

**Components:**

1. **SafetySettings** (`src/pages/Settings/SafetySettings.tsx`)
   - Master safety mode toggle
   - Cooldown period configuration
   - Trade limit settings (amount, daily frequency)
   - Risk thresholds (price impact, slippage, security score)
   - Transaction simulation toggle
   - Insurance threshold configuration

2. **Enhanced TradeConfirmationModal** (existing component enhanced)
   - Safety check integration
   - Cooldown timer display
   - Policy violation warnings
   - Transaction simulation results
   - MEV protection suggestions
   - Insurance recommendations

**State Management:**

`useSafetyStore` (Zustand):
- Persisted safety policy configuration
- Policy fetch and update methods
- Cooldown status retrieval
- Error handling

**Hooks:**

`useSafety`:
- `checkTradeSafety` - Validate trade before execution
- `approveTrade` - Record successful trade
- `getCooldownStatus` - Check wallet cooldown
- `getInsuranceQuote` - Request insurance quote
- `selectInsurance` - Select insurance coverage
- `listInsuranceProviders` - Get available providers

**Types:**

All safety types defined in `src/types/safety.ts`

### Testing

**Unit Tests (Rust):** `src-tauri/tests/safety_tests.rs`

Test Coverage:
- Policy enforcement (amount limits, daily limits, risk thresholds)
- Cooldown logic (recording, expiration, status checks)
- Transaction simulation (normal trades, high-impact trades)
- Impact preview generation
- Insurance provider management
- Insurance quote generation
- Full safety engine integration
- High-risk token blocking
- Insurance requirement logic
- Disabled safety mode behavior

### Usage Examples

**Backend (Rust):**
```rust
// Create safety engine
let policy = SafetyPolicy::default();
let mut engine = SafetyEngine::new(policy, 30);

// Check trade safety
let request = SafetyCheckRequest {
    wallet_address: "wallet123".to_string(),
    input_amount: 100.0,
    amount_usd: 5000.0,
    // ... other fields
};
let result = engine.check_trade_safety(request).await?;

// Approve trade if allowed
if result.allowed {
    engine.approve_trade("wallet123");
}
```

**Frontend (TypeScript):**
```typescript
import { useSafety } from '../hooks/useSafety';

// Check trade safety
const { checkTradeSafety, approveTrade } = useSafety();

const safetyCheck = await checkTradeSafety({
  wallet_address: walletAddress,
  input_amount: amount,
  amount_usd: amountUsd,
  // ... other fields
});

if (safetyCheck?.allowed) {
  // Execute trade
  await executeTrade();
  // Record trade
  await approveTrade(walletAddress);
}
```

### Integration with Existing Trade Flow

1. Before displaying trade confirmation modal, check safety policies
2. Display violations and warnings in confirmation UI
3. Show cooldown timer if wallet is on cooldown
4. Present transaction simulation results
5. Offer insurance if trade exceeds threshold
6. Display MEV protection suggestions
7. After successful trade, record trade for cooldown tracking

### Security Best Practices

1. **Fail-Safe**: Default to blocking trades if safety checks fail
2. **User Control**: Users can disable safety mode entirely in settings
3. **Transparency**: All violations clearly displayed with explanations
4. **Flexibility**: Most limits are configurable and can be disabled
5. **Insurance**: Optional but recommended for large trades
6. **MEV Protection**: Integrated suggestions for Jito bundles and private RPCs

## Future Enhancements

1. Per-trade 2FA enforcement thresholds
2. Hardware security key support (WebAuthn/FIDO2)
3. Session device tracking
4. Audit log for security events
5. Rate limiting for 2FA attempts
6. Emergency contact recovery
7. Real-time contract bytecode analysis
8. Integration with additional audit providers (Quantstamp, OpenZeppelin)
9. Historical audit tracking and comparison
10. Automated alerts for new security findings
11. Community-driven security reports
12. Smart contract simulation before execution (✅ Implemented)
13. Time-delayed withdrawals for large amounts
14. Multi-signature requirement for high-value trades
15. Automated circuit breakers during high volatility
16. Integration with real-time insurance oracle pricing
