# Lunar Eclipse Animation System

A comprehensive guide to the animation primitives and components in the Eclipse Market Pro application.

## Overview

The animation system is built on Framer Motion and provides reusable, accessible animation primitives inspired by lunar and eclipse themes. All animations respect user accessibility preferences, particularly the `prefers-reduced-motion` setting.

## Animation Tokens

### Duration Tokens

Located in `src/utils/animations.ts`:

```typescript
ANIMATION_DURATIONS = {
  instant: 0.01,    // Used for reduced motion or immediate transitions
  fast: 0.15,       // Quick interactions (hover, click)
  normal: 0.3,      // Standard transitions
  slow: 0.5,        // Deliberate, noticeable animations
  glacial: 0.8,     // Slow, ambient effects
}
```

### Easing Tokens

```typescript
ANIMATION_EASINGS = {
  smooth: [0.4, 0, 0.2, 1],           // General-purpose smooth easing
  bounce: [0.68, -0.55, 0.265, 1.55], // Playful bounce effect
  orbital: [0.645, 0.045, 0.355, 1],  // Celestial circular motion
  eclipse: [0.25, 0.46, 0.45, 0.94],  // Fade/reveal effects
}
```

## Animation Primitives

### Orbital Variants

For elements with circular or elliptical motion:

```tsx
import { orbitalVariants } from '@/utils/animations';

<motion.div variants={orbitalVariants} initial="hidden" animate="visible">
  Content
</motion.div>
```

### Corona Glow Variants

For pulsing glow effects:

```tsx
import { coronaGlowVariants } from '@/utils/animations';

<motion.div variants={coronaGlowVariants} animate="glow">
  Glowing element
</motion.div>
```

### Constellation Link Variants

For animated line drawings:

```tsx
import { constellationLinkVariants } from '@/utils/animations';

<motion.line variants={constellationLinkVariants} initial="hidden" animate="visible" />
```

### Panel Reveal Variants

For sliding panels and modals:

```tsx
import { panelRevealVariants } from '@/utils/animations';

<motion.div variants={panelRevealVariants} initial="hidden" animate="visible" exit="exit">
  Panel content
</motion.div>
```

### Card Hover Variants

For interactive card effects:

```tsx
import { cardHoverVariants } from '@/utils/animations';

<motion.div variants={cardHoverVariants} whileHover="hover">
  Card content
</motion.div>
```

### Fade In Stagger

For staggered list animations:

```tsx
import { fadeInStaggerVariants } from '@/utils/animations';

<motion.ul variants={fadeInStaggerVariants.container} initial="hidden" animate="visible">
  <motion.li variants={fadeInStaggerVariants.item}>Item 1</motion.li>
  <motion.li variants={fadeInStaggerVariants.item}>Item 2</motion.li>
</motion.ul>
```

## Shared Components

### EclipseLoader

An animated loading indicator with sun/moon eclipse animation.

```tsx
import { EclipseLoader } from '@/components/common/EclipseLoader';

<EclipseLoader size="md" /> // size: 'sm' | 'md' | 'lg'
```

**Accessibility**: Automatically respects reduced motion settings, includes proper ARIA labels.

### MoonPhaseIndicator

Displays the current moon phase with animated rotation.

```tsx
import { MoonPhaseIndicator } from '@/components/common/MoonPhaseIndicator';

<MoonPhaseIndicator 
  phase={0.5}         // 0-1, representing lunar cycle position
  size={48}           // Size in pixels
  showLabel={true}    // Show phase label
/>
```

**Phases**:
- 0.0 = New Moon
- 0.25 = First Quarter
- 0.5 = Full Moon
- 0.75 = Last Quarter

### ProgressBar

Animated progress bar with multiple variants and accessibility support.

```tsx
import { ProgressBar } from '@/components/common/ProgressBar';

<ProgressBar 
  value={75}                        // 0-100
  label="Loading..."                // Optional label
  showPercentage={true}             // Show percentage text
  size="md"                         // 'sm' | 'md' | 'lg'
  variant="primary"                 // 'primary' | 'success' | 'warning' | 'error'
  indeterminate={false}             // Show shimmer loading effect
/>
```

