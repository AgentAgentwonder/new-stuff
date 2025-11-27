# Accessibility Features

## Overview
Eclipse Market Pro is designed to be inclusive and accessible for traders with diverse needs. This document outlines the accessibility improvements delivered in this iteration (Phase 7 Tasks 7.5–7.6).

## Functional Accessibility Options

### 1. Screen Reader Optimizations
- ARIA labels and descriptions added to navigation, buttons, and forms
- Form elements have discernible labels and are announced appropriately
- Important updates include alerts for theme import/export feedback

### 2. Keyboard Navigation
- All interactive elements are reachable via keyboard (`Tab`, `Shift + Tab`)
- Focus indicators are visible, with optional enhanced outlines for higher visibility
- Navigation menus support arrow key selection and focus management

### 3. Font Scaling (100%–200%)
- Font size adjustable with the accessibility slider in Settings → Accessibility
- Root font scaling applies to all components and charts
- Font scaling persists between sessions

### 4. High Contrast Mode
- Toggle increases contrast and border visibility across the app
- Works alongside theme presets (High Contrast theme is also available)
- Useful for low-vision users and bright environments

### 5. Reduced Motion Toggle
- Disables animations and transitions for motion-sensitive users
- Honors system preferences (`prefers-reduced-motion`), but allows manual override

### 6. Alt Text & Icon Labels
- Icons throughout the app include accessible names via `aria-label`
- Decorative icons are marked with `aria-hidden="true"`
- Navigation items expose semantic roles for screen reader context

### 7. Community Themes Accessibility
- Shared themes include metadata (author, description, tags)
- Themes are validated against contrast best practices during sharing

## Accessibility Panel

Located under **Settings → Accessibility**, providing:
- Font scale slider
- Toggles for high contrast, reduced motion, screen reader support, keyboard hints, enhanced focus indicators
- Reset-to-defaults button

## Automated Testing
- Introduced automated accessibility tests via axe-core in the testing suite
- Checks for common issues: missing ARIA labels, color contrast violations, keyboard traps

## Best Practices for Contributors
- Use semantic HTML and ARIA roles appropriately
- Maintain focus management during modal interactions
- Provide alternative text for new icons/images
- Avoid using color as the sole means of conveying information

## Future Work
- Community theme marketplace with accessibility scoring
- Accessibility audit dashboard for components
- Localization support for screen reader messages

## References
- [WCAG 2.1 Guidelines](https://www.w3.org/TR/WCAG21/)
- [WAI-ARIA Authoring Practices](https://www.w3.org/TR/wai-aria-practices/)
