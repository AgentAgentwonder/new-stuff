# Performance Optimization - Phase 7

This document describes the rendering, loading, and monitoring optimizations implemented in Phase 7 (Tasks 7.11–7.16).

## Overview

The performance optimization phase introduces GPU acceleration, virtual scrolling, web workers for heavy computations, memory optimization, and real-time performance monitoring to ensure the application remains responsive even with large datasets and complex visualizations.

## Features Implemented

### 1. GPU Acceleration (Task 7.11)

#### WebGL Rendering
- **WebGLRenderer** class provides GPU-accelerated rendering for charts and animations
- Automatic fallback to Canvas2D when WebGL is unavailable
- GPU detection and capability reporting

#### Components
- `src/utils/gpu.ts` - GPU detection and WebGL renderer
- `src/components/charts/WebGLLineChart.tsx` - WebGL-accelerated line chart with automatic fallback

#### Features
- Detect WebGL support and GPU capabilities
- Toggle GPU acceleration on/off
- Monitor GPU usage (draw calls, triangles)
- Graceful degradation to CPU rendering

#### Usage
```tsx
import { WebGLLineChart } from './components/charts/WebGLLineChart';

<WebGLLineChart
  data={chartData}
  width={800}
  height={400}
  color={[0.65, 0.33, 0.96, 1.0]}
/>
```

### 2. Virtual Scrolling & Lazy Loading (Task 7.12)

#### Virtual Scrolling
- **VirtualList** component renders only visible items
- Built-in custom implementation in `src/components/insiders/VirtualizedList.tsx`
- react-window integration in `src/components/common/VirtualList.tsx`
- Configurable overscan for smooth scrolling

#### Lazy Loading
- **LazyLoad** wrapper for components
- **LazyImage** for images with intersection observer
- Skeleton placeholders during loading
- Intersection observer-based preloading

#### Components
- `src/components/common/VirtualList.tsx` - react-window wrapper
- `src/components/common/LazyLoad.tsx` - Lazy loading wrapper
- `src/components/common/LazyImage.tsx` - Lazy loaded images
- `src/components/common/Skeleton.tsx` - Loading skeletons
- `src/hooks/useIntersectionObserver.ts` - Intersection observer hook

#### Usage
```tsx
// Virtual scrolling
import { VirtualList } from './components/common/VirtualList';

<VirtualList
  items={largeDataset}
  height={600}
  itemHeight={80}
  renderItem={(item, index) => <ItemRow item={item} />}
/>

// Lazy loading
import { LazyLoad } from './components/common/LazyLoad';

<LazyLoad height={200}>
  <ExpensiveComponent />
</LazyLoad>

// Lazy images
import { LazyImage } from './components/common/LazyImage';

<LazyImage
  src="/large-image.jpg"
  alt="Description"
  threshold={0.1}
/>
```

### 3. Web Workers (Task 7.13)

#### Worker Pool
- **WorkerPool** class manages multiple workers efficiently
- Configurable pool size (defaults to CPU core count)
- Task queue with automatic load balancing
- Cancelable operations
- Worker status monitoring

#### Workers
- `src/workers/computation.worker.ts` - Heavy computations (MA, RSI, Bollinger Bands, data aggregation)
- `src/workers/indicator-calculator.worker.ts` - Custom indicator evaluation
- `src/workers/sentiment.worker.ts` - Sentiment analysis

#### Components
- `src/utils/workerPool.ts` - Worker pool manager
- `src/hooks/useWorkerPool.ts` - React hook for worker pool

#### Usage
```tsx
import { useWorkerPool } from './hooks/useWorkerPool';

const pool = useWorkerPool(
  () => new Worker(new URL('../workers/computation.worker.ts', import.meta.url), { type: 'module' }),
  { poolSize: 4 }
);

// Execute task
const result = await pool.execute('calculateMA', { data: prices, period: 20 });

// Cancel task
pool.cancelTask(taskId);

// Get pool status
const status = pool.getStatus();
```

#### Supported Computations
- Moving Average (MA)
- Relative Strength Index (RSI)
- Bollinger Bands
- Large array sorting
- Data filtering
- Price data aggregation