### ConstellationBackground

Animated constellation background with stars and connecting lines.

```tsx
import { ConstellationBackground } from '@/components/common/ConstellationBackground';

<ConstellationBackground 
  starCount={50}      // Number of stars
  linkCount={30}      // Number of constellation lines
  opacity={0.3}       // Overall opacity
/>
```

**Usage**: Place as a background layer with `absolute` positioning.

### Skeleton (Enhanced)

Loading skeleton with shimmer effect.

```tsx
import { Skeleton } from '@/components/common/Skeleton';

<Skeleton 
  width="100%"
  height="1rem"
  rounded="0.75rem"
  variant="shimmer"   // 'default' | 'shimmer'
/>
```

## Hooks

### useMotionPreferences

Combines user settings and system preferences to determine reduced motion state.

```tsx
import { useMotionPreferences } from '@/hooks/useMotionPreferences';

const reducedMotion = useMotionPreferences();

// Use in component logic
const animationProps = reducedMotion ? {} : { animate: true };
```

### useParallax

Creates scroll-based parallax effects.

```tsx
import { useParallax } from '@/hooks/useParallax';

const { ref, style } = useParallax({ distance: 100 });

<motion.div ref={ref} style={style}>
  Content with parallax
</motion.div>
```

### useParallaxLayer

Creates depth-based parallax for background layers.

```tsx
import { useParallaxLayer } from '@/hooks/useParallax';

const parallaxStyle = useParallaxLayer(0.2); // 0 = no movement, 1 = full movement

<motion.div style={parallaxStyle}>
  Background layer
</motion.div>
```

### useAmbientScroll

Creates ambient motion effects on scroll.

```tsx
import { useAmbientScroll } from '@/hooks/useParallax';

const ambientStyle = useAmbientScroll();

<motion.div style={ambientStyle}>
  Content with ambient effects
</motion.div>
```

## Accessibility

### Reduced Motion Support

All animations automatically respect the following:

1. **User Setting**: `useAccessibilityStore` → `reducedMotion`
2. **System Preference**: `prefers-reduced-motion` media query
3. **CSS Class**: `.reduce-motion` applied to root element

### Implementing Accessible Animations

Always use the accessibility utilities:

```tsx
import { useMotionPreferences } from '@/hooks/useMotionPreferences';
import { getAccessibleVariants } from '@/utils/animations';

const reducedMotion = useMotionPreferences();
const accessibleVariants = getAccessibleVariants(myVariants, reducedMotion);

<motion.div variants={accessibleVariants}>
  Content
</motion.div>
```

### Testing Accessibility

Run the animation accessibility tests:

```bash
npm test animations.test.tsx
```

These tests ensure:
- Animations respect reduced motion settings
- Components have proper ARIA labels
- Transitions are stripped when accessibility mode is enabled
- Performance is maintained through memoization

## Performance Guidelines

### 1. Memoize Heavy Components

All animation components use `React.memo`:

```tsx
export const MyComponent = React.memo(({ prop1, prop2 }) => {
  // Component logic
});
```

### 2. Use Transform Properties

Prefer `transform` and `opacity` for animations (GPU-accelerated):

```tsx
// Good
<motion.div animate={{ x: 100, opacity: 0.5 }} />

// Avoid
<motion.div animate={{ left: 100 }} />
```

### 3. Limit Constellation Complexity

For backgrounds, keep star/link counts reasonable:
- Dashboard: 30-50 stars, 15-30 links
- Modals: 20-30 stars, 10-15 links
- Cards: 10-20 stars, 5-10 links

### 4. Use Will-Change Sparingly

Only apply to elements that will definitely animate:

```css
.animated-element {
  will-change: transform, opacity;
}
```

## Examples

### Dashboard with Parallax Background

