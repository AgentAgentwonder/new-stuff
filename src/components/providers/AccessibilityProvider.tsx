import type { ReactNode } from 'react';
import { useEffect } from 'react';
import { useAccessibilityStore } from '@/store/accessibilityStore';

interface AccessibilityProviderProps {
  children: ReactNode;
}

export function AccessibilityProvider({ children }: AccessibilityProviderProps) {
  const fontScale = useAccessibilityStore(state => state.fontScale);
  const highContrastMode = useAccessibilityStore(state => state.highContrastMode);
  const reducedMotion = useAccessibilityStore(state => state.reducedMotion);

  useEffect(() => {
    try {
      const root = document.documentElement;
      root.style.setProperty('--font-scale', fontScale.toString());
      root.style.fontSize = `${16 * fontScale}px`;
    } catch (error) {
      console.error('Failed to set font scale', error);
    }
  }, [fontScale]);

  useEffect(() => {
    try {
      document.documentElement.classList.toggle('high-contrast', highContrastMode);
    } catch (error) {
      console.error('Failed to toggle high contrast', error);
    }
  }, [highContrastMode]);

  useEffect(() => {
    try {
      document.documentElement.classList.toggle('reduced-motion', reducedMotion);
    } catch (error) {
      console.error('Failed to toggle reduced motion', error);
    }
  }, [reducedMotion]);

  return <>{children}</>;
}
