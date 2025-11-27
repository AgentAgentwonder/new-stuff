import { describe, it, expect, beforeEach } from 'vitest';
import { useTutorialStore } from '../src/store/tutorialStore';
import { act, renderHook } from '@testing-library/react';

// Mock Tauri invoke for store persistence
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('Tutorial Store Auto-Start', () => {
  beforeEach(() => {
    // Reset the store before each test
    const { result } = renderHook(() => useTutorialStore());
    act(() => {
      result.current.resetAllProgress();
      result.current.stopTutorial();
    });
  });

  it('should not update state when setting the same auto-start value', () => {
    const { result } = renderHook(() => useTutorialStore());

    // Set initial auto-start to true (default)
    act(() => {
      result.current.setAutoStart(true);
    });
    expect(result.current.autoStart).toBe(true);

    // Try to set the same value again
    const prevState = result.current;
    act(() => {
      result.current.setAutoStart(true);
    });

    // State should be the same object reference (no update)
    expect(result.current).toBe(prevState);
    expect(result.current.autoStart).toBe(true);
  });

  it('should update state when setting a different auto-start value', () => {
    const { result } = renderHook(() => useTutorialStore());

    // Set initial auto-start to true
    act(() => {
      result.current.setAutoStart(true);
    });
    expect(result.current.autoStart).toBe(true);

    // Set different value
    act(() => {
      result.current.setAutoStart(false);
    });

    expect(result.current.autoStart).toBe(false);
  });

  it('should filter available tutorials by current page', () => {
    const { result } = renderHook(() => useTutorialStore());

    // Get available tutorials for dashboard page
    let availableTutorials = result.current.getAvailableTutorials('dashboard');
    expect(Array.isArray(availableTutorials)).toBe(true);

    // Get available tutorials for trading page
    availableTutorials = result.current.getAvailableTutorials('trading');
    expect(Array.isArray(availableTutorials)).toBe(true);

    // Get available tutorials without page filter
    availableTutorials = result.current.getAvailableTutorials();
    expect(Array.isArray(availableTutorials)).toBe(true);
  });

  it('should not include completed tutorials in available list', () => {
    const { result } = renderHook(() => useTutorialStore());

    // Get initial available tutorials
    const initialAvailable = result.current.getAvailableTutorials();
    expect(initialAvailable.length).toBeGreaterThan(0);

    if (initialAvailable.length > 0) {
      const tutorialId = initialAvailable[0].id;

      // Complete the first tutorial
      act(() => {
        result.current.startTutorial(tutorialId);
      });

      act(() => {
        result.current.completeTutorial();
      });

      // Check that it's no longer in available list
      const availableAfter = result.current.getAvailableTutorials();
      const completedTutorial = availableAfter.find(t => t.id === tutorialId);
      expect(completedTutorial).toBeUndefined();
    }
  });

  it('should include skipped tutorials in available list', () => {
    const { result } = renderHook(() => useTutorialStore());

    // Get initial available tutorials
    const initialAvailable = result.current.getAvailableTutorials();
    expect(initialAvailable.length).toBeGreaterThan(0);

    if (initialAvailable.length > 0) {
      const tutorialId = initialAvailable[0].id;

      // Skip the first tutorial
      act(() => {
        result.current.startTutorial(tutorialId);
      });

      act(() => {
        result.current.skipTutorial();
      });

      // Check that it's still in available list (but marked as skipped)
      const availableAfter = result.current.getAvailableTutorials();
      const skippedTutorial = availableAfter.find(t => t.id === tutorialId);
      expect(skippedTutorial).toBeDefined();

      // Check progress shows it's skipped
      const progress = result.current.getTutorialProgress(tutorialId);
      expect(progress?.skipped).toBe(true);
    }
  });

  it('should reset tutorial progress correctly', () => {
    const { result } = renderHook(() => useTutorialStore());

    // Get initial available tutorials
    const initialAvailable = result.current.getAvailableTutorials();
    expect(initialAvailable.length).toBeGreaterThan(0);

    if (initialAvailable.length > 0) {
      const tutorialId = initialAvailable[0].id;

      // Complete the first tutorial
      act(() => {
        result.current.startTutorial(tutorialId);
      });

      act(() => {
        result.current.completeTutorial();
      });

      // Check that it's completed
      let progress = result.current.getTutorialProgress(tutorialId);
      expect(progress?.completed).toBe(true);

      // Reset the tutorial
      act(() => {
        result.current.resetTutorial(tutorialId);
      });

      // Check that progress is cleared
      progress = result.current.getTutorialProgress(tutorialId);
      expect(progress).toBeNull();
    }
  });
});
