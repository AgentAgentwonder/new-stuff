import React, { Component, type ErrorInfo, type ReactNode, useCallback } from 'react';
import { errorLogger } from '@/utils/errorLogger';
import { useUIStore } from '../store/uiStore';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

/**
 * Error Boundary component that catches React errors and provides a fallback UI.
 * Integrates with the UI store for consistent error handling and notifications.
 */
export class AppErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
    };
  }

  static getDerivedStateFromError(error: Error): Partial<State> {
    return {
      hasError: true,
      error,
    };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    this.setState({
      error,
      errorInfo,
    });

    // Log error using the centralized error logger
    errorLogger.error(error.message, 'AppErrorBoundary', error, {
      componentStack: errorInfo.componentStack,
    });

    // Call custom error handler if provided
    this.props.onError?.(error, errorInfo);

    // Log error to console in development
    if (process.env.NODE_ENV === 'development') {
      console.error('Error Boundary caught an error:', error, errorInfo);
    }
  }

  handleReset = () => {
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null,
    });
  };

  render() {
    if (this.state.hasError) {
      // Use custom fallback if provided
      if (this.props.fallback) {
        return this.props.fallback;
      }

      // Default error fallback
      return <ErrorFallback error={this.state.error} onReset={this.handleReset} />;
    }

    return this.props.children;
  }
}

interface ErrorFallbackProps {
  error: Error | null;
  onReset: () => void;
}

/**
 * Default error fallback component
 */
function ErrorFallback({ error, onReset }: ErrorFallbackProps) {
  const uiSelector = useCallback(
    (state: ReturnType<typeof useUIStore.getState>) => ({
      addToast: state.addToast,
    }),
    []
  );
  const { addToast } = useUIStore(uiSelector);

  const handleReportError = () => {
    // In a real app, this would send the error to a reporting service
    const errorReport = {
      message: error?.message || 'Unknown error',
      stack: error?.stack,
      timestamp: new Date().toISOString(),
      userAgent: navigator.userAgent,
      url: window.location.href,
    };

    console.log('Error report:', errorReport);

    addToast({
      type: 'info',
      title: 'Error Reported',
      message: 'The error has been logged for investigation.',
    });
  };

  const handleCopyError = () => {
    const errorText = [
      'Error Details:',
      `Message: ${error?.message || 'Unknown error'}`,
      `Timestamp: ${new Date().toISOString()}`,
      `URL: ${window.location.href}`,
      '',
      'Stack Trace:',
      error?.stack || 'No stack trace available',
    ].join('\n');

    navigator.clipboard.writeText(errorText).then(() => {
      addToast({
        type: 'success',
        title: 'Copied to Clipboard',
        message: 'Error details have been copied to your clipboard.',
      });
    });
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-background p-4">
      <div className="max-w-md w-full bg-background-secondary rounded-lg border border-border p-6 text-center">
        <div className="mb-4">
          <div className="w-16 h-16 bg-error/10 rounded-full flex items-center justify-center mx-auto mb-4">
            <svg
              className="w-8 h-8 text-error"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
              />
            </svg>
          </div>

          <h1 className="text-xl font-semibold text-text mb-2">Something went wrong</h1>

          <p className="text-text-secondary mb-4">
            {error?.message || 'An unexpected error occurred while rendering this component.'}
          </p>

          {process.env.NODE_ENV === 'development' && error?.stack && (
            <details className="mb-4 text-left">
              <summary className="cursor-pointer text-sm text-text-muted hover:text-text-secondary mb-2">
                View technical details
              </summary>
              <pre className="bg-background-tertiary p-3 rounded text-xs text-text-muted overflow-auto max-h-32">
                {error.stack}
              </pre>
            </details>
          )}
        </div>

        <div className="flex flex-col gap-2">
          <button
            onClick={onReset}
            className="w-full bg-primary text-white px-4 py-2 rounded hover:bg-primary-hover transition-colors"
          >
            Try Again
          </button>

          <div className="flex gap-2">
            <button
              onClick={handleCopyError}
              className="flex-1 bg-background-tertiary text-text px-3 py-2 rounded hover:bg-background transition-colors text-sm"
            >
              Copy Error
            </button>

            <button
              onClick={handleReportError}
              className="flex-1 bg-background-tertiary text-text px-3 py-2 rounded hover:bg-background transition-colors text-sm"
            >
              Report Issue
            </button>
          </div>
        </div>

        <p className="text-xs text-text-muted mt-4">
          If this problem persists, please contact support.
        </p>
      </div>
    </div>
  );
}

/**
 * Higher-order component that wraps a component in an error boundary
 */
export function withErrorBoundary<P extends object>(
  Component: React.ComponentType<P>,
  errorBoundaryProps?: Omit<Props, 'children'>
) {
  const WrappedComponent = (props: P) => (
    <AppErrorBoundary {...errorBoundaryProps}>
      <Component {...props} />
    </AppErrorBoundary>
  );

  WrappedComponent.displayName = `withErrorBoundary(${Component.displayName || Component.name})`;

  return WrappedComponent;
}
