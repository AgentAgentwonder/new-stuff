import { describe, it, expect, beforeEach, vi } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { usePriceStream } from '../src/hooks/usePriceStream';
import { useWalletStream } from '../src/hooks/useWalletStream';
import { StreamProvider } from '../src/contexts/StreamContext';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

vi.mock('../src/hooks/useWebSocketStream', () => ({
  useStreamStatus: vi.fn(() => []),
}));

const createWrapper =
  () =>
  ({ children }: { children: React.ReactNode }) => <StreamProvider>{children}</StreamProvider>;

const getInvokeCallCount = (method: string) =>
  (invoke as any).mock.calls.filter((call: any) => call[0] === method).length;

describe('usePriceStream - Stabilization', () => {
  beforeAll(() => {
    if (typeof window !== 'undefined') {
      if (!window.requestAnimationFrame) {
        (window as any).requestAnimationFrame = (cb: FrameRequestCallback) =>
          window.setTimeout(() => cb(Date.now()), 16);
      }
      if (!window.cancelAnimationFrame) {
        (window as any).cancelAnimationFrame = (id: number) => window.clearTimeout(id);
      }
    }
  });

  beforeEach(() => {
    vi.clearAllMocks();
    (invoke as any).mockResolvedValue([]);
    (listen as any).mockResolvedValue(() => {});
    if (typeof window !== 'undefined') {
      window.sessionStorage.clear();
    }
  });

  it('should not resubscribe when symbols array reference changes but content is same', async () => {
    const { rerender } = renderHook(({ symbols }) => usePriceStream(symbols), {
      wrapper: createWrapper(),
      initialProps: { symbols: ['BTC', 'ETH'] },
    });

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('subscribe_price_stream', {
        symbols: expect.arrayContaining(['BTC', 'ETH']),
      });
    });

    const initialCallCount = getInvokeCallCount('subscribe_price_stream');

    // Render with new array instance, same content
    rerender({ symbols: ['BTC', 'ETH'] });

    await waitFor(() => {
      expect(getInvokeCallCount('subscribe_price_stream')).toBe(initialCallCount);
    });
  });

  it('should normalize and deduplicate symbols', async () => {
    renderHook(() => usePriceStream(['BTC', 'ETH', 'BTC', 'SOL']), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('subscribe_price_stream', {
        symbols: expect.any(Array),
      });
    });

    const subscribeCall = (invoke as any).mock.calls.find(
      (call: any) => call[0] === 'subscribe_price_stream'
    );
    const subscribedSymbols = subscribeCall[1].symbols;

    expect(subscribedSymbols).toHaveLength(3);
    expect(new Set(subscribedSymbols).size).toBe(3);
    expect(subscribedSymbols).toContain('BTC');
    expect(subscribedSymbols).toContain('ETH');
    expect(subscribedSymbols).toContain('SOL');
  });

  it('should handle unordered symbols as equivalent', async () => {
    const { rerender } = renderHook(({ symbols }) => usePriceStream(symbols), {
      wrapper: createWrapper(),
      initialProps: { symbols: ['BTC', 'ETH', 'SOL'] },
    });

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('subscribe_price_stream', {
        symbols: expect.any(Array),
      });
    });

    const initialCallCount = getInvokeCallCount('subscribe_price_stream');

    // Rerender with same symbols in different order
    rerender({ symbols: ['SOL', 'BTC', 'ETH'] });

    await waitFor(() => {
      expect(getInvokeCallCount('subscribe_price_stream')).toBe(initialCallCount);
    });
  });

  it('should not trigger state updates when loading/error unchanged', async () => {
    (invoke as any).mockResolvedValue(undefined);

    const { result } = renderHook(() => usePriceStream(['BTC']), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    const initialState = result.current;

    // Wait a bit to see if state changes
    await waitFor(() => {}, { timeout: 100 });

    expect(result.current).toBe(initialState);
  });

  it('should resubscribe when symbols actually change', async () => {
    const { rerender } = renderHook(({ symbols }) => usePriceStream(symbols), {
      wrapper: createWrapper(),
      initialProps: { symbols: ['BTC'] },
    });

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('subscribe_price_stream', {
        symbols: ['BTC'],
      });
    });

    // Change to different symbols
    rerender({ symbols: ['ETH', 'SOL'] });

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('unsubscribe_price_stream', {
        symbols: ['BTC'],
      });
    });

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('subscribe_price_stream', {
        symbols: expect.arrayContaining(['ETH', 'SOL']),
      });
    });
  });
});

