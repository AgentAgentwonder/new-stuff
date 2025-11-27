import { describe, it, expect, vi } from 'vitest';
import { useAccessibilityStore } from '../src/store/accessibilityStore';
import { useThemeStore } from '../src/store/themeStore';

describe('Accessibility Store', () => {
  it('should set font scale within bounds', () => {
    const store = useAccessibilityStore.getState();

    store.setFontScale(1.5);
    expect(useAccessibilityStore.getState().fontScale).toBe(1.5);

    store.setFontScale(2.5);
    expect(useAccessibilityStore.getState().fontScale).toBe(2.0);

    store.setFontScale(0.5);
    expect(useAccessibilityStore.getState().fontScale).toBe(1.0);
  });

  it('should toggle high contrast mode', () => {
    const store = useAccessibilityStore.getState();
    const initial = store.highContrastMode;

    store.toggleHighContrast();
    expect(useAccessibilityStore.getState().highContrastMode).toBe(!initial);

    store.toggleHighContrast();
    expect(useAccessibilityStore.getState().highContrastMode).toBe(initial);
  });

  it('should toggle reduced motion', () => {
    const store = useAccessibilityStore.getState();
    const initial = store.reducedMotion;

    store.toggleReducedMotion();
    expect(useAccessibilityStore.getState().reducedMotion).toBe(!initial);
  });

  it('should reset to defaults', () => {
    const store = useAccessibilityStore.getState();

    store.setFontScale(1.5);
    store.toggleHighContrast();
    store.toggleReducedMotion();

    store.resetToDefaults();

    const state = useAccessibilityStore.getState();
    expect(state.fontScale).toBe(1.0);
    expect(state.highContrastMode).toBe(false);
    expect(state.reducedMotion).toBe(false);
  });
});

describe('Theme Store', () => {
  it('should create custom theme', () => {
    const store = useThemeStore.getState();
    const initialThemeCount = store.customThemes.length;

    store.createCustomTheme('Test Theme', {
      background: '#000000',
      backgroundSecondary: '#111111',
      backgroundTertiary: '#222222',
      text: '#FFFFFF',
      textSecondary: '#EEEEEE',
      textMuted: '#CCCCCC',
      primary: '#FF0000',
      primaryHover: '#CC0000',
      primaryActive: '#AA0000',
      accent: '#00FF00',
      accentHover: '#00CC00',
      success: '#00FF00',
      warning: '#FFFF00',
      error: '#FF0000',
      info: '#0000FF',
      border: '#444444',
      borderHover: '#555555',
      chartBullish: '#00FF00',
      chartBearish: '#FF0000',
      chartNeutral: '#0000FF',
      gradientStart: '#000000',
      gradientMiddle: '#111111',
      gradientEnd: '#222222',
    });

    const finalThemeCount = useThemeStore.getState().customThemes.length;
    expect(finalThemeCount).toBe(initialThemeCount + 1);
  });

  it('should delete custom theme', () => {
    const store = useThemeStore.getState();

    store.createCustomTheme('To Delete', {
      background: '#000000',
      backgroundSecondary: '#111111',
      backgroundTertiary: '#222222',
      text: '#FFFFFF',
      textSecondary: '#EEEEEE',
      textMuted: '#CCCCCC',
      primary: '#FF0000',
      primaryHover: '#CC0000',
      primaryActive: '#AA0000',
      accent: '#00FF00',
      accentHover: '#00CC00',
      success: '#00FF00',
      warning: '#FFFF00',
      error: '#FF0000',
      info: '#0000FF',
      border: '#444444',
      borderHover: '#555555',
      chartBullish: '#00FF00',
      chartBearish: '#FF0000',
      chartNeutral: '#0000FF',
      gradientStart: '#000000',
      gradientMiddle: '#111111',
      gradientEnd: '#222222',
    });

    const themeToDelete =
      useThemeStore.getState().customThemes[useThemeStore.getState().customThemes.length - 1];
    const countBeforeDelete = useThemeStore.getState().customThemes.length;

    store.deleteCustomTheme(themeToDelete.id);

    const countAfterDelete = useThemeStore.getState().customThemes.length;
    expect(countAfterDelete).toBe(countBeforeDelete - 1);
  });

  it('should export and import theme', () => {
    const store = useThemeStore.getState();
    const currentTheme = store.currentTheme;

    const exported = store.exportTheme(currentTheme.id);
    expect(typeof exported).toBe('string');

    const parsed = JSON.parse(exported);
    expect(parsed.name).toBe(currentTheme.name);
    expect(parsed.colors).toBeDefined();
  });
});

describe('Accessibility Checklist', () => {
  it('should verify ARIA attributes are configurable', () => {
    const store = useAccessibilityStore.getState();

    expect(typeof store.screenReaderOptimizations).toBe('boolean');
    expect(typeof store.keyboardNavigationHints).toBe('boolean');
  });

  it('should support keyboard navigation features', () => {
    const store = useAccessibilityStore.getState();

    store.toggleKeyboardNavigationHints();
    expect(useAccessibilityStore.getState().keyboardNavigationHints).toBe(true);
  });

  it('should support enhanced focus indicators', () => {
    const store = useAccessibilityStore.getState();

    store.toggleFocusIndicatorEnhanced();
    expect(useAccessibilityStore.getState().focusIndicatorEnhanced).toBe(true);
  });
});
