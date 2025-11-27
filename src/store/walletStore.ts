import type {
   TokenBalance,
   TransactionFeeEstimate,
   SendTransactionInput,
   AddressBookContact,
   AddContactRequest,
   UpdateContactRequest,
   QRCodeData,
   SolanaPayQR,
   SwapHistoryEntry,
 } from '../types';
import { createBoundStore } from './createBoundStore';

export interface WalletAccount {
  publicKey: string;
  balance: number;
  network: string;
}

export interface SendWorkflow {
  step: 'input' | 'review' | 'confirm' | 'success' | 'error';
  input?: SendTransactionInput;
  feeEstimate?: TransactionFeeEstimate;
  txSignature?: string;
  error?: string;
}

interface WalletStoreState {
  accounts: WalletAccount[];
  activeAccount: WalletAccount | null;
  balances: Record<string, TokenBalance[]>;
  feeEstimates: Record<string, TransactionFeeEstimate>;
  addressBook: AddressBookContact[];
  swapHistory: SwapHistoryEntry[];
  sendWorkflow: SendWorkflow | null;
  isLoading: boolean;
  error: string | null;

  // Actions
  setAccounts: (accounts: WalletAccount[]) => void;
  setActiveAccount: (account: WalletAccount | null) => void;
  fetchBalances: (address: string, forceRefresh?: boolean) => Promise<void>;
  refreshActiveAccountBalances: () => Promise<void>;
  estimateFee: (recipient: string, amount: number, tokenMint?: string) => Promise<void>;
  sendTransaction: (input: SendTransactionInput, walletAddress: string) => Promise<string>;
  addContact: (request: AddContactRequest) => Promise<AddressBookContact>;
  updateContact: (request: UpdateContactRequest) => Promise<AddressBookContact>;
  deleteContact: (contactId: string) => Promise<void>;
  getContacts: () => Promise<void>;
  generateQR: (data: QRCodeData) => Promise<string>;
  generateSolanaPayQR: (
    recipient: string,
    amount?: number,
    splToken?: string,
    reference?: string,
    label?: string,
    message?: string,
    memo?: string
  ) => Promise<SolanaPayQR>;
  getSwapHistory: (walletAddress: string) => Promise<void>;
  startSendWorkflow: (input: SendTransactionInput) => void;
  updateSendWorkflow: (workflow: Partial<SendWorkflow>) => void;
  completeSendWorkflow: () => void;
  setError: (error: string | null) => void;
  reset: () => void;
}

const initialState = {
  accounts: [],
  activeAccount: null,
  balances: {},
  feeEstimates: {},
  addressBook: [],
  swapHistory: [],
  sendWorkflow: null,
  isLoading: false,
  error: null,
};

const storeResult = createBoundStore<WalletStoreState>((set, get) => ({
  ...initialState,

  setAccounts: accounts => {
    if (get().accounts === accounts) return;
    set({ accounts });
  },

  setActiveAccount: account => {
    if (get().activeAccount === account) return;
    set({ activeAccount: account });
  },

  fetchBalances: async (address: string, forceRefresh = false) => {
    set({ isLoading: true, error: null });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const balances = await invoke<TokenBalance[]>('wallet_get_token_balances', {
        address,
        forceRefresh,
      });
      set(state => ({
        balances: { ...state.balances, [address]: balances },
        isLoading: false,
      }));
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  refreshActiveAccountBalances: async () => {
    const activeAccount = get().activeAccount;
    if (activeAccount) {
      await get().fetchBalances(activeAccount.publicKey, true);
    }
  },

  estimateFee: async (recipient: string, amount: number, tokenMint?: string) => {
    set({ isLoading: true, error: null });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const estimate = await invoke<TransactionFeeEstimate>('wallet_estimate_fee', {
        recipient,
        amount,
        tokenMint,
      });
      const key = `${recipient}-${amount}-${tokenMint || 'SOL'}`;
      set(state => ({
        feeEstimates: { ...state.feeEstimates, [key]: estimate },
        isLoading: false,
      }));
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  sendTransaction: async (input: SendTransactionInput, walletAddress: string) => {
    set({ isLoading: true, error: null });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const signature = await invoke<string>('wallet_send_transaction', {
        input,
        walletAddress,
      });
      set({ isLoading: false });
      return signature;
    } catch (error) {
      set({ error: String(error), isLoading: false });
      throw error;
    }
  },

  addContact: async (request: AddContactRequest) => {
    set({ isLoading: true, error: null });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const contact = await invoke<AddressBookContact>('address_book_add_contact', {
        request,
      });
      set(state => ({
        addressBook: [...state.addressBook, contact],
        isLoading: false,
      }));
      return contact;
    } catch (error) {
      set({ error: String(error), isLoading: false });
      throw error;
    }
  },

  updateContact: async (request: UpdateContactRequest) => {
    set({ isLoading: true, error: null });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const updated = await invoke<AddressBookContact>('address_book_update_contact', {
        request,
      });
      set(state => ({
        addressBook: state.addressBook.map(c => (c.id === updated.id ? updated : c)),
        isLoading: false,
      }));
      return updated;
    } catch (error) {
      set({ error: String(error), isLoading: false });
      throw error;
    }
  },

  deleteContact: async (contactId: string) => {
    set({ isLoading: true, error: null });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('address_book_delete_contact', { contactId });
      set(state => ({
        addressBook: state.addressBook.filter(c => c.id !== contactId),
        isLoading: false,
      }));
    } catch (error) {
      set({ error: String(error), isLoading: false });
      throw error;
    }
  },

  getContacts: async () => {
    set({ isLoading: true, error: null });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const contacts = await invoke<AddressBookContact[]>('address_book_list_contacts');
      set({ addressBook: contacts, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  generateQR: async (data: QRCodeData) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<string>('wallet_generate_qr', { data });
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  generateSolanaPayQR: async (
    recipient: string,
    amount?: number,
    splToken?: string,
    reference?: string,
    label?: string,
    message?: string,
    memo?: string
  ) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<SolanaPayQR>('wallet_generate_solana_pay_qr', {
        recipient,
        amount,
        splToken,
        reference,
        label,
        message,
        memo,
      });
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  getSwapHistory: async (walletAddress: string) => {
    set({ isLoading: true, error: null });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const history = await invoke<SwapHistoryEntry[]>('wallet_get_swap_history', {
        walletAddress,
      });
      set({ swapHistory: history, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  startSendWorkflow: input => {
    set({
      sendWorkflow: {
        step: 'input',
        input,
      },
    });
  },

  updateSendWorkflow: workflow => {
    set(state => ({
      sendWorkflow: state.sendWorkflow ? { ...state.sendWorkflow, ...workflow } : null,
    }));
  },

  completeSendWorkflow: () => {
    set({ sendWorkflow: null });
  },

  setError: error => {
    if (get().error === error) return;
    set({ error });
  },

  reset: () => {
    set(initialState);
  },
}));

export const useWalletStore = storeResult.useStore;
export const walletStore = storeResult.store;

export const useWalletBalances = (address?: string) => {
  return useWalletStore(state => (address ? state.balances[address] : []));
};

export const useActiveAccount = () => {
  return useWalletStore(state => state.activeAccount);
};

export const useAddressBook = () => {
  return useWalletStore(state => state.addressBook);
};

export const useWalletStatus = () => {
  return useWalletStore(state => ({
    isLoading: state.isLoading,
    error: state.error,
  }));
};

export const useSendWorkflow = () => {
  return useWalletStore(state => state.sendWorkflow);
};

export const useSwapHistory = () => {
  return useWalletStore(state => state.swapHistory);
};
