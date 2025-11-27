# App Freeze Diagnosis Plan

## Quick Start - Run Test Version

The test version logs every module as it loads to pinpoint the freeze source.

### To Run Test Mode:
```bash
# Start dev server with test entry point
npm run dev -- --port 1420 --open index-test.html
```

Or manually navigate to: `http://localhost:1420/?#test`

## Phase 1: Identify Freeze Location (CURRENT)

The test version (`src/main.test.tsx` + `src/App.test.tsx`) will show:
- Module load order
- Load times for each module
- Which modules are still loading when app freezes
- Error messages if any

**What to watch for in console:**
```
[MODULE] → Loading: ModuleName
[MODULE] ✓ Loaded: ModuleName (123.45ms)
[MODULE] ✗ Failed: ModuleName - ERROR: message
```

**If app freezes:**
- Look for `→ Loading:` messages without corresponding `✓ Loaded:` 
- Check "STILL LOADING" section in error report
- That module is the culprit

## Phase 2: Enable Components One-by-One

Once we identify the problematic module from Phase 1, enable related components systematically:

### Test Checklist:

- [ ] **Test 1:** Minimal app (current test version)
  - File: `src/App.test.tsx` 
  - Components: Only ClientLayout + Sidebar
  - Expected: Loads successfully
  
- [ ] **Test 2:** Add Settings page route
  - Change `src/App.test.tsx` line 32: Add Settings route
  - Expected: Loads successfully if no heavy imports in Settings
  
- [ ] **Test 3:** Add Dashboard page route (already there)
  - Observe: Does Dashboard load?
  
- [ ] **Test 4:** Re-enable walletStore
  - Edit: `src/store/index.ts` line 3
  - Uncomment: `export * from './walletStore';`
  - Expected: Should load (no module-level Tauri calls)
  
- [ ] **Test 5:** Re-enable tradingStore
  - Edit: `src/store/index.ts` line 4
  - Uncomment: `export * from './tradingStore';`
  - Expected: Should load
  
- [ ] **Test 6:** Re-enable portfolioStore
  - Edit: `src/store/index.ts` line 5
  - Uncomment: `export * from './portfolioStore';`
  - Expected: Should load
  
- [ ] **Test 7:** Re-enable aiStore
  - Edit: `src/store/index.ts` line 6
  - Uncomment: `export * from './aiStore';`
  - Expected: Should load

- [ ] **Test 8:** Add more routes to `src/App.test.tsx`
  - Add: Portfolio, Trading routes
  - Expected: Check if specific pages cause freeze

- [ ] **Test 9:** Re-enable useTradingEventBridge hook
  - Edit: `src/layouts/ClientLayout.tsx` line 4
  - Uncomment: `import { useTradingEventBridge } from '@/hooks/useTradingEventBridge';`
  - Uncomment: `useTradingEventBridge();` (line 13)
  - Expected: Observe if this causes freeze

- [ ] **Test 10:** Switch to full App.tsx
  - Edit: `src/main.test.tsx` line 26
  - Change: `import('./App.test')` → `import('./App')`
  - Expected: Full app loads or specific page causes freeze

## How to Use Results

### If specific component freezes app:
1. Note which component
2. Check its imports
3. Look for:
   - Synchronous Tauri imports
   - Module-level API calls
   - Circular dependencies
   - Heavy library initialization

### If specific store freezes app:
1. Check persist middleware usage
2. Check storage initialization
3. Verify no Tauri imports at module level
4. Look for circular imports

### If specific page freezes app:
1. Strip its imports one-by-one
2. Find which import causes freeze
3. Move heavy imports to lazy loading or dynamic imports

## Expected Module Load Report Format

```
=== MODULE LOAD REPORT ===

1234.56ms [→] Importing AppTest
2345.67ms [✓] Importing AppTest (1111.11ms)
3456.78ms [→] Dashboard
... (more modules)

=== SUMMARY ===
Total modules loaded: 15
Failed modules: 0
Total time: 2345.67ms
Slowest module: Portfolio (456.78ms)

=== STILL LOADING (POTENTIAL DEADLOCK) ===
⏳ SomeModule (1234.56ms)
```

The "STILL LOADING" section appears if modules don't finish loading - that's where the freeze is.

## Files Modified for Testing

- ✅ `src/utils/moduleLogger.ts` - New logging utility
- ✅ `src/main.test.tsx` - Test entry point with logging
- ✅ `src/App.test.tsx` - Minimal app version
- ✅ `index-test.html` - Test HTML entry point

## To Return to Normal

Simply run `npm run dev` normally (uses `src/main.tsx` and `src/App.tsx`)

## Key Questions to Answer

1. **Does app load with test version?** → Narrows down the problem to specific modules
2. **Which stores cause freeze?** → Zustand/persist issue or store structure issue
3. **Which components cause freeze?** → Sidebar, Layout, Providers?
4. **Which pages cause freeze?** → Heavy pages need lazy loading or optimization
5. **Is it circular imports?** → Check dependency graph

Once you run the test and share console output, we can pinpoint exact culprit.
