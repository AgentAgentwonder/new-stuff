import { create } from 'zustand';

export interface TestStore {
  test: boolean;
  setTest: (value: boolean) => void;
}

export const useTestStore = create<TestStore>(set => ({
  test: true,
  setTest: (value: boolean) => set({ test: value }),
}));
