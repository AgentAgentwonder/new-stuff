/**
 * @vitest-environment jsdom
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { useTauriCommand, useTauriArrayCommand } from '../src/hooks/useTauriCommand';
import { useStreamingCommand, useAIChatStream } from '../src/hooks/useStreamingCommand';
import { useStableCallback, useDebouncedCallback } from '../src/hooks/useStableCallback';
import { useUIStore } from '../src/store/uiStore';

// Mock UI store
vi.mock('../src/store/uiStore', () => ({
  useUIStore: vi.fn(),
}));

const mockUseUIStore = vi.mocked(useUIStore);

describe('useTauriCommand', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseUIStore.mockReturnValue({
      setLoading: vi.fn(),
      addToast: vi.fn(),
    } as any);
  });

  it('should initialize with default state', () => {
    const mockCommandFn = vi.fn();

    const { result } = renderHook(() => useTauriCommand(mockCommandFn));

    expect(result.current.data).toBeNull();
    expect(result.current.isLoading).toBe(false);
    expect(result.current.error).toBeNull();
    expect(mockCommandFn).not.toHaveBeenCalled();
  });

  it('should execute command successfully', async () => {
    const mockData = { test: 'data' };
    const mockCommandFn = vi.fn().mockResolvedValue({
      success: true,
      data: mockData,
    });

    const { result } = renderHook(() => useTauriCommand(mockCommandFn));

    await act(async () => {
      const response = await result.current.execute();
      expect(response.success).toBe(true);
      expect(response.data).toEqual(mockData);
    });

    await waitFor(() => {
      expect(result.current.data).toEqual(mockData);
      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toBeNull();
    });
  });

  it('should handle command error', async () => {
    const mockError = { message: 'Test error' };
    const mockCommandFn = vi.fn().mockResolvedValue({
      success: false,
      error: mockError,
    });

    const mockAddToast = vi.fn();
    mockUseUIStore.mockReturnValue({
      setLoading: vi.fn(),
      addToast: mockAddToast,
    } as any);

    const { result } = renderHook(() => useTauriCommand(mockCommandFn, { showToastOnError: true }));

    await act(async () => {
      await result.current.execute();
    });

    await waitFor(() => {
      expect(result.current.data).toBeNull();
      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toEqual(mockError);
      expect(mockAddToast).toHaveBeenCalledWith({
        type: 'error',
        title: 'Operation Failed',
        message: 'Test error',
      });
    });
  });

  it('should handle promise rejection', async () => {
    const errorMessage = 'Network error';
    const mockCommandFn = vi.fn().mockRejectedValue(new Error(errorMessage));

    const mockAddToast = vi.fn();
    mockUseUIStore.mockReturnValue({
      setLoading: vi.fn(),
      addToast: mockAddToast,
    } as any);

    const { result } = renderHook(() => useTauriCommand(mockCommandFn, { showToastOnError: true }));

    await act(async () => {
      await result.current.execute();
    });

    await waitFor(() => {
      expect(result.current.data).toBeNull();
      expect(result.current.isLoading).toBe(false);
      expect(result.current.error?.message).toBe(errorMessage);
      expect(mockAddToast).toHaveBeenCalledWith({
        type: 'error',
        title: 'Operation Failed',
        message: errorMessage,
      });
    });
  });

  it('should call onSuccess callback', async () => {
    const mockData = { test: 'data' };
    const mockCommandFn = vi.fn().mockResolvedValue({
      success: true,
      data: mockData,
    });
    const mockOnSuccess = vi.fn();

    const { result } = renderHook(() =>
      useTauriCommand(mockCommandFn, { onSuccess: mockOnSuccess })
    );

    await act(async () => {
      await result.current.execute();
    });

    await waitFor(() => {
      expect(mockOnSuccess).toHaveBeenCalledWith(mockData);
    });
  });

  it('should call onError callback', async () => {
    const mockError = { message: 'Test error' };
    const mockCommandFn = vi.fn().mockResolvedValue({
      success: false,
      error: mockError,
    });
    const mockOnError = vi.fn();

    const { result } = renderHook(() => useTauriCommand(mockCommandFn, { onError: mockOnError }));

    await act(async () => {
      await result.current.execute();
    });

    await waitFor(() => {
      expect(mockOnError).toHaveBeenCalledWith(mockError);
    });
  });

  it('should reset state', () => {
    const mockCommandFn = vi.fn();

    const { result } = renderHook(() => useTauriCommand(mockCommandFn));

    // Set some state
    act(() => {
      result.current.reset();
    });

    expect(result.current.data).toBeNull();
    expect(result.current.isLoading).toBe(false);
    expect(result.current.error).toBeNull();
  });

  it('should set loading state in UI store', async () => {
    const mockCommandFn = vi.fn().mockResolvedValue({
      success: true,
      data: 'test',
    });
    const mockSetLoading = vi.fn();
    mockUseUIStore.mockReturnValue({
      setLoading: mockSetLoading,
      addToast: vi.fn(),
    } as any);

    const { result } = renderHook(() =>
      useTauriCommand(mockCommandFn, {
        loadingId: 'test-loading',
        loadingMessage: 'Loading...',
      })
    );

    act(() => {
      result.current.execute();
    });

    expect(mockSetLoading).toHaveBeenCalledWith('test-loading', true, 'Loading...');
  });
});

describe('useTauriArrayCommand', () => {
  beforeEach(() => {
    mockUseUIStore.mockReturnValue({
      setLoading: vi.fn(),
      addToast: vi.fn(),
    } as any);
  });

  it('should provide array-specific utilities', async () => {
    const mockData = ['item1', 'item2', 'item3'];
    const mockCommandFn = vi.fn().mockResolvedValue({
      success: true,
      data: mockData,
    });

    const { result } = renderHook(() => useTauriArrayCommand(mockCommandFn));

    await act(async () => {
      await result.current.execute();
    });

    await waitFor(() => {
      expect(result.current.data).toEqual(mockData);
      expect(result.current.isEmpty).toBe(false);
      expect(result.current.count).toBe(3);
    });
  });

  it('should handle empty array', async () => {
    const mockCommandFn = vi.fn().mockResolvedValue({
      success: true,
      data: [],
    });

    const { result } = renderHook(() => useTauriArrayCommand(mockCommandFn));

    await act(async () => {
      await result.current.execute();
    });

    await waitFor(() => {
      expect(result.current.data).toEqual([]);
      expect(result.current.isEmpty).toBe(true);
      expect(result.current.count).toBe(0);
    });
  });
});

describe('useStreamingCommand', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseUIStore.mockReturnValue({
      setLoading: vi.fn(),
      addToast: vi.fn(),
    } as any);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('should initialize with default state', () => {
    const { result } = renderHook(() => useStreamingCommand());

    expect(result.current.isStreaming).toBe(false);
    expect(result.current.content).toBe('');
    expect(result.current.error).toBeNull();
  });

  it('should reset state', () => {
    const { result } = renderHook(() => useStreamingCommand());

    act(() => {
      result.current.reset();
    });

    expect(result.current.isStreaming).toBe(false);
    expect(result.current.content).toBe('');
    expect(result.current.error).toBeNull();
  });
});

describe('useAIChatStream', () => {
  beforeEach(() => {
    mockUseUIStore.mockReturnValue({
      setLoading: vi.fn(),
      addToast: vi.fn(),
    } as any);
  });

  it('should initialize with empty messages', () => {
    const { result } = renderHook(() => useAIChatStream());

    expect(result.current.messages).toEqual([]);
    expect(result.current.isTyping).toBe(false);
  });

  it('should clear chat', () => {
    const { result } = renderHook(() => useAIChatStream());

    act(() => {
      result.current.clearChat();
    });

    expect(result.current.messages).toEqual([]);
    expect(result.current.isTyping).toBe(false);
  });
});

describe('useStableCallback', () => {
  it('should maintain stable identity', () => {
    const mockFn = vi.fn();
    const { result, rerender } = renderHook(() => useStableCallback(mockFn));

    const firstCallback = result.current;

    rerender();

    expect(result.current).toBe(firstCallback);
  });

  it('should call latest function', () => {
    const mockFn1 = vi.fn(() => 'first');
    const mockFn2 = vi.fn(() => 'second');

    const { result, rerender } = renderHook(({ fn }) => useStableCallback(fn), {
      initialProps: { fn: mockFn1 },
    });

    act(() => {
      result.current();
    });

    expect(mockFn1).toHaveBeenCalled();
    expect(mockFn1).toHaveReturnedWith('first');

    // Update function
    rerender({ fn: mockFn2 });

    act(() => {
      result.current();
    });

    expect(mockFn2).toHaveBeenCalled();
    expect(mockFn2).toHaveReturnedWith('second');
  });
});

describe('useDebouncedCallback', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('should debounce function calls', () => {
    const mockFn = vi.fn();
    const { result } = renderHook(() => useDebouncedCallback(mockFn, 100));

    act(() => {
      result.current();
      result.current();
      result.current();
    });

    expect(mockFn).not.toHaveBeenCalled();

    act(() => {
      vi.advanceTimersByTime(100);
    });

    expect(mockFn).toHaveBeenCalledTimes(1);
  });

  it('should reset timer on subsequent calls', () => {
    const mockFn = vi.fn();
    const { result } = renderHook(() => useDebouncedCallback(mockFn, 100));

    act(() => {
      result.current();
    });

    act(() => {
      vi.advanceTimersByTime(50);
      result.current();
    });

    act(() => {
      vi.advanceTimersByTime(100);
    });

    expect(mockFn).toHaveBeenCalledTimes(1);
  });
});
