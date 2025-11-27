import { create } from 'zustand';

export const useSimpleTestStore = create(() => ({
  test: 'hello',
}));
