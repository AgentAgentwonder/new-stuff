# Auto Update System - Implementation Guide

## Overview

Eclipse Market Pro includes a fully-featured auto-update system that provides:

- ✅ Automated update checking on startup
- ✅ User-configurable update schedules (Daily/Weekly/Never)
- ✅ Background downloads with progress tracking
- ✅ Delta updates for reduced bandwidth usage
- ✅ Rollback mechanism for failed updates
- ✅ User-friendly notification modal with changelog
- ✅ Settings panel for update preferences

## Architecture

### Frontend Components

#### 1. **Update Store** (`src/store/updateStore.ts`)
Manages all update-related state using Zustand:
- Update availability status
- Download progress tracking
- User settings (schedule, auto-download, auto-install)
- Rollback information

Key functions:
- `checkForUpdates()` - Checks for available updates using Tauri's built-in updater
- `downloadAndInstall()` - Downloads and installs the update, then relaunches the app
- `loadSettings()` / `saveSettings()` - Manage user preferences
- `rollbackUpdate()` - Reverts to previous version
- `setupEventListeners()` - Listens for update events

#### 2. **Update Notification Modal** (`src/components/UpdateNotificationModal.tsx`)
Displays when an update is available with:
- Current and new version numbers
- Changelog/release notes
- Download progress bar
- "Install Now" or "Install Later" options
- "Skip this version" option

#### 3. **Update Settings Panel** (`src/pages/Settings/UpdateSettings.tsx`)
Allows users to configure:
- Update check schedule (Daily/Weekly/Never)
- Auto-download preference
- Auto-install preference
- Last check timestamp
- Rollback to previous version (if available)

### Backend (Rust)

#### Updater Module (`src-tauri/src/updater.rs`)

Provides Tauri commands for:

**Commands:**
- `get_update_settings` - Retrieves user update preferences
- `save_update_settings` - Saves user update preferences
- `dismiss_update` - Marks a specific version as dismissed
- `get_rollback_info` - Checks if rollback is available
- `rollback_update` - Performs rollback to previous version

**State Management:**
- `UpdaterState` - Manages settings and backup paths
- Stores settings in `app_data_dir/updater_settings.json`
- Maintains backups in `app_data_dir/backups/`

## Configuration

### Tauri Configuration (`src-tauri/tauri.conf.json`)

```json
{
  "tauri": {
    "updater": {
      "active": true,
      "endpoints": [
        "https://updates.eclipsemarketpro.com/{{target}}/{{current_version}}"
      ],
      "dialog": false,
      "pubkey": "YOUR_PUBLIC_KEY_HERE"
    }
  }
}
```

### Configuration Options:

- **`active`**: Enables/disables the updater
- **`endpoints`**: URLs where update manifests are hosted
  - `{{target}}`: Platform (e.g., `windows-x86_64`, `darwin-aarch64`)
  - `{{current_version}}`: Current app version
- **`dialog`**: Set to `false` to use custom UI instead of native dialog
- **`pubkey`**: Public key for signature verification (generated using Tauri CLI)

## Deployment Process

### 1. Generate Signing Keys

```bash
# Install Tauri CLI
npm install -g @tauri-apps/cli

# Generate keys (run once)
tauri signer generate -w ~/.tauri/myapp.key

# This creates:
# - Private key: ~/.tauri/myapp.key
# - Public key: printed to console (add to tauri.conf.json)
```

### 2. Build and Sign the App

```bash
# Set private key path
export TAURI_PRIVATE_KEY="path/to/private.key"
export TAURI_KEY_PASSWORD="your_password"

# Build the app
npm run tauri build

# Output files (in src-tauri/target/release/bundle/):
# - Installer (e.g., .msi, .dmg, .appimage)
# - .sig signature file
# - Update manifest (auto-generated)
```

### 3. Create Update Manifest

The updater expects a JSON manifest at the configured endpoint:

**Example: `https://updates.eclipsemarketpro.com/windows-x86_64/1.0.0`**

```json
{
  "version": "1.1.0",
  "pub_date": "2024-01-15T12:00:00Z",
  "url": "https://cdn.eclipsemarketpro.com/releases/eclipse-market-pro-1.1.0.msi.zip",
  "signature": "base64_signature_from_.sig_file",
  "notes": "## What's New\n\n- Feature A\n- Bug fix B\n- Performance improvements"
}
```

### 4. Server Setup

#### Option A: Static Hosting (GitHub Releases, S3, etc.)

```
/releases/
  ├── windows-x86_64/
  │   ├── 1.0.0 (JSON manifest)
  │   ├── 1.1.0 (JSON manifest)
  │   └── eclipse-market-pro-1.1.0.msi.zip
  ├── darwin-aarch64/
  │   ├── 1.0.0 (JSON manifest)
  │   └── eclipse-market-pro-1.1.0.app.tar.gz
  └── linux-x86_64/
      ├── 1.0.0 (JSON manifest)
      └── eclipse-market-pro-1.1.0.AppImage.tar.gz
```

#### Option B: Dynamic Server

```javascript
// Express.js example
app.get('/updates/:target/:version', async (req, res) => {
  const { target, version } = req.params;
  
  // Query database for latest version
  const latest = await getLatestVersion(target, version);
  
  if (!latest || latest.version === version) {
    return res.status(204).send(); // No update
  }
  
  res.json({
    version: latest.version,
    pub_date: latest.date,
    url: latest.downloadUrl,
    signature: latest.signature,
    notes: latest.changelog
  });
});
```

## Update Channel Management

### Production Channel
- Stable releases only
- Thorough testing before deployment
- Endpoint: `https://updates.eclipsemarketpro.com/stable/`

