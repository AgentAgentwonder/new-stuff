# Freeze Diagnosis Test Checklist

## How to Use This Document

After running test mode, you'll get a module load report. Use this checklist to systematically narrow down which component/store/page is causing the freeze.

---

## STEP 1: Run Test Version (5 minutes)

```bash
npm run dev
# Open browser to: http://localhost:1420/index-test.html
```

**Expected Outcome:**
- App renders with only Dashboard route
- Console shows module load logs
- After 1 second, full load report prints to console

**Observation Points:**
- [ ] Does app load successfully?
- [ ] Are there any "STILL LOADING" messages?
- [ ] Which modules took longest to load?

---

## STEP 2: Check Test Results

### Result A: App Loads Successfully ✅
→ Problem is NOT in core initialization
→ Go to: [Add Stores (Step 3a)](#step-3a-add-stores)

### Result B: App Freezes 🔴
→ Look at console for "STILL LOADING" section
→ Identify which module doesn't finish loading
→ Go to: [Found Problem (Step 3b)](#step-3b-found-the-problem)

### Result C: Error in Module Load Report ❌
→ Check stack trace in error display
→ Identify which module failed
→ Note the error message
→ Go to: [Module Error (Step 3c)](#step-3c-module-failed-to-load)

---

## STEP 3a: Add Stores

If test version loaded successfully, stores may be the issue.

### 3a.1: Enable walletStore

**File:** `src/store/index.ts`
**Change:** Line 3 - Uncomment `export * from './walletStore';`

```typescript
// Before:
// export * from './walletStore';

// After:
export * from './walletStore';
```

**Test:** Run test again, observe result
- [ ] App still loads? 
- [ ] If YES → walletStore OK, continue to next
- [ ] If NO → walletStore causes freeze, see [Fix walletStore](#fix-walletstore)

### 3a.2: Enable tradingStore

**File:** `src/store/index.ts`
**Change:** Line 4 - Uncomment `export * from './tradingStore';`

**Test:** Run test again
- [ ] App still loads?
- [ ] If YES → tradingStore OK, continue
- [ ] If NO → tradingStore causes freeze, see [Fix tradingStore](#fix-tradingstore)

### 3a.3: Enable portfolioStore

**File:** `src/store/index.ts`
**Change:** Line 5 - Uncomment `export * from './portfolioStore';`

**Test:** Run test again
- [ ] App still loads?
- [ ] If YES → portfolioStore OK, continue
- [ ] If NO → portfolioStore causes freeze, see [Fix portfolioStore](#fix-portfoliostore)

### 3a.4: Enable aiStore

**File:** `src/store/index.ts`
**Change:** Line 6 - Uncomment `export * from './aiStore';`

**Test:** Run test again
- [ ] App still loads?
- [ ] If YES → all stores OK, go to [Step 3d](#step-3d-enable-trading-event-bridge)
- [ ] If NO → aiStore causes freeze, see [Fix aiStore](#fix-aistore)

---

## STEP 3b: Found the Problem

You've identified a "STILL LOADING" module that causes freeze.

**Module Name:** `_________________`

### Actions:
1. Open that module file
2. Look for:
   - [ ] Static `import` from `@tauri-apps` → Change to dynamic
   - [ ] Synchronous API calls at module level → Wrap in function
   - [ ] Heavy computation → Defer to lazy loading
   - [ ] Storage access → Move to no-op or lazy init
3. Apply fix (see relevant section below)
4. Re-run test

---

## STEP 3c: Module Failed to Load

**Error:** `_________________`
**Module:** `_________________`

### Diagnostics:
1. Check if module exists at that path
2. Look for syntax errors in that file
3. Check for circular imports
4. Verify all imports resolve correctly

**Fix Steps:**
1. Run `npm run build` to check for TypeScript errors
2. Check import paths use `@/` alias correctly
3. Verify module file is not corrupted
4. Try running original `src/main.tsx` to see if issue is specific to test

---

## STEP 3d: Enable Trading Event Bridge

If all stores loaded successfully:

**File:** `src/layouts/ClientLayout.tsx`
**Change:** 
- Line 4 - Uncomment `import { useTradingEventBridge } from '@/hooks/useTradingEventBridge';`
- Line 13 - Uncomment `useTradingEventBridge();`

**Test:** Run test again
- [ ] App still loads?
- [ ] If YES → useTradingEventBridge OK
- [ ] If NO → useTradingEventBridge causes freeze, see [Fix useTradingEventBridge](#fix-tradingeventbridge)

---

## STEP 4: Try Full App

If everything worked in test version, try the full app:

**File:** `src/main.test.tsx`
**Change:** Line 26

```typescript
// Before:
import('./App.test').then(...)

// After:
import('./App').then(...)
```

**Test:** Run test again with full app
- [ ] App loads with all routes?
- [ ] If YES → Issue is in which page? Keep adding routes one-by-one
- [ ] If NO → Check which page route causes freeze

---

## Fixes by Problem Type

### Fix: walletStore

If `walletStore` is causing freeze:

1. **Check file:** `src/store/walletStore.ts`
2. **Look for issues:**
   - [ ] Is there a static Tauri import? (Lines should use `await import()`)
   - [ ] Is there module-level initialization code?
   - [ ] Are there circular dependencies with other stores?

3. **Apply fix:** Move any blocking code into lazy/dynamic imports

### Fix: tradingStore

If `tradingStore` is causing freeze:

1. **Check file:** `src/store/tradingStore.ts`
2. **Look for same issues as walletStore**
3. **Additionally check:**
   - [ ] Are prices being fetched at import time?
   - [ ] Is there real-time subscription setup?

### Fix: portfolioStore

If `portfolioStore` is causing freeze:

1. **Check file:** `src/store/portfolioStore.ts`
2. **Look for:**
   - [ ] Heavy calculations in initial state?
   - [ ] API calls to fetch initial data?
   - [ ] Large data transformations?

### Fix: aiStore

If `aiStore` is causing freeze:

1. **Check file:** `src/store/aiStore.ts`
2. **Look for:**
   - [ ] LLM model initialization?
   - [ ] WebWorker setup?
   - [ ] Heavy imports of AI libraries?

### Fix: useTradingEventBridge

If `useTradingEventBridge` is causing freeze:

1. **Check file:** `src/hooks/useTradingEventBridge.ts`
2. **Look for:**
   - [ ] Is `useEffect` actually being called? (Should only run in component, not module-level)
   - [ ] Are there async operations in module scope?
   - [ ] Check if Tauri listener setup is in useEffect

3. **Verify structure:**
```typescript
// ✅ CORRECT:
export function useTradingEventBridge() {
  useEffect(() => {
    // All async/blocking code here
  }, []);
}

// ❌ WRONG:
// This would run at module load:
const unlisten = await listen(...);
```

---

## Common Patterns to Look For

### ❌ Pattern 1: Static Tauri Import
```typescript
// BAD - Runs at module load:
import { invoke } from '@tauri-apps/api/core';

// GOOD - Runs when needed:
const { invoke } = await import('@tauri-apps/api/core');
```

### ❌ Pattern 2: Module-Level Code
```typescript
// BAD:
const result = await someAsyncFunction();
export const myVar = result;

// GOOD:
export const getResult = async () => {
  return await someAsyncFunction();
};
```

### ❌ Pattern 3: Circular Imports
```typescript
// store/a.ts imports from store/b
// store/b imports from store/a
// This creates deadlock during initialization
```

### ❌ Pattern 4: Storage Access at Import
```typescript
// BAD - Synchronous I/O:
const data = localStorage.getItem('key');

// GOOD - Lazy/on-demand:
export const getData = () => localStorage.getItem('key');
```

---

## Testing Workflow Summary

```
Run Test Version
    ↓
    ├─→ Freezes? → Look at STILL LOADING → Fix that module
    ├─→ Error? → Check error message → Fix that module
    └─→ Loads? → Go to Step 3
         ↓
     Enable Stores One-by-One
         ↓
         ├─→ Store causes freeze? → Fix that store
         └─→ All stores OK? → Go to Step 3d
              ↓
           Enable Trading Bridge
              ↓
              ├─→ Freezes? → Fix hook
              └─→ OK? → Try Full App
                   ↓
                   ├─→ Freezes? → Check which page
                   └─→ Works! → PROBLEM SOLVED! 🎉
```

---

## Need Help?

- **Module load report unclear?** → Open browser DevTools Console (F12) and look for `[MODULE]` logs
- **Still stuck?** → Share:
  1. Module name from "STILL LOADING" section
  2. Full error message if any
  3. That module's import statements
  4. Browser console output
