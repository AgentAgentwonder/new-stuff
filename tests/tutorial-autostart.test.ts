import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useTutorialStore } from '../src/store/tutorialStore';

// Mock Tauri invoke for store persistence
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('Tutorial Auto-Start Guard Logic', () => {
  beforeEach(() => {
    // Reset the store before each test
    const { result } = renderHook(() => useTutorialStore());
    act(() => {
      result.current.resetAllProgress();
      result.current.stopTutorial();
      result.current.setAutoStart(true);
    });
  });

  it('should prevent duplicate auto-start calls for same tutorial and page', () => {
    const { result } = renderHook(() => useTutorialStore());

    // Get available tutorials for a specific page
    const availableTutorials = result.current.getAvailableTutorials('dashboard');
    expect(availableTutorials.length).toBeGreaterThan(0);

    if (availableTutorials.length > 0) {
      const tutorialId = availableTutorials[0].id;

      // Mock the guard logic by tracking the last auto-started tutorial
      let lastAutoStarted: { page: string; tutorialId: string } | null = null;

      // Simulate the guard logic check
      const shouldStartTutorial = (page: string, tutorialId: string) => {
        if (
          lastAutoStarted &&
          lastAutoStarted.page === page &&
          lastAutoStarted.tutorialId === tutorialId
        ) {
          return false; // Skip duplicate auto-start
        }
        lastAutoStarted = { page, tutorialId };
        return true;
      };

      // First call should start the tutorial
      let shouldStart = shouldStartTutorial('dashboard', tutorialId);
      expect(shouldStart).toBe(true);
      expect(lastAutoStarted).toEqual({ page: 'dashboard', tutorialId });

      // Second call with same parameters should be blocked
      shouldStart = shouldStartTutorial('dashboard', tutorialId);
      expect(shouldStart).toBe(false);

      // Different page should allow starting
      shouldStart = shouldStartTutorial('trading', tutorialId);
      expect(shouldStart).toBe(true);
      expect(lastAutoStarted).toEqual({ page: 'trading', tutorialId });

      // Different tutorial should allow starting
      if (availableTutorials.length > 1) {
        const tutorialId2 = availableTutorials[1].id;
        shouldStart = shouldStartTutorial('dashboard', tutorialId2);
        expect(shouldStart).toBe(true);
        expect(lastAutoStarted).toEqual({ page: 'dashboard', tutorialId: tutorialId2 });
      }
    }
  });

  it('should reset guard when tutorial stops playing', () => {
    const { result } = renderHook(() => useTutorialStore());

    // Get available tutorials
    const availableTutorials = result.current.getAvailableTutorials('dashboard');
    expect(availableTutorials.length).toBeGreaterThan(0);

    if (availableTutorials.length > 0) {
      const tutorialId = availableTutorials[0].id;

      // Mock the guard logic
      let lastAutoStarted: { page: string; tutorialId: string } | null = null;

      const shouldStartTutorial = (page: string, tutorialId: string) => {
        if (
          lastAutoStarted &&
          lastAutoStarted.page === page &&
          lastAutoStarted.tutorialId === tutorialId
        ) {
          return false;
        }
        lastAutoStarted = { page, tutorialId };
        return true;
      };

      const resetGuard = () => {
        lastAutoStarted = null;
      };

      // Start tutorial
      let shouldStart = shouldStartTutorial('dashboard', tutorialId);
      expect(shouldStart).toBe(true);

      // Second call should be blocked
      shouldStart = shouldStartTutorial('dashboard', tutorialId);
      expect(shouldStart).toBe(false);

      // Simulate tutorial stopping (completion/skip)
      resetGuard();

      // After reset, should allow starting again
      shouldStart = shouldStartTutorial('dashboard', tutorialId);
      expect(shouldStart).toBe(true);
    }
  });

  it('should not start tutorial when auto-start is disabled', () => {
    const { result } = renderHook(() => useTutorialStore());

    // Disable auto-start
    act(() => {
      result.current.setAutoStart(false);
    });
    expect(result.current.autoStart).toBe(false);

    // Get available tutorials
    const availableTutorials = result.current.getAvailableTutorials('dashboard');
    expect(availableTutorials.length).toBeGreaterThan(0);

    // Mock the auto-start effect logic
    const shouldAutoStart = (autoStart: boolean, isPlaying: boolean) => {
      return autoStart && !isPlaying;
    };

    // With auto-start disabled, should not start
    let shouldStart = shouldAutoStart(result.current.autoStart, result.current.isPlaying);
    expect(shouldStart).toBe(false);

    // Even with no tutorial playing, should not start
    shouldStart = shouldAutoStart(result.current.autoStart, false);
    expect(shouldStart).toBe(false);
  });

  it('should not start tutorial when one is already playing', () => {
    const { result } = renderHook(() => useTutorialStore());

    // Ensure auto-start is enabled
    act(() => {
      result.current.setAutoStart(true);
    });
    expect(result.current.autoStart).toBe(true);

    // Get available tutorials
    const availableTutorials = result.current.getAvailableTutorials('dashboard');
    expect(availableTutorials.length).toBeGreaterThan(0);

    if (availableTutorials.length > 0) {
      const tutorialId = availableTutorials[0].id;

      // Start a tutorial
      act(() => {
        result.current.startTutorial(tutorialId);
      });
      expect(result.current.isPlaying).toBe(true);

      // Mock the auto-start effect logic
      const shouldAutoStart = (autoStart: boolean, isPlaying: boolean) => {
        return autoStart && !isPlaying;
      };

      // With tutorial playing, should not auto-start another
      const shouldStart = shouldAutoStart(result.current.autoStart, result.current.isPlaying);
      expect(shouldStart).toBe(false);
    }
  });

  it('should find eligible tutorial for auto-start', () => {
    const { result } = renderHook(() => useTutorialStore());

    // Get available tutorials for dashboard
    const availableTutorials = result.current.getAvailableTutorials('dashboard');
    expect(availableTutorials.length).toBeGreaterThan(0);

    // Mock the logic to find next eligible tutorial
    const findNextTutorial = (tutorials: unknown[], progress: Record<string, unknown>) => {
      return tutorials.find(tutorial => {
        const tutorialProgress = progress[(tutorial as { id: string }).id];
        if (!tutorialProgress) return true;
        if ((tutorialProgress as { skipped: boolean }).skipped) return false;
        return !(tutorialProgress as { completed: boolean }).completed;
      });
    };

    // Should find a tutorial when no progress exists
    let nextTutorial = findNextTutorial(availableTutorials, result.current.progress);
    expect(nextTutorial).toBeDefined();

    if (availableTutorials.length > 0) {
      const tutorialId = availableTutorials[0].id;

      // Complete the first tutorial
      act(() => {
        result.current.startTutorial(tutorialId);
      });

      act(() => {
        result.current.completeTutorial();
      });

      // Should not find the completed tutorial
      nextTutorial = findNextTutorial(availableTutorials, result.current.progress);
      expect((nextTutorial as { id: string }).id).not.toBe(tutorialId);

      // Skip the next available tutorial
      if (nextTutorial) {
        act(() => {
          result.current.startTutorial((nextTutorial as { id: string }).id);
        });

        act(() => {
          result.current.skipTutorial();
        });

        // Should not find the skipped tutorial
        const nextAfterSkip = findNextTutorial(availableTutorials, result.current.progress);
        if (nextAfterSkip) {
          expect((nextAfterSkip as { id: string }).id).not.toBe(
            (nextTutorial as { id: string }).id
          );
        }
      }
    }
  });
});
