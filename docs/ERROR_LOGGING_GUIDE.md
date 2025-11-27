# Error Logging and Startup Crash Fix Guide

## Overview

This guide documents the comprehensive error logging system added to Eclipse Market Pro to catch and display errors that occur during initialization, instead of silently crashing to a black screen.

## Architecture

### 1. Global Error Logger (`src/utils/errorLogger.ts`)

A centralized error tracking system that:
- Logs all errors with timestamp, source, message, and stack trace
- Maintains a rolling buffer of up to 100 recent logs
- Provides styled console output in development
- Exposes logs via `window.__errorLogger` for debugging
- Supports different log levels: `error`, `warning`, `info`

**Usage:**
```typescript
import { errorLogger } from '@/utils/errorLogger';

errorLogger.error('Error message', 'ComponentName', error, { context: 'data' });
errorLogger.warning('Warning message', 'ComponentName');
errorLogger.info('Info message', 'ComponentName');
```

### 2. Dev Error Display (`src/components/DevErrorDisplay.tsx`)

A development-only overlay component that:
- Displays real-time error logs in the bottom-right corner
- Shows error count badge with visual distinction (red for errors, yellow for warnings)
- Provides detailed view with stack traces and context
- Allows users to clear logs, copy error reports
- Only visible in development mode

**Integration:**
```typescript
<DevErrorDisplay>
  <App />
</DevErrorDisplay>
```

### 3. Enhanced Error Boundary (`src/components/AppErrorBoundary.tsx`)

Updated React Error Boundary that:
- Catches component render errors
- Logs errors to the global error logger
- Displays user-friendly error fallback UI
- Provides "Try Again", "Copy Error", and "Report Issue" buttons

### 4. Main Entry Point Error Handling (`src/main.tsx`)

Top-level error capture:
- **Global error handler**: Catches all unhandled JavaScript errors
- **Unhandled promise rejection handler**: Catches promise rejections
- **Try-catch wrapper**: Wraps React initialization
- **Fallback UI**: Displays error screen if app fails to mount

## Initialization Logging

The following components log their initialization sequence:

### App Component (`src/App.tsx`)
```
✓ App.tsx - Mounting and rendering
✓ Error boundary wrapping entire app
✓ Global error handler callback
```

### APIProvider (`src/lib/api-context.tsx`)
```
✓ Loading API keys from localStorage
✓ Parsing JSON configuration
✓ Hook validation with error logging
```

### AccessibilityProvider (`src/components/providers/AccessibilityProvider.tsx`)
```
✓ Initialization and cleanup
✓ Font scale updates
✓ High contrast mode toggling
✓ Reduced motion preference application
```

### Tauri Command Hook (`src/hooks/useTauriCommand.ts`)
```
✓ Command execution start
✓ Command completion or failure
✓ Error context capture
✓ Automatic error toast display
```

## Error Flow Diagram

```
┌─────────────────────────────────────────┐
│  main.tsx - Global Error Handlers      │
│  - window.error event                  │
│  - unhandledrejection event            │
│  - Try-catch wrapper                   │
└──────────────┬──────────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│  Error Logger (errorLogger instance)    │
│  - Stores in rolling buffer (100)       │
│  - Logs to console (styled)             │
│  - Makes available on window            │
└──────────────┬──────────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│  Dev Error Display (overlay)            │
│  - Polls errorLogger every 100ms        │
│  - Shows badge & details panel          │
│  - Allows interaction (dev-only)        │
└──────────────┬──────────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│  AppErrorBoundary (React Error Boundary)│
│  - Catches component render errors      │
│  - Logs via errorLogger                 │
│  - Shows error fallback UI              │
└─────────────────────────────────────────┘
```

## File Changes Summary

### New Files Created
- `src/utils/errorLogger.ts` - Central error logging utility
- `src/components/DevErrorDisplay.tsx` - Development error overlay UI
- `ERROR_LOGGING_GUIDE.md` - This documentation

### Modified Files
- `src/main.tsx` - Added global error handlers and error display wrapper
- `src/App.tsx` - Added initialization logging
- `src/components/AppErrorBoundary.tsx` - Added error logging integration
- `src/components/index.ts` - Exported new DevErrorDisplay component
- `src/lib/api-context.tsx` - Added initialization logging
- `src/components/providers/AccessibilityProvider.tsx` - Added initialization logging
- `src/hooks/useTauriCommand.ts` - Added command error logging (restored with enhancements)

## Development Features

