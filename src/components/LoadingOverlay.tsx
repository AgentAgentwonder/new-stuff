import React, { useCallback } from 'react';
import { useUIStore } from '../store/uiStore';

interface LoadingOverlayProps {
  show?: boolean;
  message?: string;
  progress?: number;
  backdrop?: boolean;
  spinner?: boolean;
  children?: React.ReactNode;
}

/**
 * Global loading overlay component that can be controlled via props or UI store
 */
export function LoadingOverlay({
  show,
  message,
  progress,
  backdrop = true,
  spinner = true,
  children,
}: LoadingOverlayProps) {
  const uiSelector = useCallback(
    (state: ReturnType<typeof useUIStore.getState>) => ({
      isAppLoading: state.isAppLoading,
      appLoadingMessage: state.appLoadingMessage,
    }),
    []
  );
  const { isAppLoading, appLoadingMessage } = useUIStore(uiSelector);

  // Determine whether to show the overlay
  const shouldShow = show !== undefined ? show : isAppLoading;
  const displayMessage = message ?? appLoadingMessage;

  if (!shouldShow) {
    return null;
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {backdrop && <div className="absolute inset-0 bg-black/50 backdrop-blur-sm" />}

      <div className="relative bg-background-secondary border border-border rounded-lg p-6 max-w-sm w-full mx-4 shadow-lg">
        {spinner && (
          <div className="flex justify-center mb-4">
            <div className="animate-spin rounded-full h-8 w-8 border-2 border-primary border-t-transparent" />
          </div>
        )}

        {displayMessage && <p className="text-center text-text-secondary mb-2">{displayMessage}</p>}

        {progress !== undefined && (
          <div className="w-full bg-background-tertiary rounded-full h-2 mb-2">
            <div
              className="bg-primary h-2 rounded-full transition-all duration-300 ease-out"
              style={{ width: `${Math.min(100, Math.max(0, progress))}%` }}
            />
          </div>
        )}

        {progress !== undefined && (
          <p className="text-center text-xs text-text-muted">{Math.round(progress)}% complete</p>
        )}

        {children && <div className="mt-4">{children}</div>}
      </div>
    </div>
  );
}

/**
 * Simple spinner component for inline loading states
 */
interface SpinnerProps {
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}

export function Spinner({ size = 'md', className = '' }: SpinnerProps) {
  const sizeClasses = {
    sm: 'h-4 w-4',
    md: 'h-6 w-6',
    lg: 'h-8 w-8',
  };

  return (
    <div
      className={`animate-spin rounded-full border-2 border-primary border-t-transparent ${sizeClasses[size]} ${className}`}
    />
  );
}

/**
 * Loading skeleton component for placeholder content
 */
interface SkeletonProps {
  className?: string;
  lines?: number;
  height?: string;
}

export function Skeleton({ className = '', lines = 1, height = '1rem' }: SkeletonProps) {
  return (
    <div className={`space-y-2 ${className}`}>
      {Array.from({ length: lines }, (_, i) => (
        <div
          key={i}
          className="bg-background-tertiary rounded animate-pulse"
          style={{
            height,
            width: i === lines - 1 ? '75%' : '100%', // Last line is shorter
          }}
        />
      ))}
    </div>
  );
}

/**
 * Card skeleton for loading card layouts
 */
export function CardSkeleton({ className = '' }: { className?: string }) {
  return (
    <div className={`bg-background-secondary border border-border rounded-lg p-4 ${className}`}>
      <div className="flex items-center space-x-4">
        <Skeleton className="h-12 w-12 rounded-full" />
        <div className="flex-1 space-y-2">
          <Skeleton height="1rem" lines={2} />
        </div>
      </div>
      <div className="mt-4">
        <Skeleton height="0.75rem" lines={3} />
      </div>
    </div>
  );
}

/**
 * Table skeleton for loading table layouts
 */
export function TableSkeleton({
  rows = 5,
  columns = 4,
  className = '',
}: {
  rows?: number;
  columns?: number;
  className?: string;
}) {
  return (
    <div className={`space-y-2 ${className}`}>
      {/* Header */}
      <div className="flex space-x-4 p-4 bg-background-tertiary rounded">
        {Array.from({ length: columns }, (_, i) => (
          <Skeleton key={`header-${i}`} className="flex-1" height="1rem" />
        ))}
      </div>

      {/* Rows */}
      {Array.from({ length: rows }, (_, rowIndex) => (
        <div
          key={`row-${rowIndex}`}
          className="flex space-x-4 p-4 bg-background-secondary border border-border rounded"
        >
          {Array.from({ length: columns }, (_, colIndex) => (
            <Skeleton key={`cell-${rowIndex}-${colIndex}`} className="flex-1" height="0.875rem" />
          ))}
        </div>
      ))}
    </div>
  );
}

/**
 * List skeleton for loading list layouts
 */
export function ListSkeleton({
  items = 5,
  className = '',
}: {
  items?: number;
  className?: string;
}) {
  return (
    <div className={`space-y-3 ${className}`}>
      {Array.from({ length: items }, (_, i) => (
        <div
          key={`item-${i}`}
          className="flex items-center space-x-3 p-3 bg-background-secondary border border-border rounded"
        >
          <Skeleton className="h-10 w-10 rounded" />
          <div className="flex-1">
            <Skeleton height="1rem" lines={1} />
            <Skeleton height="0.75rem" lines={1} className="w-3/4 mt-1" />
          </div>
          <Skeleton className="h-8 w-20 rounded" />
        </div>
      ))}
    </div>
  );
}