### 4. Memory Optimization (Task 7.14)

#### Features
- Memory profiling and leak detection
- WeakMap caches for automatic cleanup
- LRU (Least Recently Used) cache with configurable size
- Low-memory mode detection and activation
- History size limiting

#### Components
- `src/utils/memory.ts` - Memory utilities and caches

#### Usage
```tsx
import { LRUCache, WeakCache, getMemoryInfo, detectMemoryLeaks } from './utils/memory';

// LRU Cache
const cache = new LRUCache<string, Data>(100);
cache.set('key', data);
const value = cache.get('key');

// WeakMap Cache (automatic cleanup)
const weakCache = new WeakCache<object, string>();
weakCache.set(objectKey, 'value');

// Memory monitoring
const memInfo = getMemoryInfo();
console.log(`Memory usage: ${(memInfo.usageRatio * 100).toFixed(1)}%`);

// Detect leaks
detectMemoryLeaks(50 * 1024 * 1024); // 50MB threshold
```

### 5. Performance Monitor (Task 7.15)

#### Features
- Real-time FPS tracking
- Frame time measurement
- CPU load monitoring (via Long Task API)
- GPU load estimation
- Memory usage tracking
- Network downlink speed
- Performance budget alerts
- Metrics export

#### Components
- `src/components/common/PerformanceMonitor.tsx` - Overlay UI
- `src/hooks/usePerformanceMonitor.ts` - Performance measurement hook
- `src/store/performanceStore.ts` - State management

#### Metrics Tracked
- **FPS**: Frames per second
- **Frame Time**: Time per frame in milliseconds
- **CPU Load**: CPU utilization percentage
- **GPU Load**: Draw calls and estimated usage
- **Memory**: Heap usage (used/total)
- **Network**: Downlink speed and connection type

#### Performance Budgets
Default thresholds:
- FPS: minimum 30 fps
- Frame Time: maximum 55ms
- CPU Load: maximum 85%
- GPU Load: maximum 90%
- Memory: maximum 80% of available
- Network: minimum 1 Mbps for streaming

#### Usage
The Performance Monitor is automatically integrated into the app:

```tsx
import { PerformanceMonitor } from './components/common/PerformanceMonitor';

// In App.tsx
<PerformanceMonitor position="bottom-right" />
```

Features:
- Click to expand/collapse
- Real-time metrics display
- GPU acceleration toggle
- Alert notifications
- Export report button

### 6. Tests (Task 7.16)

#### Test Coverage
- Worker pool functionality
- LRU and WeakMap caches
- Memory optimization utilities
- Virtual scrolling calculations
- GPU detection
- Worker computations (MA, aggregation)
- Performance metrics tracking

#### Test File
- `src/__tests__/performance.test.ts`

#### Running Tests
```bash
npm test performance.test.ts
```

## Architecture

### Performance Store
Central state management for performance metrics:
```typescript
interface PerformanceState {
  gpuInfo: GPUInfo | null;
  gpuStats: GPUStats | null;
  gpuEnabled: boolean;
  lowMemoryMode: boolean;
  metrics: PerformanceMetrics;
  history: PerformanceMetrics[];
  budgets: PerformanceBudget[];
  alerts: PerformanceAlert[];
}
```

### Worker Pool Architecture
```
┌─────────────┐
│ Application │
└──────┬──────┘
       │
       ▼
┌─────────────┐     ┌─────────┐
│ Worker Pool ├────▶│ Worker 1│
└─────────────┘     ├─────────┤
       │            │ Worker 2│
       │            ├─────────┤
       │            │ Worker 3│
       │            ├─────────┤
       │            │ Worker 4│
       │            └─────────┘
       │
       ▼
┌──────────────┐
│  Task Queue  │
└──────────────┘
```

### Virtual Scrolling Architecture
```
┌────────────────────┐
│  Container         │ ← Fixed height
│  ┌──────────────┐  │
│  │ Virtual      │  │ ← Total height
│  │ Content      │  │
│  │              │  │
│  │ ┌──────────┐ │  │
│  │ │ Visible  │ │  │ ← Rendered items
│  │ │ Items    │ │  │
│  │ └──────────┘ │  │
│  │              │  │
│  └──────────────┘  │
└────────────────────┘
```

