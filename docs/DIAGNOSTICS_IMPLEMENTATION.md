# Diagnostics & Crash Reporting Implementation

## Overview

This document describes the implementation of Phase 8 Tasks 8.4–8.5: Session Recording and Crash Reporting.

## Features Implemented

### 1. Session Recording (rrweb)

**Technology**: rrweb (Record and Replay the Web)

**Components**:
- `DiagnosticsProvider` - Manages rrweb recording lifecycle
- `SessionReplayViewer` - UI for viewing recorded sessions
- `DiagnosticsSettings` - Settings panel for managing diagnostics

**Features**:
- ✅ Records UI interactions (clicks, scrolls, typing)
- ✅ Captures console logs (log, warn, error, info, debug)
- ✅ Captures JavaScript errors and promise rejections
- ✅ Privacy masking for sensitive inputs (password fields, etc.)
- ✅ 30-minute rolling window storage
- ✅ Local-only storage (no automatic upload)
- ✅ Export functionality for support
- ✅ Opt-in with consent dialog
- ✅ Manual start/stop controls

**Privacy Features**:
- Automatic masking of password fields
- Configurable privacy masking toggle
- CSS-class based masking (`rrweb-mask`, `rrweb-mask-text`)
- Local storage only
- 30-minute automatic cleanup

### 2. Crash Reporting (Sentry)

**Technology**: Sentry for React

**Components**:
- `ErrorBoundary` - Catches React component errors
- `CrashDashboard` - Displays crash statistics and details
- Sentry integration in `main.tsx`

**Features**:
- ✅ Automatic error capturing
- ✅ Stack traces with component hierarchy
- ✅ Environment information collection
- ✅ User feedback/comments on crashes
- ✅ Local crash report storage
- ✅ Crash frequency analytics
- ✅ Optional Sentry integration
- ✅ Auto-restart capability
- ✅ Opt-in with consent dialog

**Crash Dashboard**:
- Last 24 hours crash count
- Last 7 days statistics
- All-time crash count
- Detailed crash view with stack traces
- User feedback display
- Delete individual or all reports

### 3. Auto-Restart

**Features**:
- Automatic app restart after crashes (configurable)
- Graceful error UI with user options
- Tauri `relaunch()` API integration
- Fallback to `window.location.reload()`

### 4. Privacy & Consent

**Consent Management**:
- Separate opt-in for session recording and crash reporting
- Detailed privacy information in consent dialogs
- Revocable consent at any time
- No data collection without explicit consent

## File Structure

```
src/
├── store/
│   └── diagnosticsStore.ts           # Zustand store for diagnostics state
├── providers/
│   └── DiagnosticsProvider.tsx       # rrweb lifecycle management
├── components/
│   └── common/
│       └── ErrorBoundary.tsx         # Error boundary with Sentry
├── pages/
│   └── Settings/
│       ├── DiagnosticsSettings.tsx   # Main settings UI
│       ├── SessionReplayViewer.tsx   # Session replay viewer
│       └── CrashDashboard.tsx        # Crash analytics dashboard
├── types/
│   └── rrweb-player.d.ts            # Type definitions for rrweb-player
└── __tests__/
    └── diagnostics.test.tsx          # Comprehensive test suite
```

## Dependencies

```json
{
  "rrweb": "^2.x",
  "rrweb-player": "^1.x",
  "@sentry/react": "^7.x",
  "@sentry/tracing": "^7.x"
}
```

## Configuration

### Environment Variables

Create a `.env` file (see `.env.example`):

```bash
# Optional: Sentry DSN for crash reporting
# Leave empty to disable Sentry integration
VITE_SENTRY_DSN=https://your-sentry-dsn@sentry.io/project-id
```

### Sentry Configuration

In `main.tsx`:

```typescript
Sentry.init({
  dsn: import.meta.env.VITE_SENTRY_DSN,
  environment: import.meta.env.MODE,
  tracesSampleRate: 0.2, // 20% of transactions in production
  enabled: Boolean(import.meta.env.VITE_SENTRY_DSN),
});
```

## Usage

### For Users

1. Navigate to **Settings** → **Diagnostics & Crash Reporting**
2. Click **Enable Session Recording** or **Enable Crash Reporting**
3. Review and accept the privacy consent dialog
4. Toggle features on/off as needed

**Session Recording**:
- View recent recordings in Settings
- Click the eye icon to replay a session
- Export recordings for support

**Crash Reporting**:
- View crash dashboard for statistics
- Review individual crash details
- Add feedback after crashes occur

### For Developers

#### Adding Privacy Masking

Mark sensitive elements with CSS classes:

```tsx
// Mask entire element
<div className="rrweb-mask">
  <input type="text" value={sensitiveData} />
</div>

// Mask only text content
<span className="rrweb-mask-text">{privateInfo}</span>
```

#### Capturing Custom Events

```typescript
import { useDiagnosticsStore } from '@/store/diagnosticsStore';

const { addConsoleLog, addErrorLog } = useDiagnosticsStore();

// Add custom console log
addConsoleLog({
  timestamp: Date.now(),
  level: 'info',
  message: 'Custom event occurred',
  args: [eventData],
});

// Add custom error
addErrorLog({
  timestamp: Date.now(),
  message: error.message,
  stack: error.stack,
});
```

