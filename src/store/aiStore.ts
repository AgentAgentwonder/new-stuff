import type {
   ChatMessage,
   ChatResponse,
   PatternWarning,
   PortfolioOptimization,
   StreamingMetadata,
 } from '../types';
import { createBoundStore } from './createBoundStore';

interface AiStoreState {
  chatHistory: ChatMessage[];
  patternWarnings: PatternWarning[];
  streamingMetadata: StreamingMetadata | null;
  currentResponse: string;
  isStreaming: boolean;
  isLoading: boolean;
  error: string | null;

  // Actions
  sendMessage: (message: string, commandType?: string) => Promise<ChatResponse>;
  sendMessageStream: (message: string, commandType?: string) => Promise<void>;
  addMessage: (message: ChatMessage) => void;
  clearHistory: () => void;
  fetchPatternWarnings: () => Promise<void>;
  dismissPatternWarning: (warningId: string) => Promise<void>;
  optimizePortfolio: (holdings: Record<string, number>) => Promise<PortfolioOptimization>;
  applyOptimization: (optimizationId: string) => Promise<void>;
  submitFeedback: (messageId: string, score: number, comment: string) => Promise<void>;
  executeQuickAction: (
    actionId: string,
    actionType: string,
    token: string,
    amount?: number
  ) => Promise<void>;
  setError: (error: string | null) => void;
  reset: () => void;
}

const initialState = {
  chatHistory: [],
  patternWarnings: [],
  streamingMetadata: null,
  currentResponse: '',
  isStreaming: false,
  isLoading: false,
  error: null,
};

const storeResult = createBoundStore<AiStoreState>((set, get) => ({
  ...initialState,

  sendMessage: async (message: string, commandType?: string) => {
    const userMessage: ChatMessage = {
      role: 'user',
      content: message,
    };

    set({ isLoading: true, error: null });
    get().addMessage(userMessage);

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const response = await invoke<ChatResponse>('ai_chat_message', {
        message,
        commandType,
        history: get().chatHistory,
      });

      const assistantMessage: ChatMessage = {
        role: 'assistant',
        content: response.content,
      };
      get().addMessage(assistantMessage);

      set({ isLoading: false });
      return response;
    } catch (error) {
      set({ error: String(error), isLoading: false });
      throw error;
    }
  },

  sendMessageStream: async (message: string, commandType?: string) => {
    const userMessage: ChatMessage = {
      role: 'user',
      content: message,
    };

    set({ isStreaming: true, currentResponse: '', error: null });
    get().addMessage(userMessage);

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const { listen } = await import('@tauri-apps/api/event');

      const streamId = await invoke<string>('ai_chat_message_stream', {
        message,
        commandType,
        history: get().chatHistory,
      });

      const eventName = `ai:chat:${streamId}`;

      set({
        streamingMetadata: {
          streamId,
          eventName,
          isStreaming: true,
          currentChunk: '',
        },
      });

      const unlisten = await listen<any>(eventName, (event: any) => {
        const { chunk, done, reasoning } = event.payload;

        if (done) {
          const finalResponse = get().currentResponse;
          const assistantMessage: ChatMessage = {
            role: 'assistant',
            content: finalResponse,
          };
          get().addMessage(assistantMessage);

          set({
            isStreaming: false,
            currentResponse: '',
            streamingMetadata: null,
          });

          unlisten();
        } else if (chunk) {
          set(state => ({
            currentResponse: state.currentResponse + chunk,
            streamingMetadata: state.streamingMetadata
              ? { ...state.streamingMetadata, currentChunk: chunk }
              : null,
          }));
        }
      });
    } catch (error) {
      set({ error: String(error), isStreaming: false, currentResponse: '' });
      throw error;
    }
  },

  addMessage: (message: ChatMessage) => {
    set(state => ({
      chatHistory: [...state.chatHistory, message],
    }));
  },

  clearHistory: () => {
    set({ chatHistory: [] });
  },

  fetchPatternWarnings: async () => {
    set({ isLoading: true, error: null });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const warnings = await invoke<PatternWarning[]>('ai_get_pattern_warnings');
      set({ patternWarnings: warnings, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  dismissPatternWarning: async (warningId: string) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('ai_dismiss_pattern_warning', { warningId });
      set(state => ({
        patternWarnings: state.patternWarnings.filter(w => w.id !== warningId),
      }));
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  optimizePortfolio: async (holdings: Record<string, number>) => {
    set({ isLoading: true, error: null });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const optimization = await invoke<PortfolioOptimization>('ai_optimize_portfolio', {
        holdings,
      });
      set({ isLoading: false });
      return optimization;
    } catch (error) {
      set({ error: String(error), isLoading: false });
      throw error;
    }
  },

  applyOptimization: async (optimizationId: string) => {
    set({ isLoading: true, error: null });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('ai_apply_optimization', { optimizationId });
      set({ isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
      throw error;
    }
  },

  submitFeedback: async (messageId: string, score: number, comment: string) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('ai_submit_feedback', { messageId, score, comment });
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  executeQuickAction: async (
    actionId: string,
    actionType: string,
    token: string,
    amount?: number
  ) => {
    set({ isLoading: true, error: null });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('ai_execute_quick_action', {
        actionId,
        actionType,
        token,
        amount,
      });
      set({ isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
      throw error;
    }
  },

  setError: (error: string | null) => {
    if (get().error === error) return;
    set({ error });
  },

  reset: () => {
    set(initialState);
  },
}));

export const useAiStore = storeResult.useStore;
export const aiStore = storeResult.store;

export const useChatHistory = () => {
  return useAiStore(state => state.chatHistory);
};

export const usePatternWarnings = () => {
  return useAiStore(state => state.patternWarnings);
};

export const useStreamingStatus = () => {
  return useAiStore(state => ({
    isStreaming: state.isStreaming,
    currentResponse: state.currentResponse,
    metadata: state.streamingMetadata,
  }));
};
