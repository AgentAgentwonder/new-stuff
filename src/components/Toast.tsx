import React, { useEffect, useState, useCallback } from 'react';
import { useUIStore, type ToastMessage } from '../store/uiStore';

interface ToastProps {
  toast: ToastMessage;
  onRemove: (id: string) => void;
}

/**
 * Individual toast component
 */
export function Toast({ toast, onRemove }: ToastProps) {
  const [isVisible, setIsVisible] = useState(false);
  const [isLeaving, setIsLeaving] = useState(false);

  useEffect(() => {
    // Trigger entrance animation
    const timer = setTimeout(() => setIsVisible(true), 10);
    return () => clearTimeout(timer);
  }, []);

  const handleRemove = () => {
    setIsLeaving(true);
    setTimeout(() => onRemove(toast.id), 300); // Wait for exit animation
  };

  const getIcon = () => {
    switch (toast.type) {
      case 'success':
        return (
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
          </svg>
        );
      case 'error':
        return (
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        );
      case 'warning':
        return (
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
            />
          </svg>
        );
      case 'info':
      default:
        return (
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
        );
    }
  };

  const getColors = () => {
    switch (toast.type) {
      case 'success':
        return 'bg-success text-white';
      case 'error':
        return 'bg-error text-white';
      case 'warning':
        return 'bg-warning text-white';
      case 'info':
      default:
        return 'bg-info text-white';
    }
  };

  return (
    <div
      className={`
        flex items-start gap-3 p-4 rounded-lg shadow-lg border border-border
        min-w-[300px] max-w-md
        transform transition-all duration-300 ease-out
        ${isVisible && !isLeaving ? 'translate-x-0 opacity-100 scale-100' : 'translate-x-full opacity-0 scale-95'}
        ${isLeaving ? 'translate-x-full opacity-0 scale-95' : ''}
      `}
    >
      <div className={`flex-shrink-0 rounded-full p-1 ${getColors()}`}>{getIcon()}</div>

      <div className="flex-1 min-w-0">
        <h4 className="font-semibold text-text text-sm">{toast.title}</h4>
        {toast.message && <p className="text-text-secondary text-sm mt-1">{toast.message}</p>}
        {toast.action && (
          <button
            onClick={toast.action.onClick}
            className="mt-2 text-xs font-medium text-primary hover:text-primary-hover transition-colors"
          >
            {toast.action.label}
          </button>
        )}
      </div>

      <button
        onClick={handleRemove}
        className="flex-shrink-0 p-1 text-text-muted hover:text-text transition-colors"
      >
        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M6 18L18 6M6 6l12 12"
          />
        </svg>
      </button>
    </div>
  );
}

/**
 * Toast container component that manages all toasts
 */
export function ToastContainer() {
  const uiSelector = useCallback(
    (state: ReturnType<typeof useUIStore.getState>) => ({
      toasts: state.toasts,
      removeToast: state.removeToast,
    }),
    []
  );
  const { toasts, removeToast } = useUIStore(uiSelector);

  if (toasts.length === 0) {
    return null;
  }

  return (
    <div className="fixed top-4 right-4 z-50 space-y-2 pointer-events-none">
      {toasts.map(toast => (
        <div key={toast.id} className="pointer-events-auto">
          <Toast toast={toast} onRemove={removeToast} />
        </div>
      ))}
    </div>
  );
}

/**
 * Hook for easy toast management
 */
export function useToast() {
  const uiSelector = useCallback(
    (state: ReturnType<typeof useUIStore.getState>) => ({
      addToast: state.addToast,
      removeToast: state.removeToast,
      clearToasts: state.clearToasts,
    }),
    []
  );
  const { addToast, removeToast, clearToasts } = useUIStore(uiSelector);

  return {
    success: (
      title: string,
      message?: string,
      options?: Partial<Omit<ToastMessage, 'id' | 'type' | 'title' | 'message'>>
    ) => addToast({ type: 'success', title, message, ...options }),

    error: (
      title: string,
      message?: string,
      options?: Partial<Omit<ToastMessage, 'id' | 'type' | 'title' | 'message'>>
    ) => addToast({ type: 'error', title, message, ...options }),

    warning: (
      title: string,
      message?: string,
      options?: Partial<Omit<ToastMessage, 'id' | 'type' | 'title' | 'message'>>
    ) => addToast({ type: 'warning', title, message, ...options }),

    info: (
      title: string,
      message?: string,
      options?: Partial<Omit<ToastMessage, 'id' | 'type' | 'title' | 'message'>>
    ) => addToast({ type: 'info', title, message, ...options }),

    remove: removeToast,
    clear: clearToasts,
  };
}
