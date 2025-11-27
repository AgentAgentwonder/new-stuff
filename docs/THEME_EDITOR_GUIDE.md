# Theme Editor Guide

## Overview
The Theme Editor allows users to customize the visual appearance of Eclipse Market Pro with full color control, presets, and community sharing capabilities.

## Features

### Theme Presets
Five professionally-designed theme presets are available out of the box:

1. **Aurora Light** - Bright, neutral palette optimized for daytime usage
2. **Nebula Dark** (Default) - High-contrast dark theme ideal for low-light environments
3. **Supernova Contrast** - Certified high contrast theme optimized for accessibility
4. **Deep Sea** - Cool blues and teals inspired by ocean depths
5. **Solar Flare** - Warm gradient-driven theme with vibrant accenting

### Custom Theme Creation

#### Creating a Custom Theme
1. Navigate to **Settings** → **Theme Editor**
2. Click **"Create Custom Theme"**
3. Customize colors using the color picker or hex values
4. Enter a theme name
5. Click **"Save Theme"**

#### Color Customization
The theme editor provides full control over:

- **Background Colors**: Primary, secondary, and tertiary backgrounds
- **Text Colors**: Main text, secondary text, and muted text
- **Brand Colors**: Primary, accent, and hover states
- **Status Colors**: Success, warning, error, info
- **Chart Colors**: Bullish, bearish, and neutral
- **Gradient Colors**: Three-stop gradients for backgrounds

#### Best Practices
- Maintain sufficient color contrast (WCAG AA: 4.5:1 for normal text, 3:1 for large text)
- Test themes with the accessibility high contrast mode
- Use colorblind-friendly palettes when possible
- Provide clear visual hierarchy through color differentiation

### Export & Import

#### Export Theme
1. Select the theme you want to export
2. Click **"Export Theme"** to download as JSON
3. Alternatively, click **"Copy to Clipboard"** for quick sharing

#### Import Theme
1. Click **"Import Theme"**
2. Select a `.json` theme file
3. The theme will be added to your custom themes list

### Community Sharing (Future)

The theme system is built with community sharing hooks:
- Share themes to a community gallery
- Browse and download themes from other users
- Rate and review themes
- Tag themes for easier discovery

## Theme File Format

```json
{
  "id": "custom-theme-1234567890",
  "name": "My Custom Theme",
  "colors": {
    "background": "#0F172A",
    "backgroundSecondary": "#131F3A",
    "backgroundTertiary": "#1E293B",
    "text": "#E2E8F0",
    "textSecondary": "#CBD5F5",
    "textMuted": "#94A3B8",
    "primary": "#6366F1",
    "primaryHover": "#4F46E5",
    "primaryActive": "#4338CA",
    "accent": "#F471B5",
    "accentHover": "#EC4899",
    "success": "#22C55E",
    "warning": "#F97316",
    "error": "#EF4444",
    "info": "#38BDF8",
    "border": "#1E293B",
    "borderHover": "#334155",
    "chartBullish": "#22C55E",
    "chartBearish": "#F87171",
    "chartNeutral": "#6366F1",
    "gradientStart": "#0F172A",
    "gradientMiddle": "#3B0764",
    "gradientEnd": "#1E3A8A"
  },
  "isCustom": true,
  "createdAt": 1234567890000,
  "updatedAt": 1234567890000,
  "author": "Your Name",
  "description": "A beautiful custom theme"
}
```

## Technical Details

### CSS Variables
Themes are applied using CSS custom properties:
- `--color-background`
- `--color-text`
- `--color-primary`
- etc.

These variables are automatically updated when themes change and persist across sessions.

### Theme Application
Themes apply to:
- All UI components (buttons, inputs, panels)
- Chart visualizations
- Workspace backgrounds
- Border colors and shadows

### State Management
Themes are managed via Zustand with localStorage persistence:
- Current theme selection persists across sessions
- Custom themes are stored locally
- Theme changes apply instantly without reload

## Troubleshooting

### Theme not applying
- Ensure you've clicked "Save Theme" after customization
- Check browser console for any error messages
- Try refreshing the page

### Colors look washed out
- Increase contrast between text and background
- Enable high contrast mode in accessibility settings
- Use the Supernova Contrast preset as a reference

### Import fails
- Verify JSON file format matches the specification
- Ensure all required color fields are present
- Check that hex color codes are valid

## Keyboard Shortcuts

- `Tab` - Navigate between color inputs
- `Enter` - Confirm color selection
- `Escape` - Close theme editor modal

## See Also

- [Accessibility Guide](./ACCESSIBILITY_GUIDE.md) - Accessibility features including high contrast mode
- [Settings Documentation](./README.md#settings) - General settings configuration
