# Developer Tools Guide

## Overview

The Developer Tools system provides comprehensive auto-compilation, auto-fixing, and logging capabilities for the Eclipse Market Pro application. This system makes the app more self-healing and provides detailed insights into application behavior.

## Components

### Backend (Rust/Tauri)

#### 1. Logger (`src-tauri/src/logger/`)

A comprehensive logging system with multiple levels and destinations:

**Log Levels:**
- TRACE: Extremely detailed debugging information
- DEBUG: Development debugging
- INFO: General informational messages  
- WARN: Potential issues
- ERROR: Non-critical errors
- FATAL: Critical errors that crash the app
- SUCCESS: Successful operations
- PERFORMANCE: Performance metrics

**Features:**
- Colored console output
- File logging with rotation
- In-memory buffer for UI display
- Configurable log levels
- Metadata enrichment (timestamps, thread IDs, memory usage, etc.)

#### 2. Performance Monitor (`src-tauri/src/monitor/`)

Real-time system performance monitoring:

**Metrics Tracked:**
- CPU usage (system and process)
- Memory usage (total and process)
- Disk I/O rates
- Network throughput
- FPS estimates
- Event loop lag

**Features:**
- Updates every 500ms
- Broadcast channels for subscribers
- Lightweight overhead

#### 3. Error Recovery (`src-tauri/src/recovery/`)

Automatic error recovery mechanisms:

**Recovery Strategies:**
- Exponential backoff retry
- Fallback alternatives
- State recovery
- Data recovery from backups

**Features:**
- Configurable retry policies
- Recovery attempt tracking
- Success rate monitoring

#### 4. Crash Reporter (`src-tauri/src/errors/`)

Comprehensive crash reporting:

**Captured Information:**
- Error message and stack trace
- System state at crash time
- Last 1000 log entries
- App version and environment
- User actions leading to crash

**Features:**
- Automatic crash dumps
- Crash report persistence
- Crash history tracking

#### 5. Auto-Compiler (`src-tauri/src/compiler/`)

Build status tracking and error detection:

**Features:**
- Build status monitoring (idle/building/success/failed)
- Compilation error tracking
- Warning collection
- Build duration metrics

#### 6. Auto-Fixer (`src-tauri/src/fixer/`)

Automatic code fix attempts:

**Fix Types:**
- Import fixes (missing, unused, incorrect paths)
- Type annotations
- Formatting issues
- Common syntax errors

**Features:**
- Pattern-based fixing
- Fix history tracking
- Success rate statistics
- Max attempts limiting

### Frontend (React/TypeScript)

#### DevConsole Component (`src/pages/DevConsole.tsx`)

A comprehensive developer console with multiple tabs:

**Console Tab:**
- Real-time log stream
- Color-coded by level
- Filter by level/search
- Export/clear logs

**Compiler Tab:**
- Build status indicator
- Error list with file locations
- Auto-fix button
- Manual compilation trigger

**Errors Tab:**
- Error statistics
- Error grouping by type/code
- Auto-fix success rate
- Recovery rate metrics

**Performance Tab:**
- Real-time CPU/Memory graphs
- Process metrics
- System resource usage

## API Commands

### Logging

```typescript
// Get logs
await invoke('get_logs', { limit: 1000, level: 'INFO' });

// Clear logs
await invoke('clear_logs');

// Export logs
await invoke('export_logs', { format: 'json' });

// Log a message
await invoke('log_message', { 
  level: 'INFO',
  message: 'Custom log message',
  category: 'app',
  details: { foo: 'bar' }
});

// Get/set logger config
await invoke('get_logger_config');
await invoke('set_logger_config', { config });
```

### Compilation

```typescript
// Force compilation
await invoke('compile_now');

// Get build status
await invoke('get_build_status');

// Get compilation errors
await invoke('get_compile_errors');
```

### Auto-Fixing