### Beta Channel
- Pre-release features
- For early adopters
- Endpoint: `https://updates.eclipsemarketpro.com/beta/`

### Configuration:
```json
{
  "updater": {
    "endpoints": [
      "https://updates.eclipsemarketpro.com/{{channel}}/{{target}}/{{current_version}}"
    ]
  }
}
```

## Delta Updates

Tauri's updater automatically uses delta updates when possible:

1. **Binary Diff**: Only changed bytes are downloaded
2. **Signature Verification**: Each delta is cryptographically verified
3. **Fallback**: Falls back to full download if delta fails

**Benefits:**
- Reduced bandwidth (often 10-100x smaller)
- Faster updates
- Lower server costs

## Rollback Mechanism

### How it Works:

1. **Before Update**: Current version metadata saved to `app_data_dir/backups/metadata.json`
2. **After Failed Update**: User can trigger rollback from Settings
3. **Rollback Process**:
   - Restores files from backup
   - Restarts application
   - Updates metadata

### Implementation:

```typescript
// Frontend
const { rollbackInfo } = useUpdateStore();

if (rollbackInfo.available) {
  await rollbackUpdate();
}
```

```rust
// Backend (simplified)
#[tauri::command]
pub async fn rollback_update() -> Result<(), String> {
    // 1. Verify backup exists
    // 2. Stop current app
    // 3. Restore files from backup
    // 4. Restart app
    app_handle.restart()?;
    Ok(())
}
```

## Testing

### Development/Staging Testing

#### 1. Mock Update Server

```bash
# Create test manifest
cat > test-update.json << EOF
{
  "version": "1.0.1",
  "pub_date": "2024-01-01T00:00:00Z",
  "url": "http://localhost:8080/test-update.zip",
  "signature": "mock_signature",
  "notes": "Test update"
}
EOF

# Serve locally
python3 -m http.server 8080
```

#### 2. Update Configuration

```json
{
  "updater": {
    "endpoints": ["http://localhost:8080/test-update.json"]
  }
}
```

#### 3. Test Scenarios

- ✅ Update available and accepted
- ✅ Update available but dismissed
- ✅ Update available but "Install Later"
- ✅ No update available
- ✅ Update check failure (network error)
- ✅ Download progress tracking
- ✅ Update installation and relaunch
- ✅ Rollback after failed update

### Integration Tests

Located in `tests/updater.test.ts`:

```typescript
describe('Updater Integration', () => {
  it('should check for updates on startup', async () => {
    // Test implementation
  });

  it('should display update notification modal', async () => {
    // Test implementation
  });

  it('should handle update installation', async () => {
    // Test implementation
  });

  it('should respect user update schedule', async () => {
    // Test implementation
  });
});
```

## Security Considerations

### 1. Signature Verification
- All updates MUST be signed with private key
- Public key embedded in app config
- Tampering detected and rejected

### 2. HTTPS Only
- Update endpoints MUST use HTTPS in production
- HTTP allowed only for local testing

### 3. Version Validation
- Semantic versioning enforced
- Downgrades prevented by default

### 4. User Control
- Users can disable auto-updates
- Manual approval required by default
- Clear changelog display

## Troubleshooting

### Update Check Fails

**Symptom:** "Failed to check for updates" error

**Possible Causes:**
1. Network connectivity issues
2. Update server down
3. Invalid endpoint configuration
4. Firewall blocking request

**Solutions:**
- Check network connection
- Verify endpoint URL in `tauri.conf.json`
- Check server logs
- Test endpoint manually: `curl https://your-endpoint.com/...`

### Update Download Fails

**Symptom:** Download starts but fails

**Possible Causes:**
1. Signature verification failed
2. Corrupted download
3. Insufficient disk space

**Solutions:**
- Verify signature matches
- Check available disk space
- Re-download update package

### Rollback Not Available

**Symptom:** Rollback option grayed out

**Possible Causes:**
1. No previous version backup
2. Backup corrupted
3. First installation

**Solutions:**
- Backup only created after first update
- Check `app_data_dir/backups/` directory
- Verify `metadata.json` exists and is valid

## Best Practices

### For Users

1. **Enable Auto-Updates**: Keep the app secure with latest patches
2. **Review Changelogs**: Understand what's changing
3. **Backup Data**: Before major updates (though app handles this automatically)
4. **Test Beta Versions**: Help improve the app (optional)

### For Developers

1. **Version Properly**: Use semantic versioning (MAJOR.MINOR.PATCH)
2. **Write Clear Changelogs**: Users appreciate knowing what changed
3. **Test Thoroughly**: Test updates in staging before production
4. **Monitor Rollout**: Watch for issues after release
5. **Keep Keys Secure**: Never commit private signing keys
6. **Maintain Old Versions**: Keep previous versions available for rollback

## Support

### User Documentation
- Settings → Auto Update section
- In-app help tooltips
- FAQ: [https://docs.eclipsemarketpro.com/updates](https://docs.eclipsemarketpro.com/updates)

### Developer Documentation
- Tauri Updater Docs: [https://tauri.app/v1/guides/distribution/updater](https://tauri.app/v1/guides/distribution/updater)
- This guide
- Code comments in `src-tauri/src/updater.rs` and `src/store/updateStore.ts`

## Future Enhancements

Potential improvements for future versions:

- [ ] A/B testing for updates
- [ ] Automatic rollback on crash
- [ ] Update scheduling (specific time/day)
- [ ] Bandwidth throttling for downloads
- [ ] Multi-language changelog support
- [ ] Update statistics dashboard
- [ ] Staged rollout (gradual release to users)
