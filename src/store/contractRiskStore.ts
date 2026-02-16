import type { ContractAssessment, RiskEvent } from '@/types';
import { createBoundStore } from './createBoundStore';

interface ContractRiskState {
  assessment: ContractAssessment | null;
  events: RiskEvent[];
  monitoredContracts: string[];
  isLoading: boolean;
  error: string | null;
  emergencyHaltActive: boolean;

  assessContract: (contractAddress: string) => Promise<ContractAssessment | null>;
  loadRiskEvents: (contractAddress: string) => Promise<void>;
  monitorContract: (contractAddress: string) => Promise<void>;
  unmonitorContract: (contractAddress: string) => Promise<void>;
  refreshMonitoredContracts: () => Promise<ContractAssessment[]>;
  loadMonitoredContracts: () => Promise<void>;
  fetchEmergencyHalt: () => Promise<void>;
  setEmergencyHalt: (enabled: boolean) => Promise<void>;
  reset: () => void;
}

const initialState = {
  assessment: null,
  events: [],
  monitoredContracts: [],
  isLoading: false,
  error: null,
  emergencyHaltActive: false,
};

const storeResult = createBoundStore<ContractRiskState>((set, get) => ({
  ...initialState,

  assessContract: async (contractAddress: string) => {
    if (!contractAddress) return null;
    set({ isLoading: true, error: null });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const assessment = await invoke<ContractAssessment>('assess_contract_risk', {
        contractAddress,
      });
      set({ assessment, isLoading: false });
      return assessment;
    } catch (error) {
      set({ error: String(error), isLoading: false });
      return null;
    }
  },

  loadRiskEvents: async (contractAddress: string) => {
    if (!contractAddress) return;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const events = await invoke<RiskEvent[]>('get_contract_risk_events', {
        contractAddress,
        limit: 20,
      });
      set({ events });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  monitorContract: async (contractAddress: string) => {
    if (!contractAddress) return;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('monitor_contract', { contractAddress });
      await get().loadMonitoredContracts();
    } catch (error) {
      set({ error: String(error) });
    }
  },

  unmonitorContract: async (contractAddress: string) => {
    if (!contractAddress) return;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('unmonitor_contract', { contractAddress });
      await get().loadMonitoredContracts();
    } catch (error) {
      set({ error: String(error) });
    }
  },

  refreshMonitoredContracts: async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const assessments = await invoke<ContractAssessment[]>('refresh_monitored_contracts');
      return assessments;
    } catch (error) {
      set({ error: String(error) });
      return [];
    }
  },

  loadMonitoredContracts: async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const monitoredContracts = await invoke<string[]>('list_monitored_contracts');
      set({ monitoredContracts });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  fetchEmergencyHalt: async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const emergencyHaltActive = await invoke<boolean>('get_emergency_halt');
      set({ emergencyHaltActive });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  setEmergencyHalt: async (enabled: boolean) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('set_emergency_halt', { enabled });
      set({ emergencyHaltActive: enabled });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  reset: () => {
    set({ ...initialState });
  },
}));

export const useContractRiskStore = storeResult.useStore;
export const contractRiskStore = storeResult.store;
