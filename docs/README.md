# Eclipse Market Pro - Theme Documentation

This directory contains comprehensive documentation for the Lunar Eclipse design language and theme system.

## 📚 Documentation Files

### [Lunar Eclipse Theme Overview](./lunar-eclipse-theme.md)

A complete reference guide for the Lunar Eclipse design language, including:
- Color palette tokens (Deep Space, Eclipse Orange, Moonlight Silver, Shadow Accent)
- Gradient configurations
- Glassmorphism utility classes
- Accessibility guidelines (WCAG AA compliance)
- Effect variables (glow strength, ambience, glassmorphism)
- Usage examples

**Audience**: Designers, developers, and stakeholders

### [Theme Implementation Guide](./theme-implementation-guide.md)

Step-by-step instructions for implementing the theme in components:
- Quick start guide
- Component examples (modals, cards, navigation)
- Migration tips from legacy styles
- CSS variable reference
- Accessibility checklist
- Testing patterns
- Common UI patterns

**Audience**: Frontend developers

## 🎨 Theme System Architecture

```
┌─────────────────────────────────────────────┐
│         Lunar Eclipse Theme System           │
├─────────────────────────────────────────────┤
│                                              │
│  ┌────────────────┐   ┌──────────────────┐ │
│  │  Theme Store   │──▶│  CSS Variables   │ │
│  │  (Zustand)     │   │  (DOM)           │ │
│  └────────────────┘   └──────────────────┘ │
│         │                      │            │
│         │                      ▼            │
│         │              ┌──────────────────┐ │
│         │              │  Tailwind Config │ │
│         │              │  Utilities       │ │
│         │              └──────────────────┘ │
│         │                      │            │
│         ▼                      ▼            │
│  ┌─────────────────────────────────────┐   │
│  │      React Components               │   │
│  │  (glass-*, lunar-*, eclipse-*)      │   │
│  └─────────────────────────────────────┘   │
│                                              │
└─────────────────────────────────────────────┘
```

### Core Files

1. **Type Definitions**: `src/types/theme.ts`
   - `ThemeColors`: Color tokens interface
   - `ThemeEffects`: Effect controls (glow, ambience, glassmorphism)
   - `Theme`: Complete theme object
   - `ThemePreset`: Predefined theme configuration

2. **Theme Store**: `src/store/themeStore.ts`
   - State management with Zustand
   - Persistence with localStorage
   - Color and effect application to DOM
   - Import/export functionality

3. **Presets**: `src/constants/themePresets.ts`
   - Predefined theme configurations
   - Default effects settings
   - Theme factory function

4. **Styles**:
   - Global CSS: `src/index.css` (variables, utility classes)
   - Tailwind Config: `tailwind.config.js` (custom colors, gradients, shadows)

## 🚀 Getting Started

### For Designers

1. Review the [Lunar Eclipse Theme Overview](./lunar-eclipse-theme.md)
2. Use the color tokens in your design tools:
   - Deep Space: `#050810`
   - Eclipse Orange: `#FF6B35`
   - Moonlight Silver: `#C0CCDA`
   - Shadow Accent: `#1F2937`

### For Developers

1. Read the [Theme Implementation Guide](./theme-implementation-guide.md)
2. Use the glassmorphism classes in your components:
   ```tsx
   <div className="glass-panel rounded-xl p-6">
     <h2 className="eclipse-accent">Title</h2>
     <p className="text-moonlight-silver">Content</p>
   </div>
   ```

3. Test your changes:
   ```bash
   npm test -- themeStore.test.ts
   ```

### For QA/Testers

1. Verify WCAG AA contrast compliance (4.5:1 minimum)
2. Test theme switching in Settings → Appearance
3. Validate glassmorphism toggle functionality
4. Check focus indicators on all interactive elements
5. Test with reduced motion preferences enabled

## 🧪 Testing

Theme store unit tests are located at:
- `src/store/themeStore.test.ts`

Run tests:
```bash
npm test themeStore
```

## 🎛️ Theme Controls

Users can customize the theme through:

1. **Settings → Appearance**
   - Select from preset themes (including Lunar Eclipse)
   - Create custom themes with color pickers
   - Adjust glow strength (none, subtle, normal, strong)
   - Configure ambience (minimal, balanced, immersive)
   - Toggle glassmorphism on/off

2. **Programmatic Access**
   ```tsx
   import { useThemeStore } from '@/store/themeStore';
   
   const { currentTheme, setThemeEffects } = useThemeStore();
   ```

## 🔄 Migration Path

Existing components can be migrated gradually:

1. **Phase 1**: Update backgrounds (bg-gray-900 → eclipse-gradient)
2. **Phase 2**: Apply glassmorphism (bg-slate-900 → glass-panel)
3. **Phase 3**: Update typography colors (text-white → moonlight-text)
4. **Phase 4**: Add glow effects to interactive elements

See the [Implementation Guide](./theme-implementation-guide.md) for detailed migration patterns.

## 📊 Accessibility Standards

All themes must meet:
- ✅ WCAG AA contrast ratios (4.5:1 for normal text, 3:1 for large)
- ✅ Focus indicators visible on all interactive elements
- ✅ Keyboard navigation fully functional
- ✅ Motion respects `prefers-reduced-motion`
- ✅ Color not the only means of conveying information

## 🔗 Related Resources

- [Tailwind CSS Documentation](https://tailwindcss.com/docs)
- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [Glassmorphism Design Trend](https://uxdesign.cc/glassmorphism-in-user-interfaces-1f39bb1308c9)

## 💬 Support

For questions or issues:
1. Check existing documentation in this directory
2. Review component examples in the Implementation Guide
3. Run theme store tests to verify expected behavior
4. Consult the theme store source code for advanced customization

---

**Last Updated**: 2024
**Version**: 1.0.0
**Maintained by**: Eclipse Market Pro Team