## Performance Best Practices

### When to Use GPU Acceleration
✅ **Use for:**
- Large datasets with continuous updates
- Complex visualizations
- Real-time animations
- Multiple charts

❌ **Avoid for:**
- Simple static charts
- Small datasets (< 100 points)
- Environments where WebGL is unreliable

### Virtual Scrolling Guidelines
- Use for lists with > 100 items
- Set appropriate `itemHeight` for accurate scrolling
- Use `overscan` prop to prevent blank areas during fast scrolling
- Combine with lazy loading for nested content

### Worker Delegation
**Good candidates:**
- Mathematical computations (indicators, statistics)
- Data transformation/aggregation
- Sorting/filtering large datasets
- Complex parsing operations

**Poor candidates:**
- DOM manipulation
- Simple operations (< 10ms CPU time)
- Operations requiring frequent communication with main thread

### Memory Management
- Enable low-memory mode automatically at 75% usage
- Use WeakMap for object caches that reference DOM elements
- Use LRU cache for data that needs deterministic cleanup
- Limit history buffers (default: 120 samples)
- Monitor for memory leaks in development

## Configuration

### GPU Acceleration
```typescript
import { usePerformanceStore } from './store/performanceStore';

const { gpuEnabled, setGpuEnabled } = usePerformanceStore();

// Toggle GPU acceleration
setGpuEnabled(true);
```

### Performance Budgets
```typescript
import { usePerformanceStore } from './store/performanceStore';

const { setBudgets } = usePerformanceStore();

setBudgets([
  { metric: 'fps', threshold: 60, description: 'Target FPS' },
  { metric: 'frameTime', threshold: 16.67, description: '60 FPS target' },
]);
```

### Worker Pool Size
```typescript
const pool = useWorkerPool(workerFactory, { 
  poolSize: navigator.hardwareConcurrency || 4 
});
```

## Browser Compatibility

### WebGL Support
- Chrome/Edge: ✅ Full support
- Firefox: ✅ Full support
- Safari: ✅ Full support
- Opera: ✅ Full support

### Performance API
- Long Task API: Chrome, Edge (limited support in other browsers)
- Memory API: Chrome with `--enable-precise-memory-info` flag
- Navigation Connection API: Chrome, Edge, Opera

### Fallbacks
All features gracefully degrade when not supported:
- WebGL → Canvas2D
- IntersectionObserver → Immediate loading
- SharedArrayBuffer → Regular arrays
- Performance APIs → Estimated values

## Monitoring & Debugging

### Performance Monitor
1. Open the app
2. Click the FPS indicator in the bottom-right corner
3. View real-time metrics and alerts
4. Click "Export" to save a report

### Console Debugging
```javascript
// Check GPU support
console.log(detectGPUSupport());

// Get memory info
console.log(getMemoryInfo());

// Worker pool status
console.log(pool.getStatus());

// Performance metrics
console.log(usePerformanceStore.getState().metrics);
```

## Future Enhancements

Potential improvements for future phases:
- WebGPU support for advanced compute shaders
- OffscreenCanvas for background rendering
- WASM integration for CPU-intensive computations
- Service Worker caching for performance data
- Performance regression testing in CI/CD
- Adaptive quality based on device capabilities
- Frame budget enforcement
- React Profiler integration

## References

- [Web Workers API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API)
- [WebGL Fundamentals](https://webglfundamentals.org/)
- [Intersection Observer API](https://developer.mozilla.org/en-US/docs/Web/API/Intersection_Observer_API)
- [Performance API](https://developer.mozilla.org/en-US/docs/Web/API/Performance_API)
- [react-window](https://github.com/bvaughn/react-window)

## Acceptance Criteria

- ✅ Charts leverage GPU acceleration with toggle and gracefully degrade when unsupported
- ✅ Large lists use virtual scrolling; lazy loading and skeletons improve perceived performance
- ✅ Heavy computations execute in web workers with cancellable jobs and status UI
- ✅ Performance monitor overlay shows metrics with logging/export; alerts trigger when thresholds exceeded
- ✅ Tests validate worker computations and performance metrics accuracy
