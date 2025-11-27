/**
 * Module Load Logger - Tracks when modules are imported and loaded
 * Helps diagnose initialization deadlocks and freezes
 */

interface ModuleLoadEvent {
  timestamp: number;
  module: string;
  stage: 'start' | 'end' | 'error';
  duration?: number;
  error?: string;
}

class ModuleLogger {
  private logs: ModuleLoadEvent[] = [];
  private loadingModules = new Map<string, number>();
  private enabled = true;

  start(module: string) {
    if (!this.enabled) return;
    
    const timestamp = performance.now();
    this.loadingModules.set(module, timestamp);
    
    const event: ModuleLoadEvent = {
      timestamp,
      module,
      stage: 'start',
    };
    
    this.logs.push(event);
    console.log(`[MODULE] → Loading: ${module}`);
    
    // Store in sessionStorage for debugging
    this.syncToStorage();
  }

  end(module: string) {
    if (!this.enabled) return;
    
    const timestamp = performance.now();
    const startTime = this.loadingModules.get(module);
    const duration = startTime ? timestamp - startTime : 0;
    
    const event: ModuleLoadEvent = {
      timestamp,
      module,
      stage: 'end',
      duration,
    };
    
    this.logs.push(event);
    console.log(`[MODULE] ✓ Loaded: ${module} (${duration.toFixed(2)}ms)`);
    
    this.loadingModules.delete(module);
    this.syncToStorage();
  }

  error(module: string, error: Error | string) {
    if (!this.enabled) return;
    
    const timestamp = performance.now();
    const errorMsg = error instanceof Error ? error.message : String(error);
    
    const event: ModuleLoadEvent = {
      timestamp,
      module,
      stage: 'error',
      error: errorMsg,
    };
    
    this.logs.push(event);
    console.error(`[MODULE] ✗ Failed: ${module}`, error);
    
    this.loadingModules.delete(module);
    this.syncToStorage();
  }

  getLogs(): ModuleLoadEvent[] {
    return [...this.logs];
  }

  getReport(): string {
    let report = '=== MODULE LOAD REPORT ===\n\n';
    
    let totalTime = 0;
    let slowestModule = '';
    let slowestTime = 0;

    this.logs.forEach((log, index) => {
      const time = log.timestamp.toFixed(2);
      const duration = log.duration ? ` (${log.duration.toFixed(2)}ms)` : '';
      const status = log.stage === 'start' ? '→' : log.stage === 'end' ? '✓' : '✗';
      const error = log.error ? ` - ERROR: ${log.error}` : '';
      
      report += `${time}ms [${status}] ${log.module}${duration}${error}\n`;

      if (log.duration && log.duration > slowestTime) {
        slowestTime = log.duration;
        slowestModule = log.module;
      }

      if (log.duration) {
        totalTime += log.duration;
      }
    });

    report += `\n=== SUMMARY ===\n`;
    report += `Total modules loaded: ${this.logs.filter(l => l.stage === 'end').length}\n`;
    report += `Failed modules: ${this.logs.filter(l => l.stage === 'error').length}\n`;
    report += `Total time: ${totalTime.toFixed(2)}ms\n`;
    report += `Slowest module: ${slowestModule} (${slowestTime.toFixed(2)}ms)\n`;

    const stillLoading = Array.from(this.loadingModules.keys());
    if (stillLoading.length > 0) {
      report += `\n=== STILL LOADING (POTENTIAL DEADLOCK) ===\n`;
      stillLoading.forEach(module => {
        const startTime = this.loadingModules.get(module) || 0;
        const hangTime = performance.now() - startTime;
        report += `⏳ ${module} (${hangTime.toFixed(2)}ms)\n`;
      });
    }

    return report;
  }

  printReport() {
    console.log(this.getReport());
  }

  private syncToStorage() {
    try {
      sessionStorage.setItem('eclipse_module_logs', JSON.stringify({
        logs: this.logs,
        stillLoading: Array.from(this.loadingModules.keys()),
      }));
    } catch (e) {
      // Fail silently
    }
  }

  disable() {
    this.enabled = false;
  }

  clear() {
    this.logs = [];
    this.loadingModules.clear();
  }
}

export const moduleLogger = new ModuleLogger();

/**
 * Wraps a dynamic import to automatically log loading
 * Usage: await loggedImport('./module', () => import('./module'))
 */
export async function loggedImport<T>(
  moduleName: string,
  importFn: () => Promise<T>
): Promise<T> {
  moduleLogger.start(moduleName);
  try {
    const result = await importFn();
    moduleLogger.end(moduleName);
    return result;
  } catch (error) {
    moduleLogger.error(moduleName, error instanceof Error ? error : new Error(String(error)));
    throw error;
  }
}
