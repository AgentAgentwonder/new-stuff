import { useState, useCallback, useEffect, useRef } from 'react';
import { useUIStore } from '../store/uiStore';
import type { StreamingChunk, TauriError } from '../lib/tauri/types';

interface UseStreamingCommandOptions {
  onChunk?: (chunk: StreamingChunk) => void;
  onComplete?: (fullContent: string) => void;
  onError?: (error: TauriError) => void;
  showToastOnError?: boolean;
  loadingId?: string;
  loadingMessage?: string;
}

interface UseStreamingCommandReturn {
  isStreaming: boolean;
  content: string;
  error: TauriError | null;
  startStream: (...args: any[]) => Promise<string | null>;
  stopStream: () => void;
  reset: () => void;
}

/**
 * Hook for handling streaming Tauri commands (like AI chat)
 *
 * @param options - Configuration options
 * @returns Hook state and control functions
 */
export function useStreamingCommand(
  options: UseStreamingCommandOptions = {}
): UseStreamingCommandReturn {
  const {
    onChunk,
    onComplete,
    onError,
    showToastOnError = true,
    loadingId,
    loadingMessage,
  } = options;

  const [isStreaming, setIsStreaming] = useState(false);
  const [content, setContent] = useState('');
  const [error, setError] = useState<TauriError | null>(null);
  const [streamId, setStreamId] = useState<string | null>(null);

  const uiSelector = useCallback(
    (state: ReturnType<typeof useUIStore.getState>) => ({
      setLoading: state.setLoading,
      addToast: state.addToast,
    }),
    []
  );
  const { setLoading, addToast } = useUIStore(uiSelector);
  const mountedRef = useRef(true);
  const contentRef = useRef('');
  const streamIdRef = useRef<string | null>(null);

  // Reset state
  const reset = useCallback(() => {
    setContent('');
    contentRef.current = '';
    setError(null);
    setIsStreaming(false);

    if (streamIdRef.current) {
      import('../lib/tauri/commands').then(({ StreamingCommandManager }) => {
        StreamingCommandManager.stopChatStream(streamIdRef.current!);
      });
      streamIdRef.current = null;
      setStreamId(null);
    }
  }, []);

  // Stop streaming
  const stopStream = useCallback(() => {
    if (streamIdRef.current) {
      import('../lib/tauri/commands').then(({ StreamingCommandManager }) => {
        StreamingCommandManager.stopChatStream(streamIdRef.current!);
      });
      streamIdRef.current = null;
      setStreamId(null);
    }

    if (mountedRef.current) {
      setIsStreaming(false);
    }

    if (loadingId) {
      setLoading(loadingId, false);
    }
  }, [loadingId, setLoading]);

  // Start streaming
  const startStream = useCallback(
    async (...args: any[]): Promise<string | null> => {
      // Reset previous state
      reset();

      // Extract arguments for AI chat stream
      const [message, commandType, history] = args;

      setIsStreaming(true);
      setError(null);

      // Set loading state in UI store if loadingId provided
      if (loadingId) {
        setLoading(loadingId, true, loadingMessage);
      }

      try {
        const { StreamingCommandManager } = await import('../lib/tauri/commands');
        const streamId = await StreamingCommandManager.startChatStream(
          message,
          commandType,
          history,
          (chunk: StreamingChunk) => {
            if (!mountedRef.current) {
              return;
            }

            // Handle chunk
            if (chunk.error) {
              const errorObj: TauriError = {
                message: chunk.error,
              };

              setError(errorObj);
              setIsStreaming(false);
              onError?.(errorObj);

              // Show toast error if enabled
              if (showToastOnError) {
                addToast({
                  type: 'error',
                  title: 'Stream Error',
                  message: chunk.error,
                });
              }

              if (loadingId) {
                setLoading(loadingId, false);
              }
              return;
            }

            // Append content
            const newContent = contentRef.current + chunk.content;
            contentRef.current = newContent;
            setContent(newContent);

            // Call chunk callback
            onChunk?.(chunk);

            // Handle completion
            if (chunk.finished) {
              setIsStreaming(false);
              streamIdRef.current = null;
              setStreamId(null);

              if (loadingId) {
                setLoading(loadingId, false);
              }

              onComplete?.(newContent);
            }
          }
        );

        if (mountedRef.current) {
          streamIdRef.current = streamId;
          setStreamId(streamId);
        }

        return streamId;
      } catch (err) {
        const errorObj: TauriError = {
          message: err instanceof Error ? err.message : 'Failed to start stream',
        };

        if (mountedRef.current) {
          setError(errorObj);
          setIsStreaming(false);
          onError?.(errorObj);

          // Show toast error if enabled
          if (showToastOnError) {
            addToast({
              type: 'error',
              title: 'Stream Failed',
              message: errorObj.message,
            });
          }
        }

        if (loadingId) {
          setLoading(loadingId, false);
        }

        return null;
      }
    },
    [
      reset,
      onChunk,
      onComplete,
      onError,
      showToastOnError,
      loadingId,
      loadingMessage,
      setLoading,
      addToast,
    ]
  );

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      mountedRef.current = false;
      if (streamIdRef.current) {
        import('../lib/tauri/commands').then(({ StreamingCommandManager }) => {
          StreamingCommandManager.stopChatStream(streamIdRef.current!);
        });
      }
    };
  }, []);

  return {
    isStreaming,
    content,
    error,
    startStream,
    stopStream,
    reset,
  };
}

/**
 * Hook specifically for AI chat streaming with additional chat-specific utilities
 */
export function useAIChatStream(options: Omit<UseStreamingCommandOptions, 'onChunk'> = {}) {
  const [messages, setMessages] = useState<Array<{ role: string; content: string }>>([]);
  const [isTyping, setIsTyping] = useState(false);

  const streamingCommand = useStreamingCommand({
    ...options,
    onChunk: chunk => {
      setIsTyping(true);
      options.onChunk?.(chunk);
    },
    onComplete: fullContent => {
      setIsTyping(false);
      setMessages(prev => [...prev, { role: 'assistant', content: fullContent }]);
      options.onComplete?.(fullContent);
    },
    onError: error => {
      setIsTyping(false);
      options.onError?.(error);
    },
  });

  const sendMessage = useCallback(
    async (
      message: string,
      commandType?: string,
      history?: Array<{ role: string; content: string }>
    ) => {
      // Add user message to history
      const userMessage = { role: 'user' as const, content: message };
      setMessages(prev => [...prev, userMessage]);

      // Start streaming with provided history or current messages
      const chatHistory = history || messages;
      return streamingCommand.startStream(message, commandType, chatHistory);
    },
    [messages, streamingCommand]
  );

  const clearChat = useCallback(() => {
    setMessages([]);
    streamingCommand.reset();
    setIsTyping(false);
  }, [streamingCommand]);

  return {
    ...streamingCommand,
    messages,
    isTyping,
    sendMessage,
    clearChat,
  };
}
