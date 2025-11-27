# Test System for Diagnosing App Freeze 🔍

A minimal test version of the app that logs every module as it loads, to pinpoint which component is causing the freeze.

## 📁 Files Created

### Core Testing Files
```
src/
  utils/
    └─ moduleLogger.ts         ← Tracks module loads in real-time
  main.test.tsx                ← Test entry point with logging
  App.test.tsx                 ← Minimal app (Dashboard only)
index-test.html                ← Test HTML entry point
```

### Documentation
```
QUICK_START.md                 ← 2-min quick reference
FREEZE_DIAGNOSIS.md            ← Full diagnostic guide  
TEST_CHECKLIST.md              ← Step-by-step checklist
TESTING_GUIDE.txt              ← All info in ASCII
DIAGNOSTIC_SYSTEM_SUMMARY.md   ← Complete overview
README_TEST_SYSTEM.md          ← This file
```

### Helper Scripts
```
scripts/
  ├─ test-freeze.sh            ← Linux/Mac launcher
  └─ test-freeze.bat           ← Windows launcher
```

## 🚀 How to Use (30 seconds)

### 1. Start the test
```bash
npm run dev
```

### 2. Open in browser
```
http://localhost:1420/index-test.html
```

### 3. Open DevTools Console
```
F12 (Windows/Linux)
Cmd+Option+I (Mac)
```

### 4. Watch the logs
You'll see messages like:
```
[MODULE] → Loading: AccessibilityProvider
[MODULE] ✓ Loaded: AccessibilityProvider (45.23ms)
[MODULE] → Loading: ClientLayout
[MODULE] ✓ Loaded: ClientLayout (12.34ms)
...
```

### 5. If app freezes
Note the last message you see. It will look like:
```
[MODULE] → Loading: ProblematicModuleName
```

That module is causing the freeze. 

**Its name is your answer.**

## 📊 What You'll See

### If App Works ✅
```
[MODULE] ✓ Loaded: ClientLayout (12ms)
[MODULE] ✓ Loaded: Dashboard (89ms)
[MODULE] ✓ Loaded: App.test (234ms)

=== MODULE LOAD REPORT ===
Total modules loaded: 3
Failed modules: 0
Total time: 335ms
Slowest module: App.test (234ms)

App displays and works fine
```

### If App Freezes 🔴
```
[MODULE] → Loading: ProblematicModule
[App freezes here]
...after 5 seconds, timeout error:

=== MODULE LOAD REPORT ===
Total modules loaded: 2
Failed modules: 0
Total time: 5000+ms

=== STILL LOADING (POTENTIAL DEADLOCK) ===
⏳ ProblematicModule (5000ms)
```

### If Module Errors ❌
```
[MODULE] ✗ Failed: SomeModule - ERROR: Cannot find module
...
=== MODULE LOAD REPORT ===
Total modules loaded: 2
Failed modules: 1
...
```

## 🔧 How to Fix

### Once you know which module causes the freeze:

1. **Open that module file**
   - Look at the imports (top of file)
   - Look at initialization code (module-level code)

2. **Look for these patterns:**

   **Pattern A: Static Tauri Import**
   ```typescript
   ❌ BAD:
   import { invoke } from '@tauri-apps/api/core';
   
   ✅ FIX:
   // Move import inside function/hook:
   const { invoke } = await import('@tauri-apps/api/core');
   ```

   **Pattern B: Module-level Async Code**
   ```typescript
   ❌ BAD:
   const data = await someFunction();
   export const myVar = data;
   
   ✅ FIX:
   export async function getData() {
     return await someFunction();
   }
   ```

   **Pattern C: Heavy Library Import**
   ```typescript
   ❌ BAD:
   import * as recharts from 'recharts'; // imports entire lib
   
   ✅ FIX:
   // In a function or useEffect:
   const recharts = await import('recharts');
   ```

3. **Apply the fix**

4. **Re-run test to verify**
   ```bash
   npm run dev
   # Refresh browser
   ```

## 📋 Systematic Testing

If app loads in test version, stores may be the issue:

### Edit `src/store/index.ts` line by line:

```typescript
export * from './createBoundStore';
export * from './walletStore';        ← Uncomment this, test
export * from './tradingStore';       ← Uncomment this, test  
export * from './portfolioStore';     ← Uncomment this, test
export * from './aiStore';            ← Uncomment this, test
export * from './uiStore';
// ... rest of stores
```

For each one:
1. Uncomment ONE line
2. Save
3. Refresh test in browser
4. If freezes, you found the problem
5. If loads, comment it back and try next

### Edit `src/layouts/ClientLayout.tsx`:

```typescript
// Line 4: Uncomment
import { useTradingEventBridge } from '@/hooks/useTradingEventBridge';

// Line 13: Uncomment  
useTradingEventBridge();

// Test - if freezes, this hook is the problem
```

## 📖 Full Documentation

For detailed information, see:
- **QUICK_START.md** - 2-minute summary
- **TEST_CHECKLIST.md** - Complete checklist with all options
- **FREEZE_DIAGNOSIS.md** - Full methodology
- **DIAGNOSTIC_SYSTEM_SUMMARY.md** - Technical overview

## 🎯 Most Likely Freeze Locations (Ranked)

Based on what you've already fixed:

1. **A Disabled Store** (walletStore, tradingStore, portfolioStore, aiStore)
   - Check: `src/store/index.ts` lines 3-6
   - Solution: Check store for static Tauri imports

2. **useTradingEventBridge Hook**
   - Check: `src/layouts/ClientLayout.tsx` lines 4, 13
   - Solution: Ensure Tauri code is in useEffect, not module-level

3. **Circular Import Between Modules**
   - Check: Which module leads to which
   - Solution: Use lazy imports or refactor dependencies

4. **Module-Level Code in Store or Hook**
   - Check: Any async code at module level
   - Solution: Wrap in functions or useEffect

5. **Heavy Component Import**
   - Check: Components importing large libraries
   - Solution: Use lazy/dynamic imports

## ✅ Success Checklist

- [ ] Test version runs without errors
- [ ] Console shows [MODULE] logs
- [ ] Report prints after 1 second
- [ ] Identified which module/store/hook causes freeze
- [ ] Applied appropriate fix
- [ ] Test runs again successfully
- [ ] Normal app (`npm run dev`) works

## 🆘 If Still Stuck

Share these details:
1. The module name from "STILL LOADING" section
2. That module's file path
3. The first 20 lines of that file
4. Any error message shown

## 📝 Next Steps

1. **Run test now** → `npm run dev` then open `http://localhost:1420/index-test.html`
2. **Note freeze location** → Look at last `[MODULE]` log message
3. **Follow TEST_CHECKLIST.md** → Systematic re-enabling of components
4. **Apply fix** → Based on module type
5. **Verify** → Re-run test
6. **Done!** → Return to normal `npm run dev`

---

**Time to solve this! Let's go! 🚀**