#### Exporting Recordings Programmatically

```typescript
import { useDiagnosticsStore } from '@/store/diagnosticsStore';

const { recordings, exportRecording } = useDiagnosticsStore();

// Export the most recent recording
const latestRecording = recordings[recordings.length - 1];
if (latestRecording) {
  exportRecording(latestRecording.id);
}
```

## Testing

Run the test suite:

```bash
npm test -- src/__tests__/diagnostics.test.tsx
```

### Test Coverage

- ✅ Session recording consent flow
- ✅ Recording start/stop lifecycle
- ✅ Event capture and storage
- ✅ Privacy masking toggle
- ✅ Recording export
- ✅ Crash reporting consent flow
- ✅ Crash report creation and storage
- ✅ User feedback/comments
- ✅ Crash frequency analytics
- ✅ Auto-restart toggle

## Architecture Decisions

### 1. Local-First Storage

**Decision**: Store all diagnostic data locally in browser LocalStorage

**Rationale**:
- Privacy: User data never leaves device without explicit action
- Compliance: Easier GDPR/CCPA compliance
- Performance: No network overhead
- Security: Reduced attack surface

### 2. Separate Consent for Features

**Decision**: Require separate opt-in for session recording and crash reporting

**Rationale**:
- User control: Different privacy implications
- Flexibility: Users can enable only what they're comfortable with
- Compliance: Granular consent requirements
- Transparency: Clear about each feature's data collection

### 3. 30-Minute Rolling Window

**Decision**: Automatically delete recordings older than 30 minutes

**Rationale**:
- Privacy: Limited data retention
- Storage: Prevents localStorage bloat
- Relevance: Recent sessions are most useful for debugging
- Compliance: Aligns with data minimization principles

### 4. Optional Sentry Integration

**Decision**: Make Sentry integration optional via environment variable

**Rationale**:
- Flexibility: Works with or without external service
- Self-hosted: Users can run without cloud dependencies
- Cost: Free tier limitations
- Privacy: Some users prefer no external transmission

### 5. Zustand for State Management

**Decision**: Use Zustand with persistence middleware

**Rationale**:
- Consistency: Matches existing app architecture
- Persistence: Settings survive page reloads
- Simplicity: Less boilerplate than Redux
- Performance: Optimized re-renders

## Performance Considerations

### Recording Overhead

- **CPU**: ~2-5% increase during recording (rrweb is optimized)
- **Memory**: ~10-20MB for 30 minutes of recording
- **Storage**: ~1-5MB LocalStorage per session (compressed events)

### Optimization Strategies

1. **Throttling**: rrweb throttles mouse movements by default
2. **Sampling**: Only 20% of production transactions traced by Sentry
3. **Cleanup**: Old recordings auto-deleted
4. **Lazy Loading**: Replay player loaded on-demand

## Security Considerations

### Data Sanitization

Automatic filtering in crash reports:
- API keys and tokens in environment variables
- Authorization headers
- Query parameters with 'key', 'token', 'secret' in name

### Storage Security

- LocalStorage encrypted by browser
- No plaintext sensitive data in recordings
- Privacy masking prevents capturing passwords

### Network Security

- Sentry uses HTTPS only
- DSN can be rotated if compromised
- Rate limiting prevents data exfiltration

## Troubleshooting

### Session Recording Not Starting

1. Check consent is granted
2. Verify recording toggle is enabled
3. Check browser console for errors
4. Ensure localStorage is not full
5. Try disabling browser extensions

### Crash Reports Not Appearing

1. Verify crash reporting consent is granted
2. Check Sentry DSN is configured (if using Sentry)
3. Review browser network tab for failed requests
4. Check local crash dashboard for local reports

### Recording Playback Issues

1. Ensure recording has events (not empty)
2. Check for rrweb-player CSS loading
3. Verify browser compatibility (modern browsers only)
4. Clear browser cache and reload

### Performance Issues

1. Disable recording if not needed
2. Reduce recording duration
3. Enable privacy masking (less data captured)
4. Clear old recordings manually

## Future Enhancements

Potential improvements for future phases:

1. **Compressed Storage**: Use compression library for recordings
2. **Selective Recording**: Record only specific pages/components
3. **Remote Storage**: Optional cloud backup for recordings
4. **Video Export**: Export recordings as video files
5. **Heat Maps**: Generate interaction heat maps from recordings
6. **Sourcemap Integration**: Better error stack traces with sourcemaps
7. **Performance Profiling**: Integrate with browser performance APIs
8. **Network Recording**: Capture and replay network requests
9. **Mobile Support**: Extend to Tauri mobile apps
10. **Team Sharing**: Share recordings securely with team members

## Resources

- [rrweb Documentation](https://github.com/rrweb-io/rrweb)
- [Sentry React Documentation](https://docs.sentry.io/platforms/javascript/guides/react/)
- [Privacy Guide](./DIAGNOSTICS_PRIVACY_GUIDE.md)
- [Phase 8 Specification](./docs/phase8-spec.md)

## Support

For issues or questions:
1. Check this documentation
2. Review privacy guide
3. Check test suite for usage examples
4. Open an issue on the repository

---

**Last Updated**: 2024
**Version**: 1.0.0
**Phase**: 8.4-8.5
