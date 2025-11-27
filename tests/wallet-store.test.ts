import { describe, it, expect, beforeEach } from 'vitest';
import { useWalletStore } from '../src/store/walletStore';
import { act, renderHook } from '@testing-library/react';
import { WalletAdapterNetwork } from '@solana/wallet-adapter-base';

// Mock Tauri invoke for store persistence
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('Wallet Store Setters', () => {
  beforeEach(() => {
    // Reset the store before each test
    const { result } = renderHook(() => useWalletStore());
    act(() => {
      result.current.reset();
    });
  });

  it('should not update state when setting the same status', () => {
    const { result } = renderHook(() => useWalletStore());

    // Set initial status
    act(() => {
      result.current.setStatus('connected');
    });
    expect(result.current.status).toBe('connected');

    // Try to set the same status again
    const prevState = result.current;
    act(() => {
      result.current.setStatus('connected');
    });

    // State should be the same object reference (no update)
    expect(result.current).toBe(prevState);
    expect(result.current.status).toBe('connected');
  });

  it('should update state when setting a different status', () => {
    const { result } = renderHook(() => useWalletStore());

    // Set initial status
    act(() => {
      result.current.setStatus('connected');
    });
    expect(result.current.status).toBe('connected');

    // Set different status
    act(() => {
      result.current.setStatus('disconnected');
    });

    expect(result.current.status).toBe('disconnected');
  });

  it('should not update state when setting the same publicKey', () => {
    const { result } = renderHook(() => useWalletStore());

    // Set initial public key
    act(() => {
      result.current.setPublicKey('test-key-123');
    });
    expect(result.current.publicKey).toBe('test-key-123');

    // Try to set the same public key again
    const prevState = result.current;
    act(() => {
      result.current.setPublicKey('test-key-123');
    });

    // State should be the same object reference (no update)
    expect(result.current).toBe(prevState);
    expect(result.current.publicKey).toBe('test-key-123');
  });

  it('should update state when setting a different publicKey', () => {
    const { result } = renderHook(() => useWalletStore());

    // Set initial public key
    act(() => {
      result.current.setPublicKey('test-key-123');
    });
    expect(result.current.publicKey).toBe('test-key-123');

    // Set different public key
    act(() => {
      result.current.setPublicKey('different-key-456');
    });

    expect(result.current.publicKey).toBe('different-key-456');
  });

  it('should not update state when setting the same balance', () => {
    const { result } = renderHook(() => useWalletStore());

    // Set initial balance
    act(() => {
      result.current.setBalance(1.5);
    });
    expect(result.current.balance).toBe(1.5);

    // Try to set the same balance again
    const prevState = result.current;
    act(() => {
      result.current.setBalance(1.5);
    });

    // State should be the same object reference (no update)
    expect(result.current).toBe(prevState);
    expect(result.current.balance).toBe(1.5);
  });

  it('should update state when setting a different balance', () => {
    const { result } = renderHook(() => useWalletStore());

    // Set initial balance
    act(() => {
      result.current.setBalance(1.5);
    });
    expect(result.current.balance).toBe(1.5);

    // Set different balance
    act(() => {
      result.current.setBalance(2.0);
    });

    expect(result.current.balance).toBe(2.0);
  });

  it('should not update state when setting the same error', () => {
    const { result } = renderHook(() => useWalletStore());

    // Set initial error
    act(() => {
      result.current.setError('test error');
    });
    expect(result.current.error).toBe('test error');

    // Try to set the same error again
    const prevState = result.current;
    act(() => {
      result.current.setError('test error');
    });

    // State should be the same object reference (no update)
    expect(result.current).toBe(prevState);
    expect(result.current.error).toBe('test error');
  });

  it('should update state when setting a different error', () => {
    const { result } = renderHook(() => useWalletStore());

    // Set initial error
    act(() => {
      result.current.setError('test error');
    });
    expect(result.current.error).toBe('test error');

    // Set different error
    act(() => {
      result.current.setError('different error');
    });

    expect(result.current.error).toBe('different error');
  });

  it('should not update state when setting the same session', () => {
    const { result } = renderHook(() => useWalletStore());
    const session = {
      publicKey: 'test-key',
      network: 'devnet',
      connected: true,
    };

    // Set initial session
    act(() => {
      result.current.setSession(session);
    });
    expect(result.current.session).toBe(session);

    // Try to set the same session again
    const prevState = result.current;
    act(() => {
      result.current.setSession(session);
    });

    // State should be the same object reference (no update)
    expect(result.current).toBe(prevState);
    expect(result.current.session).toBe(session);
  });

  it('should update state when setting a different session', () => {
    const { result } = renderHook(() => useWalletStore());
    const session1 = {
      publicKey: 'test-key',
      network: 'devnet',
      connected: true,
    };
    const session2 = {
      publicKey: 'different-key',
      network: 'mainnet',
      connected: true,
    };

    // Set initial session
    act(() => {
      result.current.setSession(session1);
    });
    expect(result.current.session).toBe(session1);

    // Set different session
    act(() => {
      result.current.setSession(session2);
    });

    expect(result.current.session).toBe(session2);
  });

  it('should not update state when setting the same lastConnected', () => {
    const { result } = renderHook(() => useWalletStore());
    const timestamp = '2024-01-01T00:00:00.000Z';

    // Set initial timestamp
    act(() => {
      result.current.setLastConnected(timestamp);
    });
    expect(result.current.lastConnected).toBe(timestamp);

    // Try to set the same timestamp again
    const prevState = result.current;
    act(() => {
      result.current.setLastConnected(timestamp);
    });

    // State should be the same object reference (no update)
    expect(result.current).toBe(prevState);
    expect(result.current.lastConnected).toBe(timestamp);
  });

  it('should update state when setting a different lastConnected', () => {
    const { result } = renderHook(() => useWalletStore());
    const timestamp1 = '2024-01-01T00:00:00.000Z';
    const timestamp2 = '2024-01-02T00:00:00.000Z';

    // Set initial timestamp
    act(() => {
      result.current.setLastConnected(timestamp1);
    });
    expect(result.current.lastConnected).toBe(timestamp1);

    // Set different timestamp
    act(() => {
      result.current.setLastConnected(timestamp2);
    });

    expect(result.current.lastConnected).toBe(timestamp2);
  });

  it('should not update state when setting the same attemptedAutoConnect', () => {
    const { result } = renderHook(() => useWalletStore());

    // Set initial value
    act(() => {
      result.current.setAttemptedAutoConnect(true);
    });
    expect(result.current.attemptedAutoConnect).toBe(true);

    // Try to set the same value again
    const prevState = result.current;
    act(() => {
      result.current.setAttemptedAutoConnect(true);
    });

    // State should be the same object reference (no update)
    expect(result.current).toBe(prevState);
    expect(result.current.attemptedAutoConnect).toBe(true);
  });

  it('should update state when setting a different attemptedAutoConnect', () => {
    const { result } = renderHook(() => useWalletStore());

    // Set initial value
    act(() => {
      result.current.setAttemptedAutoConnect(true);
    });
    expect(result.current.attemptedAutoConnect).toBe(true);

    // Set different value
    act(() => {
      result.current.setAttemptedAutoConnect(false);
    });

    expect(result.current.attemptedAutoConnect).toBe(false);
  });
});