```typescript
// Auto-fix errors
await invoke('auto_fix_errors', { errors: ['error message 1', 'error message 2'] });

// Get fix statistics
await invoke('get_fix_stats');

// Get fix attempt history
await invoke('get_fix_attempts');

// Clear fix history
await invoke('clear_fix_history');
```

### Performance Monitoring

```typescript
// Get current performance metrics
await invoke('get_performance_metrics');
```

### Error Handling

```typescript
// Get error statistics
await invoke('get_error_stats');

// Report a crash
await invoke('report_crash', { 
  message: 'App crashed',
  stackTrace: error.stack 
});

// Get crash report
await invoke('get_crash_report', { crashId: 'uuid' });

// List all crash reports
await invoke('list_crash_reports');
```

### Developer Settings

```typescript
// Get dev settings
await invoke('get_dev_settings');

// Update dev settings
await invoke('update_dev_settings', { settings });
```

## Usage

### Accessing the DevConsole

1. **Via Navigation:** Click the sidebar menu and select "Dev Console"
2. **Via Workspace:** Add a "Developer Console" panel to your workspace
3. **Via Keyboard:** Press the configured hotkey (default: Ctrl+Shift+D)

### Viewing Logs

1. Navigate to the Console tab
2. Use the filter input to search logs
3. Select a minimum log level from the dropdown
4. Click download icon to export logs
5. Click trash icon to clear the log buffer

### Monitoring Build Status

1. Navigate to the Compiler tab
2. View current build status badge
3. Click "Compile Now" to trigger a build
4. Click "Auto Fix" to attempt automatic fixes for errors
5. Errors are displayed with file location and severity

### Checking Performance

1. Navigate to the Performance tab
2. View real-time CPU and memory graphs
3. Monitor process-specific metrics
4. Check for performance bottlenecks

### Analyzing Errors

1. Navigate to the Errors tab
2. View overall error statistics
3. See breakdown by error code
4. Monitor auto-fix success rates

## Configuration

Logger configuration options:

```typescript
{
  minLevel: 'INFO',           // Minimum log level to capture
  consoleEnabled: true,       // Enable console output
  fileEnabled: true,          // Enable file logging
  bufferEnabled: true,        // Enable in-memory buffer
  coloredOutput: true,        // Use colored console output
  includeMetadata: true,      // Include system metrics
  maxFileSizeMb: 100,        // Max log file size
  maxFiles: 10               // Max number of log files to keep
}
```

Developer settings:

```typescript
{
  autoCompilationEnabled: true,
  autoFixEnabled: true,
  logLevel: 'INFO',
  logRetentionDays: 30,
  crashReportingEnabled: true,
  performanceMonitoringEnabled: true,
  autoFixConfidenceThreshold: 0.7,
  maxCompilationRetries: 3,
  developerConsoleHotkey: 'Ctrl+Shift+D'
}
```

## Best Practices

1. **Log Appropriately:** Use the right log level for each message
2. **Include Context:** Add details to help diagnose issues
3. **Monitor Performance:** Check the performance tab regularly
4. **Review Errors:** Investigate patterns in the error statistics
5. **Export Logs:** Save logs before clearing for historical analysis
6. **Test Auto-Fixes:** Verify auto-fixed code works as expected
7. **Keep Logs Manageable:** Clear old logs to avoid storage bloat

## Troubleshooting

**Logs not appearing:**
- Check that the minimum log level includes your messages
- Verify buffer is enabled in logger config
- Check that the logger was initialized properly

**Build status not updating:**
- Manually trigger a compilation
- Check that file watchers are active
- Verify compiler integration is working

**Performance metrics unavailable:**
- Check that sysinfo is properly initialized
- Verify permissions to read system metrics
- Restart the performance monitor if needed

**Auto-fix not working:**
- Check that error messages match fix patterns
- Verify fix attempt limit hasn't been reached
- Review fix history for similar failed attempts

## Future Enhancements

- AI-powered fix suggestions
- Advanced pattern learning
- Remote log aggregation
- Real-time collaborative debugging
- Performance regression detection
- Automated test generation from errors
- Integration with external monitoring services
