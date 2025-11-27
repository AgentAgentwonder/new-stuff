# Theme Implementation Guide

This guide provides step-by-step instructions for implementing the Lunar Eclipse theme system in new components or refactoring existing ones.

## Quick Start

### 1. Apply Background Gradients

Replace static backgrounds with the new gradient system:

```tsx
// Before
<div className="min-h-screen bg-gray-900">

// After - Lunar Eclipse
<div className="min-h-screen eclipse-gradient">
```

### 2. Use Glassmorphism Panels

Apply glass effect classes to cards, modals, and elevated surfaces:

```tsx
// Card components
<div className="glass-card rounded-2xl p-6">
  {/* content */}
</div>

// Panel components (sidebar, navigation)
<div className="glass-panel rounded-xl p-4 border" style={{ borderColor: 'rgba(255, 140, 66, 0.2)' }}>
  {/* content */}
</div>

// Header/sticky sections
<header className="glass-header sticky top-0 z-50">
  {/* content */}
</header>
```

### 3. Apply Typography Colors

Use semantic color classes for text hierarchy:

```tsx
// Primary headings
<h1 className="text-2xl font-bold eclipse-accent">Title</h1>

// Body text
<p className="text-moonlight-silver">Content text</p>

// Muted/secondary text
<span className="text-[var(--color-text-muted)]">Subtitle</span>
```

### 4. Add Glow Effects

Apply glow to accent elements, buttons, and interactive components:

```tsx
// Subtle glow for cards
<div className="glass-card lunar-glow">

// Strong glow for CTAs
<button className="px-6 py-3 bg-[var(--color-eclipse-orange)] lunar-glow-strong rounded-xl">
  Primary Action
</button>
```

## CSS Variable Reference

Use these CSS custom properties in inline styles or CSS modules:

### Colors
- `--color-deep-space`: Deep background color
- `--color-eclipse-orange`: Primary action/accent color
- `--color-moonlight-silver`: Secondary text color
- `--color-shadow-accent`: Border/separator color

### Effects
- `--effect-glow-strength`: Current glow opacity (0–0.65)
- `--effect-ambience`: Atmospheric overlay opacity (0.1–0.4)
- `--glass-opacity`: Glass surface transparency
- `--glass-border-opacity`: Glass border alpha

## Component Examples

### Modal Dialog

```tsx
<motion.div className="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-center justify-center">
  <div className="glass-panel rounded-3xl p-8 max-w-2xl w-full shadow-2xl">
    <h2 className="text-2xl font-bold eclipse-accent mb-4">Modal Title</h2>
    <p className="text-moonlight-silver mb-6">Modal content goes here...</p>
    <button className="px-4 py-2 bg-[var(--color-eclipse-orange)] text-[var(--color-deep-space)] rounded-lg lunar-glow">
      Confirm
    </button>
  </div>
</motion.div>
```

### Data Card

```tsx
<div className="glass-card rounded-2xl p-6 space-y-4">
  <div className="flex items-center justify-between">
    <h3 className="text-lg font-semibold text-moonlight-silver">Market Cap</h3>
    <span className="text-2xl font-bold eclipse-accent">$2.4B</span>
  </div>
  <div className="h-px bg-gradient-to-r from-[var(--color-eclipse-orange)] to-transparent opacity-30" />
  <p className="text-sm text-[var(--color-text-muted)]">Last 24 hours</p>
</div>
```

### Navigation Item (Active State)

```tsx
<button
  className={`px-4 py-3 rounded-xl transition-all ${
    isActive
      ? 'glass-card lunar-glow'
      : 'glass-panel hover:-translate-y-0.5'
  }`}
  style={isActive ? { borderColor: 'rgba(255, 140, 66, 0.45)' } : undefined}
>
  <Icon className={isActive ? 'text-[var(--color-eclipse-orange)]' : 'text-moonlight-silver'} />
  <span className={isActive ? 'text-[var(--color-eclipse-orange)]' : 'text-moonlight-silver'}>
    {label}
  </span>
</button>
```

## Accessibility Checklist

When implementing the Lunar Eclipse theme:

- [ ] Verify text contrast ratios meet WCAG AA (4.5:1 for body, 3:1 for large text)
- [ ] Test focus indicators on all interactive elements
- [ ] Ensure glassmorphism doesn't obscure critical content when layered
- [ ] Test with `prefers-reduced-motion` enabled (motion should gracefully degrade)
- [ ] Verify keyboard navigation works with updated styles

## Theme Store Integration

To read or update theme effects programmatically:

```tsx
import { useThemeStore } from '@/store/themeStore';

function MyComponent() {
  const { currentTheme, setThemeEffects } = useThemeStore();

  const toggleGlassmorphism = () => {
    setThemeEffects({
      ...currentTheme.effects,
      glassmorphism: !currentTheme.effects?.glassmorphism,
    });
  };

  return (
    <button onClick={toggleGlassmorphism}>
      {currentTheme.effects?.glassmorphism ? 'Disable' : 'Enable'} Glass
    </button>
  );
}
```

## Testing

When adding tests for components using the theme:

```tsx
import { renderHook } from '@testing-library/react';
import { useThemeStore } from '@/store/themeStore';

beforeEach(() => {
  const { result } = renderHook(() => useThemeStore());
  act(() => {
    result.current.setThemeFromPreset('lunar-eclipse');
  });
});

it('should apply glassmorphism classes', () => {
  const { container } = render(<MyComponent />);
  expect(container.querySelector('.glass-panel')).toBeInTheDocument();
});
```

## Migration Tips

### From Slate Backgrounds

Replace static Slate backgrounds with glass panels:

```tsx
// Before
className="bg-slate-900/95 backdrop-blur-xl border border-purple-500/20"

// After
className="glass-panel"
style={{ borderColor: 'rgba(255, 140, 66, 0.2)' }}
```

### From Gradient Backgrounds

Update gradient utilities:

```tsx
// Before
className="bg-gradient-to-r from-purple-500 to-pink-500"

// After (for backgrounds)
className="eclipse-gradient"

// After (for accent buttons)
style={{
  background: 'linear-gradient(135deg, rgba(255, 107, 53, 0.92), rgba(255, 140, 66, 0.85))'
}}
```

### Typography Updates

```tsx
// Before
className="text-white"

// After - for headings
className="text-moonlight-silver"

// After - for accents
className="eclipse-accent"
```

## Common Patterns

### Dividers

```tsx
<div className="h-px bg-gradient-to-r from-[rgba(255,140,66,0.45)] via-[rgba(78,205,196,0.25)] to-transparent" />
```

### Hover States

```tsx
className="glass-panel hover:-translate-y-0.5 transition-transform"
```

### Active State Glow

```tsx
<button
  className={isActive ? 'lunar-glow' : ''}
  style={{
    boxShadow: isActive ? '0 0 30px rgba(255, 107, 53, var(--effect-glow-strength))' : undefined
  }}
>
```

## Resources

- [Lunar Eclipse Theme Overview](./lunar-eclipse-theme.md)
- [Theme Store Tests](../src/store/themeStore.test.ts)
- [Tailwind Config](../tailwind.config.js)
- [Global Styles](../src/index.css)
