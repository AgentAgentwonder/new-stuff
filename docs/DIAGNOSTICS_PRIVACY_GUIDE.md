# Diagnostics & Crash Reporting - Privacy Guide

## Overview

Eclipse Market Pro implements session recording and crash reporting features to improve user experience and application stability. This guide outlines the privacy implications, data handling practices, and user controls.

## Session Recording

### What is Recorded

When session recording is enabled, the application captures:

- **UI Interactions**: Mouse movements, clicks, scrolling, and keyboard events
- **DOM Changes**: Dynamic updates to the user interface
- **Console Logs**: JavaScript console output (info, warn, error, debug)
- **JavaScript Errors**: Uncaught exceptions and promise rejections
- **Navigation**: Page changes and routing events

### What is NOT Recorded

- **Input Values**: Text entered in password fields and other sensitive inputs (when privacy masking is enabled)
- **Network Request Bodies**: Only metadata is captured, not actual request/response payloads
- **Binary Data**: Images, videos, and other binary content
- **Third-party Content**: Content from external iframes and domains

### Privacy Controls

1. **Opt-in Required**: Session recording is completely disabled by default and requires explicit user consent
2. **Privacy Masking**: Sensitive input fields and text are automatically masked using CSS classes
3. **Time-Limited Storage**: Recordings are automatically deleted after 30 minutes
4. **Local Storage**: All recordings are stored locally on the user's device
5. **Manual Export Only**: Recordings never leave the device unless the user explicitly exports them
6. **Instant Disable**: Users can stop recording at any time from Settings

### Privacy Masking Implementation

The following elements are automatically masked:

- Input fields with `type="password"`
- Input fields with `autocomplete="cc-number"`, `autocomplete="cc-cvc"`, etc.
- Elements with the `rrweb-mask` CSS class
- Text within elements having the `rrweb-mask-text` CSS class

To mark custom elements for masking:

```html
<!-- Mask entire element and its children -->
<div class="rrweb-mask">
  Sensitive content here
</div>

<!-- Mask only text content -->
<div class="rrweb-mask-text">
  This text will be masked
</div>
```

### Data Retention

- **Active Recording**: Last 30 minutes of activity
- **Stored Recordings**: Automatically pruned after 30 minutes
- **Manual Deletion**: Users can delete individual recordings at any time
- **Consent Revocation**: Revoking consent stops recording but preserves existing recordings

## Crash Reporting

### What is Sent

When crash reporting is enabled and a crash occurs:

- **Error Information**: Error message and stack trace
- **Component Stack**: React component hierarchy at crash time
- **Environment Data**:
  - Platform (OS)
  - Browser/runtime version
  - Screen resolution and viewport size
  - Page URL
  - Browser language
- **Session Context**: Session ID if session recording is also enabled
- **User Feedback**: Optional comments provided by the user

### What is NOT Sent

- **Personal Information**: No names, email addresses, or contact information
- **Authentication Tokens**: Access tokens and API keys are filtered
- **Private Keys**: Wallet private keys and sensitive cryptographic material
- **User-Generated Content**: Chat messages, notes, or documents
- **Financial Data**: Account balances, transaction amounts, or wallet addresses

### Privacy Controls

1. **Opt-in Required**: Crash reporting requires explicit user consent
2. **Automatic Filtering**: Sensitive data is automatically filtered before transmission
3. **User Feedback Control**: Users can choose whether to add comments to crash reports
4. **Local Storage**: Crash reports are stored locally for review
5. **Manual Review**: Users can view all crash reports before they're sent (if Sentry is configured)
6. **Anonymization**: Reports are anonymized and don't include identifying information

### Sentry Integration

Sentry is used as the crash reporting backend (optional):

- **Configuration**: Requires `VITE_SENTRY_DSN` environment variable
- **Disabled by Default**: If no DSN is configured, Sentry remains disabled
- **Development Mode**: Crash reporting in dev mode doesn't send data to Sentry
- **Production Mode**: 20% of transactions are sampled for performance monitoring
- **User IP**: Not collected or stored

## Auto-Restart Feature

### Behavior

When enabled (default), the application will:

1. Catch critical errors that would crash the app
2. Display an error screen with crash details
3. Offer to automatically restart the application
4. Save crash report locally (if crash reporting is enabled)

### User Control

