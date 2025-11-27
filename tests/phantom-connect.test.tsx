import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { PhantomConnect } from '../src/components/wallet/PhantomConnect';
import { useWalletStore } from '../src/store/walletStore';
import { useWallet as useAdapterWallet, useConnection } from '@solana/wallet-adapter-react';
import { invoke } from '@tauri-apps/api/core';

// Mock the wallet adapter
vi.mock('@solana/wallet-adapter-react', () => ({
  useWallet: vi.fn(),
  useConnection: vi.fn(),
}));

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

// Mock the wallet store
vi.mock('../src/store/walletStore', () => ({
  useWalletStore: vi.fn(),
  PhantomSession: vi.fn(),
  WalletStatus: vi.fn(),
}));

const mockUseWallet = vi.mocked(useAdapterWallet);
const mockUseConnection = vi.mocked(useConnection);
const mockInvoke = vi.mocked(invoke);
const mockUseWalletStore = vi.mocked(useWalletStore);

describe('PhantomConnect', () => {
  const mockStoreState = {
    status: 'disconnected' as const,
    setStatus: vi.fn(),
    publicKey: null,
    setPublicKey: vi.fn(),
    balance: 0,
    setBalance: vi.fn(),
    error: null,
    setError: vi.fn(),
    autoReconnect: true,
    attemptedAutoConnect: false,
    setAttemptedAutoConnect: vi.fn(),
    lastConnected: null,
    setLastConnected: vi.fn(),
    setSession: vi.fn(),
    reset: vi.fn(),
    network: 'devnet' as const,
  };

  const mockAdapterState = {
    publicKey: null,
    connected: false,
    connecting: false,
    connect: vi.fn(),
    disconnect: vi.fn(),
    wallet: null,
    readyState: 'Installed' as const,
  };

  const mockConnection = {
    getBalance: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWalletStore.mockReturnValue(mockStoreState);
    mockUseWallet.mockReturnValue(mockAdapterState);
    mockUseConnection.mockReturnValue(mockConnection);

    // Mock console methods to avoid noise in tests
    vi.spyOn(console, 'warn').mockImplementation(() => {});
    vi.spyOn(console, 'log').mockImplementation(() => {});
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('should render connect button when disconnected', () => {
    render(<PhantomConnect />);

    expect(screen.getByText('Connect Wallet')).toBeInTheDocument();
  });

  it('should not cause infinite re-renders when mounting', async () => {
    const setStatus = vi.fn();
    mockUseWalletStore.mockReturnValue({
      ...mockStoreState,
      setStatus,
    });

    // Track how many times setStatus is called
    let statusCallCount = 0;
    setStatus.mockImplementation(() => {
      statusCallCount++;
    });

    render(<PhantomConnect />);

    // Wait for any potential useEffect runs
    await waitFor(
      () => {
        // setStatus should only be called once for initial status
        expect(statusCallCount).toBeLessThanOrEqual(2);
      },
      { timeout: 1000 }
    );
  });

  it('should only call setPublicKey when the key actually changes', async () => {
    const setPublicKey = vi.fn();
    const mockPublicKey = { toBase58: () => 'test-public-key' };

    mockUseWalletStore.mockReturnValue({
      ...mockStoreState,
      publicKey: 'test-public-key', // Same as adapter
      setPublicKey,
    });

    mockUseWallet.mockReturnValue({
      ...mockAdapterState,
      publicKey: mockPublicKey,
      connected: true,
    });

    mockInvoke.mockResolvedValue({
      publicKey: 'test-public-key',
      network: 'devnet',
      connected: true,
    });

    render(<PhantomConnect />);

    await waitFor(() => {
      // setPublicKey should not be called since the key is the same
      expect(setPublicKey).not.toHaveBeenCalled();
    });
  });

  it('should call setPublicKey when the key changes', async () => {
    const setPublicKey = vi.fn();
    const mockPublicKey = { toBase58: () => 'new-public-key' };

    mockUseWalletStore.mockReturnValue({
      ...mockStoreState,
      publicKey: 'old-public-key', // Different from adapter
      setPublicKey,
    });

    mockUseWallet.mockReturnValue({
      ...mockAdapterState,
      publicKey: mockPublicKey,
      connected: true,
    });

    mockInvoke.mockResolvedValue({
      publicKey: 'new-public-key',
      network: 'devnet',
      connected: true,
    });

    render(<PhantomConnect />);

    await waitFor(() => {
      // setPublicKey should be called since the key is different
      expect(setPublicKey).toHaveBeenCalledWith('new-public-key');
    });
  });

  it('should only update balance when it changes', async () => {
    const setBalance = vi.fn();
    const mockPublicKey = { toBase58: () => 'test-public-key' };

    mockUseWalletStore.mockReturnValue({
      ...mockStoreState,
      publicKey: 'test-public-key',
      balance: 1.5, // Same as will be returned
      setBalance,
    });

    mockUseWallet.mockReturnValue({
      ...mockAdapterState,
      publicKey: mockPublicKey,
      connected: true,
    });

    mockInvoke.mockResolvedValue(1.5); // Same balance

    render(<PhantomConnect />);

    await waitFor(() => {
      // setBalance should not be called since the balance is the same
      expect(setBalance).not.toHaveBeenCalled();
    });
  });

  it('should update balance when it changes', async () => {
    const setBalance = vi.fn();
    const mockPublicKey = { toBase58: () => 'test-public-key' };

    mockUseWalletStore.mockReturnValue({
      ...mockStoreState,
      publicKey: 'test-public-key',
      balance: 1.0, // Different from what will be returned
      setBalance,
    });

    mockUseWallet.mockReturnValue({
      ...mockAdapterState,
      publicKey: mockPublicKey,
      connected: true,
    });

    mockInvoke.mockResolvedValue(2.5); // Different balance

    render(<PhantomConnect />);

    await waitFor(() => {
      // setBalance should be called since the balance is different
      expect(setBalance).toHaveBeenCalledWith(2.5);
    });
  });

  it('should not update session when session data is identical', async () => {
    const setSession = vi.fn();
    const mockPublicKey = { toBase58: () => 'test-public-key' };
    const existingSession = {
      publicKey: 'test-public-key',
      network: 'devnet',
      connected: true,
    };

    mockUseWalletStore.mockReturnValue({
      ...mockStoreState,
      publicKey: 'test-public-key',
      session: existingSession, // Same as will be returned
      setSession,
    });

    mockUseWallet.mockReturnValue({
      ...mockAdapterState,
      publicKey: mockPublicKey,
      connected: true,
    });

    mockInvoke.mockResolvedValue(existingSession); // Same session

    render(<PhantomConnect />);

    await waitFor(() => {
      // setSession should not be called since the session is the same
      expect(setSession).not.toHaveBeenCalled();
    });
  });

  it('should display connected state when wallet is connected', () => {
    const mockPublicKey = { toBase58: () => 'test-very-long-public-key-address' };

    mockUseWalletStore.mockReturnValue({
      ...mockStoreState,
      publicKey: 'test-very-long-public-key-address',
      balance: 2.5,
      status: 'connected' as const,
    });

    mockUseWallet.mockReturnValue({
      ...mockAdapterState,
      publicKey: mockPublicKey,
      connected: true,
    });

    render(<PhantomConnect />);

    expect(screen.getByText('test...ress')).toBeInTheDocument(); // Truncated key
    expect(screen.getByText('2.5000 SOL')).toBeInTheDocument();
    expect(screen.getByText('Disconnect')).toBeInTheDocument();
  });

  it('should handle connection errors gracefully', async () => {
    const setError = vi.fn();
    const setStatus = vi.fn();

    mockUseWalletStore.mockReturnValue({
      ...mockStoreState,
      setError,
      setStatus,
    });

    mockUseWallet.mockReturnValue({
      ...mockAdapterState,
      wallet: { adapter: { name: 'Phantom' } },
    });

    const connect = vi.fn().mockRejectedValue(new Error('User rejected'));
    mockUseWallet.mockReturnValue({
      ...mockAdapterState,
      wallet: { adapter: { name: 'Phantom' } },
      connect,
    });

    render(<PhantomConnect />);

    const connectButton = screen.getByText('Connect Wallet');
    connectButton.click();

    await waitFor(() => {
      expect(setError).toHaveBeenCalledWith('Connection rejected');
      expect(setStatus).toHaveBeenCalledWith('error');
    });
  });
});
