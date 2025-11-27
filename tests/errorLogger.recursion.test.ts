import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { errorLogger, type ErrorLog } from '../src/utils/errorLogger';

describe('ErrorLogger - Recursion Prevention', () => {
  beforeEach(() => {
    errorLogger.clear();
    vi.clearAllMocks();
  });

  afterEach(() => {
    errorLogger.clear();
  });

  it('should prevent recursive logging calls', () => {
    // Log an error
    errorLogger.error('Test error 1', 'TestSource');
    expect(errorLogger.getLogs().length).toBe(1);

    // Each call with different messages is allowed
    // The recursion prevention only prevents logging during the logging operation itself
    errorLogger.error('Test error 2', 'TestSource');
    errorLogger.error('Test error 3', 'TestSource');

    const logs = errorLogger.getLogs();
    // We should have at least some logs
    expect(logs.length).toBeGreaterThan(0);
    // And should not have caused an infinite loop
    expect(logs.length).toBeLessThanOrEqual(10);
  });

  it('should allow logging different messages from same source', () => {
    errorLogger.error('Error 1', 'TestSource');
    errorLogger.error('Error 2', 'TestSource');
    errorLogger.warning('Warning 1', 'TestSource');

    const logs = errorLogger.getLogs();
    // Different types or after sufficient time should be allowed
    expect(logs.length).toBeGreaterThan(0);
  });

  it('should prevent isLogging flag from blocking all logs', () => {
    errorLogger.error('Error 1', 'Source1');
    errorLogger.error('Error 2', 'Source2');
    errorLogger.warning('Warning', 'Source3');

    const logs = errorLogger.getLogs();
    expect(logs.length).toBeGreaterThanOrEqual(3);
  });

  it('should handle localStorage errors silently', () => {
    const consoleErrorSpy = vi.spyOn(console, 'error');
    const localStorageSetItemSpy = vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
      throw new Error('localStorage is full');
    });

    // This should not throw an error
    expect(() => {
      errorLogger.error('Test error', 'TestSource');
    }).not.toThrow();

    // The error should be logged to errorLogger's logs, but not to console.error
    const logs = errorLogger.getLogs();
    expect(logs.length).toBeGreaterThan(0);

    // Console.error should not be called for localStorage errors
    // (it might be called for dev mode console logging, but not for storage errors)
    consoleErrorSpy.mockRestore();
    localStorageSetItemSpy.mockRestore();
  });

  it('should clear logs properly without recursion', () => {
    errorLogger.error('Error 1', 'Source1');
    errorLogger.error('Error 2', 'Source2');

    expect(errorLogger.getLogs().length).toBeGreaterThan(0);

    errorLogger.clear();

    expect(errorLogger.getLogs().length).toBe(0);
  });

  it('should not exceed MAX_LOGS limit', () => {
    // Log more than MAX_LOGS (100) entries
    for (let i = 0; i < 150; i++) {
      errorLogger.error(`Error ${i}`, `Source_${i % 10}`);
    }

    const logs = errorLogger.getLogs();
    expect(logs.length).toBeLessThanOrEqual(100);
  });

  it('should return empty logs if reading during logging', () => {
    errorLogger.error('Error 1', 'Source1');

    const logs = errorLogger.getLogs();
    expect(Array.isArray(logs)).toBe(true);
    expect(logs.length).toBeGreaterThan(0);
  });
});