Users can disable auto-restart from Settings:

- When disabled, users must manually acknowledge errors
- Errors still display with option to manually restart
- Crash reports are still captured if enabled

## Data Storage Locations

### Local Storage

All diagnostic data is stored in browser LocalStorage under the following keys:

- `diagnostics-storage`: Session recordings, crash reports, and settings

### Session Storage

No diagnostic data is stored in SessionStorage.

### IndexedDB

Not used for diagnostics features.

## User Rights

Users have the following rights regarding their diagnostic data:

1. **Right to Access**: View all stored recordings and crash reports
2. **Right to Export**: Download recordings in JSON format
3. **Right to Delete**: Remove individual recordings or crash reports
4. **Right to Revoke Consent**: Disable features at any time
5. **Right to Data Minimization**: Privacy masking limits data collection

## Compliance

### GDPR Compliance

- **Lawful Basis**: Consent (Article 6(1)(a))
- **Data Minimization**: Only necessary data is collected (Article 5(1)(c))
- **Storage Limitation**: 30-minute retention period (Article 5(1)(e))
- **Security**: Client-side encryption possible through browser storage encryption
- **Right to Erasure**: Users can delete all diagnostic data

### CCPA Compliance

- **Notice at Collection**: Users are informed before data collection begins
- **Right to Know**: Users can view all stored data
- **Right to Delete**: Deletion available through Settings
- **Opt-in Required**: No data collected without explicit consent

## Best Practices for Developers

When adding new features:

1. **Mark Sensitive Fields**: Add `rrweb-mask` class to sensitive elements
2. **Avoid Logging Secrets**: Don't log tokens, keys, or passwords to console
3. **Test Privacy Masking**: Verify sensitive data is masked in recordings
4. **Error Sanitization**: Ensure error messages don't contain sensitive data
5. **Review Crash Reports**: Check that crash reports don't expose private information

## Opt-in Flows

### First-Time Setup

When users first access the Diagnostics settings:

1. Both features are disabled by default
2. Clear explanations of what data is collected
3. Separate consent for session recording and crash reporting
4. Privacy protections clearly outlined
5. User can decline without affecting app functionality

### Consent Dialog Content

The consent dialog includes:

- Feature description and benefits
- List of data collected
- Privacy protections implemented
- Data retention period
- User's right to revoke consent
- Accept/Decline buttons with equal visual weight

## Security Considerations

### Client-Side Security

- Recordings never transmitted automatically
- Local storage encryption depends on browser/OS security
- No cloud storage by default
- Export requires user action

### Network Security

- Sentry transmissions use HTTPS
- Rate limiting prevents data leakage
- DSN can be rotated if compromised

### Crash Report Filtering

Automatic filtering for:

- Environment variables containing 'KEY', 'TOKEN', 'SECRET'
- URL parameters with sensitive names
- Headers with authentication information
- Stack traces from secure storage modules

## Support and Troubleshooting

### Debugging Issues

If session recording doesn't work:

1. Check browser console for errors
2. Verify consent has been granted
3. Ensure privacy settings allow local storage
4. Try disabling browser extensions that block tracking

If crash reporting doesn't work:

1. Verify `VITE_SENTRY_DSN` is configured (if using Sentry)
2. Check browser network tab for failed requests
3. Review crash dashboard for local reports
4. Ensure browser allows third-party requests (if using Sentry)

### Exporting Support Packages

To help support diagnose issues:

1. Enable session recording and crash reporting
2. Reproduce the issue
3. Export the relevant recording from Settings > Diagnostics
4. Include crash report ID if applicable
5. Send exported JSON to support via secure channel

### Privacy-Safe Sharing

When sharing diagnostic data with support:

1. Review exported recording in replay viewer first
2. Ensure no sensitive data is visible
3. Delete recording after support resolves issue
4. Use encrypted communication channels
5. Never share recordings publicly

## Updates to Privacy Practices

This privacy guide is versioned alongside the application. When privacy practices change:

- Users will be re-prompted for consent if changes are material
- Changelog will document privacy-related changes
- Existing consent may be invalidated requiring opt-in again

## Contact

For privacy-related questions or concerns:

- Review this document and in-app privacy notices
- Check the main application privacy policy
- Contact support through official channels

---

**Last Updated**: 2024
**Version**: 1.0.0