### Viewing Logs in Console
```javascript
// In browser console:
window.__errorLogger.getLogs()           // Get all logs
window.__errorLogger.getRecentLogs(5)    // Get last 5 logs
window.__errorLogger.getErrorReport()    // Get formatted report
window.__errorLogger.clear()             // Clear all logs
```

### Error Display Badge
- Red badge (❌): Indicates errors are present
- Yellow badge (⚠️): Indicates warnings only
- Click to expand/collapse details panel
- Shows error count

### Details Panel Features
- Lists all errors with timestamp
- Color-coded by type (red/yellow/green)
- "Show Details" button reveals:
  - Full stack trace
  - Context object (parameters, state, etc.)
- Copy Report button for easy sharing
- Clear button to reset logs

## Integration Points

### 1. Startup Crash Prevention
```
Before:  Error occurs → Black screen (silent failure)
After:   Error occurs → Logged → Displayed → Recovery UI
```

### 2. Tauri Command Failures
All Tauri commands wrapped with `useTauriCommand` hook now:
- Log command execution
- Log success/failure with context
- Automatically show error toast
- Populate error display

### 3. Component Initialization
Each provider logs:
- Initialization start
- Key state changes
- Cleanup on unmount
- Any errors encountered

## Usage Examples

### Basic Error Logging
```typescript
import { errorLogger } from '@/utils/errorLogger';

try {
  // Some operation
} catch (error) {
  errorLogger.error(
    'Operation failed',
    'MyComponent',
    error instanceof Error ? error : undefined
  );
}
```

### With Context
```typescript
errorLogger.error(
  'Failed to fetch data',
  'DataComponent',
  error instanceof Error ? error : undefined,
  {
    userId: user.id,
    endpoint: '/api/data',
    retryCount: 3
  }
);
```

### Info Logging
```typescript
errorLogger.info('User logged in', 'AuthProvider', {
  userId: user.id,
  method: 'google',
  timestamp: new Date().toISOString()
});
```

## Testing Error Display

### Trigger React Error
```typescript
// In any component
throw new Error('Test error');
```

### Trigger Global Error
```javascript
// In console
throw new Error('Unhandled error');
```

### Trigger Promise Rejection
```javascript
// In console
Promise.reject(new Error('Unhandled rejection'));
```

All should appear in the Dev Error Display when running in development mode.

## Performance Considerations

1. **Error Logger**: Minimal overhead, max 100 entries
2. **Dev Error Display**: Only renders in development
3. **Error Polling**: 100ms interval (only in dev)
4. **No Memory Leaks**: Automatic cleanup on component unmount

## Debugging Tips

### Finding Root Cause
1. Open Dev Error Display (bottom-right corner)
2. Click "Show Details" to see stack traces
3. Check the source component name
4. Look at context data for state at time of error

### Collecting Error Reports
1. Click "Copy Report" in error display
2. Paste into ticket/issue tracker
3. Includes timestamps, stack traces, context

### Monitoring Initialization
1. Watch console in F12 (styled colored logs)
2. Check Dev Error Display for any issues
3. Look for initialization sequence in logs

## Future Enhancements

1. **Remote Error Reporting**: Send critical errors to monitoring service
2. **Error Analytics**: Track error patterns and frequency
3. **Auto-Recovery**: Implement automatic retry mechanisms
4. **Error Filtering**: Allow users to filter by severity/source
5. **Persistent Logs**: Store logs to IndexedDB for analysis
6. **Source Maps**: Better stack trace deobfuscation
7. **Network Errors**: Track API/Tauri command failures
8. **Performance Monitoring**: Track slow operations

## Troubleshooting

### Errors Not Appearing
- Check if running in development mode
- Verify DevErrorDisplay is in component tree
- Check browser console for errors (F12)
- Clear browser cache

### Black Screen Still Occurring
- Check main.tsx error handlers
- Verify HTML root element exists
- Check browser console for errors before render
- Try clearing localStorage

### Error Display Not Showing
- Dev error display only shows in development
- Check NODE_ENV variable
- Verify DevErrorDisplay component is wrapped around App
- Check z-index conflicts with other overlays

## Additional Resources

- React Error Boundaries: https://react.dev/reference/react/Component#catching-rendering-errors-with-an-error-boundary
- Error Handling: https://developer.mozilla.org/en-US/docs/Web/API/GlobalEventHandlers/onerror
- Promise Rejection: https://developer.mozilla.org/en-US/docs/Web/API/PromiseRejectionEvent
