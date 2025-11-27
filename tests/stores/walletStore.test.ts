import { describe, it, expect, beforeEach, vi } from 'vitest';
import { act, renderHook } from '@testing-library/react';
import { useWalletStore, walletStore } from '../../src/store/walletStore';
import type { TokenBalance, AddressBookContact } from '../../src/types';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

const { invoke } = await import('@tauri-apps/api/core');

describe('Wallet Store', () => {
  beforeEach(() => {
    act(() => {
      walletStore.getState().reset();
    });
    vi.clearAllMocks();
  });

  describe('setAccounts', () => {
    it('should not update state when setting the same accounts', () => {
      const { result } = renderHook(() => useWalletStore());
      const accounts = [
        { publicKey: 'key1', balance: 1.5, network: 'mainnet' },
        { publicKey: 'key2', balance: 2.0, network: 'devnet' },
      ];

      act(() => {
        result.current.setAccounts(accounts);
      });
      expect(result.current.accounts).toBe(accounts);

      const prevState = result.current;
      act(() => {
        result.current.setAccounts(accounts);
      });

      expect(result.current).toBe(prevState);
    });

    it('should update state when setting different accounts', () => {
      const { result } = renderHook(() => useWalletStore());
      const accounts1 = [{ publicKey: 'key1', balance: 1.5, network: 'mainnet' }];
      const accounts2 = [{ publicKey: 'key2', balance: 2.0, network: 'devnet' }];

      act(() => {
        result.current.setAccounts(accounts1);
      });
      expect(result.current.accounts).toEqual(accounts1);

      act(() => {
        result.current.setAccounts(accounts2);
      });
      expect(result.current.accounts).toEqual(accounts2);
    });
  });

  describe('setActiveAccount', () => {
    it('should not update state when setting the same active account', () => {
      const { result } = renderHook(() => useWalletStore());
      const account = { publicKey: 'key1', balance: 1.5, network: 'mainnet' };

      act(() => {
        result.current.setActiveAccount(account);
      });
      expect(result.current.activeAccount).toBe(account);

      const prevState = result.current;
      act(() => {
        result.current.setActiveAccount(account);
      });

      expect(result.current).toBe(prevState);
    });

    it('should update state when setting different active account', () => {
      const { result } = renderHook(() => useWalletStore());
      const account1 = { publicKey: 'key1', balance: 1.5, network: 'mainnet' };
      const account2 = { publicKey: 'key2', balance: 2.0, network: 'devnet' };

      act(() => {
        result.current.setActiveAccount(account1);
      });
      expect(result.current.activeAccount).toEqual(account1);

      act(() => {
        result.current.setActiveAccount(account2);
      });
      expect(result.current.activeAccount).toEqual(account2);
    });
  });

  describe('fetchBalances', () => {
    it('should fetch and store token balances', async () => {
      const { result } = renderHook(() => useWalletStore());
      const address = 'test-address';
      const mockBalances: TokenBalance[] = [
        {
          mint: 'SOL',
          symbol: 'SOL',
          name: 'Solana',
          balance: 1.5,
          decimals: 9,
          usdValue: 150,
          change24h: 2.5,
          lastUpdated: new Date().toISOString(),
        },
      ];

      vi.mocked(invoke).mockResolvedValueOnce(mockBalances);

      await act(async () => {
        await result.current.fetchBalances(address);
      });

      expect(invoke).toHaveBeenCalledWith('wallet_get_token_balances', {
        address,
        forceRefresh: false,
      });
      expect(result.current.balances[address]).toEqual(mockBalances);
      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toBeNull();
    });

    it('should handle fetch balances error', async () => {
      const { result } = renderHook(() => useWalletStore());
      const address = 'test-address';
      const errorMessage = 'Failed to fetch balances';

      vi.mocked(invoke).mockRejectedValueOnce(new Error(errorMessage));

      await act(async () => {
        await result.current.fetchBalances(address);
      });

      expect(result.current.error).toContain(errorMessage);
      expect(result.current.isLoading).toBe(false);
    });

    it('should force refresh when requested', async () => {
      const { result } = renderHook(() => useWalletStore());
      const address = 'test-address';
      const mockBalances: TokenBalance[] = [];

      vi.mocked(invoke).mockResolvedValueOnce(mockBalances);

      await act(async () => {
        await result.current.fetchBalances(address, true);
      });

      expect(invoke).toHaveBeenCalledWith('wallet_get_token_balances', {
        address,
        forceRefresh: true,
      });
    });
  });

  describe('estimateFee', () => {
    it('should estimate transaction fee', async () => {
      const { result } = renderHook(() => useWalletStore());
      const recipient = 'recipient-address';
      const amount = 1.5;
      const mockEstimate = {
        baseFee: 0.000005,
        priorityFee: 0.000001,
        totalFee: 0.000006,
        estimatedUnits: 200000,
      };

      vi.mocked(invoke).mockResolvedValueOnce(mockEstimate);

      await act(async () => {
        await result.current.estimateFee(recipient, amount);
      });

      expect(invoke).toHaveBeenCalledWith('wallet_estimate_fee', {
        recipient,
        amount,
        tokenMint: undefined,
      });
      expect(result.current.isLoading).toBe(false);
    });
  });

  describe('sendTransaction', () => {
    it('should send transaction and return signature', async () => {
      const { result } = renderHook(() => useWalletStore());
      const input = {
        recipient: 'recipient-address',
        amount: 1.5,
      };
      const walletAddress = 'wallet-address';
      const mockSignature = 'tx-signature-123';

      vi.mocked(invoke).mockResolvedValueOnce(mockSignature);

      let signature: string | undefined;
      await act(async () => {
        signature = await result.current.sendTransaction(input, walletAddress);
      });

      expect(invoke).toHaveBeenCalledWith('wallet_send_transaction', {
        input,
        walletAddress,
      });
      expect(signature).toBe(mockSignature);
      expect(result.current.isLoading).toBe(false);
    });

    it('should handle send transaction error', async () => {
      const { result } = renderHook(() => useWalletStore());
      const input = {
        recipient: 'recipient-address',
        amount: 1.5,
      };
      const walletAddress = 'wallet-address';
      const errorMessage = 'Insufficient funds';

      vi.mocked(invoke).mockRejectedValueOnce(new Error(errorMessage));

      await act(async () => {
        try {
          await result.current.sendTransaction(input, walletAddress);
        } catch (error) {
          expect(String(error)).toContain(errorMessage);
        }
      });

      expect(result.current.error).toContain(errorMessage);
      expect(result.current.isLoading).toBe(false);
    });
  });

  describe('addContact', () => {
    it('should add contact to address book', async () => {
      const { result } = renderHook(() => useWalletStore());
      const request = {
        address: 'contact-address',
        label: 'Test Contact',
        tags: ['friend'],
      };
      const mockContact: AddressBookContact = {
        id: 'contact-1',
        address: request.address,
        label: request.label,
        tags: request.tags,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        transactionCount: 0,
      };

      vi.mocked(invoke).mockResolvedValueOnce(mockContact);

      await act(async () => {
        await result.current.addContact(request);
      });

      expect(invoke).toHaveBeenCalledWith('address_book_add_contact', { request });
      expect(result.current.addressBook).toContainEqual(mockContact);
      expect(result.current.isLoading).toBe(false);
    });
  });

  describe('sendWorkflow', () => {
    it('should start send workflow', () => {
      const { result } = renderHook(() => useWalletStore());
      const input = {
        recipient: 'recipient-address',
        amount: 1.5,
      };

      act(() => {
        result.current.startSendWorkflow(input);
      });

      expect(result.current.sendWorkflow).toEqual({
        step: 'input',
        input,
      });
    });

    it('should update send workflow', () => {
      const { result } = renderHook(() => useWalletStore());
      const input = {
        recipient: 'recipient-address',
        amount: 1.5,
      };

      act(() => {
        result.current.startSendWorkflow(input);
      });

      act(() => {
        result.current.updateSendWorkflow({ step: 'review' });
      });

      expect(result.current.sendWorkflow?.step).toBe('review');
      expect(result.current.sendWorkflow?.input).toEqual(input);
    });

    it('should complete send workflow', () => {
      const { result } = renderHook(() => useWalletStore());
      const input = {
        recipient: 'recipient-address',
        amount: 1.5,
      };

      act(() => {
        result.current.startSendWorkflow(input);
      });

      act(() => {
        result.current.completeSendWorkflow();
      });

      expect(result.current.sendWorkflow).toBeNull();
    });
  });

  describe('setError', () => {
    it('should not update state when setting the same error', () => {
      const { result } = renderHook(() => useWalletStore());
      const error = 'test error';

      act(() => {
        result.current.setError(error);
      });
      expect(result.current.error).toBe(error);

      const prevState = result.current;
      act(() => {
        result.current.setError(error);
      });

      expect(result.current).toBe(prevState);
    });

    it('should update state when setting different error', () => {
      const { result } = renderHook(() => useWalletStore());
      const error1 = 'error 1';
      const error2 = 'error 2';

      act(() => {
        result.current.setError(error1);
      });
      expect(result.current.error).toBe(error1);

      act(() => {
        result.current.setError(error2);
      });
      expect(result.current.error).toBe(error2);
    });
  });

  describe('reset', () => {
    it('should reset store to initial state', () => {
      const { result } = renderHook(() => useWalletStore());

      act(() => {
        result.current.setAccounts([{ publicKey: 'key1', balance: 1.5, network: 'mainnet' }]);
        result.current.setError('test error');
      });

      act(() => {
        result.current.reset();
      });

      expect(result.current.accounts).toEqual([]);
      expect(result.current.error).toBeNull();
      expect(result.current.activeAccount).toBeNull();
      expect(result.current.balances).toEqual({});
    });
  });
});