describe('useWalletStream - Stabilization', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (invoke as any).mockResolvedValue([]);
    (listen as any).mockResolvedValue(() => {});
  });

  it('should not resubscribe when addresses array reference changes but content is same', async () => {
    const { rerender } = renderHook(({ addresses }) => useWalletStream(addresses), {
      wrapper: createWrapper(),
      initialProps: { addresses: ['addr1', 'addr2'] },
    });

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('subscribe_wallet_stream', {
        addresses: expect.arrayContaining(['addr1', 'addr2']),
      });
    });

    const initialCallCount = getInvokeCallCount('subscribe_wallet_stream');

    // Render with new array instance, same content
    rerender({ addresses: ['addr1', 'addr2'] });

    await waitFor(() => {
      expect(getInvokeCallCount('subscribe_wallet_stream')).toBe(initialCallCount);
    });
  });

  it('should normalize and deduplicate addresses', async () => {
    renderHook(() => useWalletStream(['addr1', 'addr2', 'addr1', 'addr3']), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('subscribe_wallet_stream', {
        addresses: expect.any(Array),
      });
    });

    const subscribeCall = (invoke as any).mock.calls.find(
      (call: any) => call[0] === 'subscribe_wallet_stream'
    );
    const subscribedAddresses = subscribeCall[1].addresses;

    expect(subscribedAddresses).toHaveLength(3);
    expect(new Set(subscribedAddresses).size).toBe(3);
  });

  it('should handle unordered addresses as equivalent', async () => {
    const { rerender } = renderHook(({ addresses }) => useWalletStream(addresses), {
      wrapper: createWrapper(),
      initialProps: { addresses: ['addr1', 'addr2', 'addr3'] },
    });

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('subscribe_wallet_stream', {
        addresses: expect.any(Array),
      });
    });

    const initialCallCount = getInvokeCallCount('subscribe_wallet_stream');

    // Rerender with same addresses in different order
    rerender({ addresses: ['addr3', 'addr1', 'addr2'] });

    await waitFor(() => {
      expect(getInvokeCallCount('subscribe_wallet_stream')).toBe(initialCallCount);
    });
  });

  it('should not trigger state updates when loading/error unchanged', async () => {
    (invoke as any).mockResolvedValue(undefined);

    const { result } = renderHook(() => useWalletStream(['addr1']), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    const initialState = result.current;

    // Wait a bit to see if state changes
    await waitFor(() => {}, { timeout: 100 });

    expect(result.current).toBe(initialState);
  });

  it('should resubscribe when addresses actually change', async () => {
    const { rerender } = renderHook(({ addresses }) => useWalletStream(addresses), {
      wrapper: createWrapper(),
      initialProps: { addresses: ['addr1'] },
    });

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('subscribe_wallet_stream', {
        addresses: ['addr1'],
      });
    });

    // Change to different addresses
    rerender({ addresses: ['addr2', 'addr3'] });

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('unsubscribe_wallet_stream', {
        addresses: ['addr1'],
      });
    });

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('subscribe_wallet_stream', {
        addresses: expect.arrayContaining(['addr2', 'addr3']),
      });
    });
  });

  it('should deduplicate consecutive identical transactions', async () => {
    let eventHandler: any;
    (listen as any).mockImplementation((eventName: string, handler: any) => {
      eventHandler = handler;
      return Promise.resolve(() => {});
    });

    const { result } = renderHook(() => useWalletStream(['addr1']), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    const tx = {
      signature: 'sig1',
      slot: 12345,
      timestamp: 1000,
      typ: 'transfer',
      amount: 100,
      symbol: 'SOL',
      from: 'addr1',
      to: 'addr2',
    };

    // Send the same transaction twice
    eventHandler({ payload: tx });
    eventHandler({ payload: tx });

    await waitFor(() => {
      expect(result.current.transactions).toHaveLength(1);
    });
  });
});
