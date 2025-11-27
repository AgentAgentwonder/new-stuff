import React, { useState } from 'react';
import { useAIChatStream } from '../../hooks';
import { Spinner } from '../LoadingOverlay';

/**
 * Example component demonstrating AI chat streaming functionality
 */
export function AIChatExample() {
  const [inputMessage, setInputMessage] = useState('');

  // Use the AI chat stream hook for managing chat state and streaming
  const { messages, isStreaming, content, sendMessage, clearChat } = useAIChatStream({
    onComplete: fullContent => {
      console.log('AI response completed:', fullContent);
    },
    onError: error => {
      console.error('Chat error:', error);
    },
  });

  const handleSendMessage = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!inputMessage.trim() || isStreaming) {
      return;
    }

    const message = inputMessage.trim();
    setInputMessage('');

    // Send message to AI with optional command type
    await sendMessage(message, 'market_analysis');
  };

  const handleClearChat = () => {
    clearChat();
    setInputMessage('');
  };

  return (
    <div className="flex flex-col h-full max-h-[600px] bg-background-secondary rounded-lg border border-border">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-border">
        <h3 className="text-lg font-semibold text-text">AI Assistant</h3>
        <button
          onClick={handleClearChat}
          className="px-3 py-1 text-sm text-text-secondary hover:text-text transition-colors"
        >
          Clear Chat
        </button>
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {messages.length === 0 && !isStreaming && (
          <div className="text-center text-text-secondary py-8">
            Start a conversation with the AI assistant
          </div>
        )}

        {messages.map((message, index) => (
          <div
            key={index}
            className={`flex ${message.role === 'user' ? 'justify-end' : 'justify-start'}`}
          >
            <div
              className={`max-w-[80%] p-3 rounded-lg ${
                message.role === 'user'
                  ? 'bg-primary text-white'
                  : 'bg-background border border-border text-text'
              }`}
            >
              <div className="text-sm whitespace-pre-wrap">{message.content}</div>
            </div>
          </div>
        ))}

        {/* Streaming response */}
        {isStreaming && (
          <div className="flex justify-start">
            <div className="max-w-[80%] bg-background border border-border text-text p-3 rounded-lg">
              <div className="flex items-start gap-2">
                <Spinner size="sm" />
                <div className="flex-1">
                  <div className="text-sm whitespace-pre-wrap">
                    {content || <span className="text-text-secondary">Thinking...</span>}
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Input */}
      <form onSubmit={handleSendMessage} className="p-4 border-t border-border">
        <div className="flex gap-2">
          <input
            type="text"
            value={inputMessage}
            onChange={e => setInputMessage(e.target.value)}
            placeholder="Ask about market analysis, trading strategies, or portfolio optimization..."
            className="flex-1 px-3 py-2 bg-background border border-border rounded text-text placeholder-text-muted disabled:opacity-50"
            disabled={isStreaming}
          />
          <button
            type="submit"
            disabled={!inputMessage.trim() || isStreaming}
            className="px-4 py-2 bg-primary text-white rounded hover:bg-primary-hover disabled:opacity-50 disabled:cursor-not-allowed transition-colors flex items-center gap-2"
          >
            {isStreaming ? <Spinner size="sm" /> : 'Send'}
          </button>
        </div>

        {/* Command type hints */}
        <div className="mt-2 text-xs text-text-muted">
          Try: "Analyze SOL market", "Optimize my portfolio", "Risk analysis", "Trading suggestions"
        </div>
      </form>
    </div>
  );
}