```tsx
import { useParallaxLayer } from '@/hooks/useParallax';
import { ConstellationBackground } from '@/components/common/ConstellationBackground';

export function Dashboard() {
  const parallaxBackground = useParallaxLayer(0.2);

  return (
    <div className="relative">
      <motion.div style={parallaxBackground} className="absolute inset-0">
        <ConstellationBackground starCount={30} linkCount={15} opacity={0.2} />
      </motion.div>
      
      <div className="relative">
        {/* Dashboard content */}
      </div>
    </div>
  );
}
```

### Loading State with Eclipse Loader

```tsx
import { EclipseLoader } from '@/components/common/EclipseLoader';

export function DataView({ loading, data }) {
  if (loading) {
    return (
      <div className="flex justify-center items-center h-64">
        <EclipseLoader size="lg" />
      </div>
    );
  }

  return <DataDisplay data={data} />;
}
```

### Progress with Moon Phase

```tsx
import { ProgressBar } from '@/components/common/ProgressBar';
import { MoonPhaseIndicator } from '@/components/common/MoonPhaseIndicator';

export function UploadProgress({ progress }) {
  const phase = progress / 100;

  return (
    <div>
      <div className="flex justify-center mb-4">
        <MoonPhaseIndicator phase={phase} />
      </div>
      <ProgressBar value={progress} label="Uploading..." showPercentage />
    </div>
  );
}
```

### Staggered Card Grid

```tsx
import { fadeInStaggerVariants, cardHoverVariants } from '@/utils/animations';

export function CardGrid({ items }) {
  return (
    <motion.div 
      className="grid grid-cols-3 gap-4"
      variants={fadeInStaggerVariants.container}
      initial="hidden"
      animate="visible"
    >
      {items.map(item => (
        <motion.div
          key={item.id}
          variants={fadeInStaggerVariants.item}
          whileHover="hover"
        >
          <Card {...item} />
        </motion.div>
      ))}
    </motion.div>
  );
}
```

## Migration Guide

### Updating Existing Components

1. **Import motion utilities**:
   ```tsx
   import { useMotionPreferences } from '@/hooks/useMotionPreferences';
   import { getAccessibleVariants } from '@/utils/animations';
   ```

2. **Check for reduced motion**:
   ```tsx
   const reducedMotion = useMotionPreferences();
   ```

3. **Apply accessible variants**:
   ```tsx
   const variants = getAccessibleVariants(myVariants, reducedMotion);
   ```

4. **Update animations conditionally**:
   ```tsx
   <motion.div 
     animate={reducedMotion ? {} : { x: 100 }}
     transition={reducedMotion ? { duration: 0.01 } : { duration: 0.3 }}
   />
   ```

## Browser Support

- Chrome/Edge: Full support
- Firefox: Full support
- Safari: Full support
- Mobile browsers: Full support with performance considerations

## Troubleshooting

### Animations Not Playing

1. Check if reduced motion is enabled: Open browser DevTools → Settings → Emulate CSS media → prefers-reduced-motion
2. Verify accessibility store: `useAccessibilityStore.getState().reducedMotion`
3. Check for CSS conflicts with `.reduce-motion` class

### Performance Issues

1. Reduce constellation complexity
2. Use `transform` instead of position properties
3. Enable GPU acceleration in browser settings
4. Check PerformanceMonitor for FPS metrics

### Accessibility Issues

1. Always include ARIA labels on animated components
2. Test with screen readers
3. Run automated accessibility tests: `npm test animations.test.tsx`
4. Verify keyboard navigation still works

## Contributing

When adding new animations:

1. Add tokens to `src/utils/animations.ts`
2. Create memoized components with React.memo
3. Support reduced motion via `useMotionPreferences`
4. Add ARIA labels for screen readers
5. Write tests in `src/__tests__/animations.test.tsx`
6. Document in this guide

## Resources

- [Framer Motion Documentation](https://www.framer.com/motion/)
- [Web Animations API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Animations_API)
- [Reduced Motion Guide](https://web.dev/prefers-reduced-motion/)
- [WCAG Animation Guidelines](https://www.w3.org/WAI/WCAG21/Understanding/animation-from-interactions.html)
