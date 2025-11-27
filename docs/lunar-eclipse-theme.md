# Lunar Eclipse Design Language

The Lunar Eclipse system introduces gradient-driven surfaces, deep space palettes, and glassmorphism overlays to the Eclipse Market Pro interface. This guide describes the available tokens, recommended usage, and accessibility guidelines for the refreshed theme.

## Palette Tokens

| Token | CSS Variable | Default Value | Usage |
|-------|--------------|---------------|-------|
| Deep Space | `--color-deep-space` | `#050810` | Application background and elevated surfaces |
| Eclipse Orange | `--color-eclipse-orange` | `#FF6B35` | Primary actions, glow effects, highlight strokes |
| Moonlight Silver | `--color-moonlight-silver` | `#C0CCDA` | Secondary text, subtle dividers |
| Shadow Accent | `--color-shadow-accent` | `#1F2937` | Borders, separator bars, muted surfaces |

### Gradient Tokens

The primary background is a 3-stop gradient using:

- `--color-gradient-start`: `#0A0E1A`
- `--color-gradient-middle`: `#1A1F35`
- `--color-gradient-end`: `#2A1F3D`

Use the Tailwind utility `bg-lunar-gradient` or the CSS helper class `.eclipse-gradient` to apply the gradient background to containers and pages.

## Effects

Theme effects are configurable via the theme store and applied to the DOM as CSS variables:

- `--effect-glow-strength`: modifies the intensity of `lunar-glow` shadows.
- `--effect-ambience`: controls atmospheric shadows for panels (`0.1 – 0.4`).
- `--glass-opacity`: determines the base alpha channel for glassmorphism panes.
- `--glass-border-opacity`: adjusts outlines for glass components.

These values can be adjusted using the `setThemeEffects` action exposed by the `useThemeStore` hook.

## Glassmorphism Utilities

New classes are available globally:

- `.glass-panel`: Elevated surfaces with blur and specular highlights.
- `.glass-card`: Dense glass surface for cards and widgets.
- `.glass-header`: Sticky headers with increased blur and eclipse accent border.
- `.lunar-glow` & `.lunar-glow-strong`: Box-shadows keyed to glow strength.
- `.moonlight-text` & `.eclipse-accent`: Typographic helpers for contrast presets.

These utilities respect the `glass-enabled` class on the `<html>` element. Disabling glassmorphism removes blur and lowers opacity while preserving layout integrity.

## Accessibility

- **Contrast**: Default palette maintains WCAG AA (4.5:1) for body text on glass surfaces. Avoid lowering glass opacity below `0.25` when using foggy backgrounds.
- **Focus States**: Focus rings inherit from `--color-accent-hover`, ensuring a minimum 3:1 contrast on dark backgrounds.
- **Motion**: Respect `prefers-reduced-motion`—the theme retains existing reduced-motion rules and color transitions degrade gracefully.

## Storybook Usage

For Storybook or component documentation:

```tsx
<div className="min-h-screen bg-lunar-gradient text-moonlight-silver space-y-6 p-10">
  <header className="glass-header px-6 py-4 rounded-2xl flex items-center justify-between">
    <h1 className="text-2xl font-semibold eclipse-accent lunar-glow">Lunar Eclipse</h1>
  </header>
  <section className="grid md:grid-cols-2 gap-6">
    <article className="glass-card rounded-3xl p-6 space-y-3">
      <h2 className="text-xl font-semibold">Glass Cards</h2>
      <p className="text-moonlight-silver/80">Use for widgets, market tickers, and contextual data.</p>
    </article>
    <article className="glass-panel rounded-3xl p-6 space-y-3">
      <h2 className="text-xl font-semibold">Glow States</h2>
      <button className="px-4 py-2 rounded-full bg-eclipse-orange text-deep-space font-semibold lunar-glow">Primary Action</button>
    </article>
  </section>
</div>
```

## Implementation Notes

- The Tailwind config exposes `bg-lunar-gradient`, `bg-eclipse-radial`, and glow shadows (`shadow-glow-subtle`, `shadow-glow-normal`, `shadow-glow-strong`).
- Theme persistence now stores `effects` alongside color tokens. Rehydration automatically reapplies both color and effect variables.

Use this document as a reference when building new components or adapting existing ones to the Lunar Eclipse system.
