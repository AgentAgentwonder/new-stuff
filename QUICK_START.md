# Quick Start: Freeze Diagnosis 🚀

## One Command to Test

```bash
npm run dev
# Then open in browser: http://localhost:1420/index-test.html
```

## What Happens

1. **Minimal app loads** with only Dashboard route
2. **Console shows module loads** in real-time as they happen
3. **After 1 second**, detailed load report appears
4. **If app freezes**, report shows which module is stuck

## What You'll See in Console

```
[MODULE] → Loading: ModuleName
[MODULE] ✓ Loaded: ModuleName (123.45ms)
[MODULE] → Loading: ProblematicModule
🔴 APP FREEZES HERE - MODULE NEVER FINISHES LOADING
```

## If App Freezes

1. Open DevTools Console (F12)
2. Look for last `[MODULE] → Loading:` message **without** `✓ Loaded:` after it
3. That's your culprit
4. Note the module name
5. Go to `TEST_CHECKLIST.md` Step 3b

## If App Loads Successfully ✅

1. All core components are fine
2. Problem is in disabled stores or hooks
3. Go to `TEST_CHECKLIST.md` Step 3a
4. Re-enable stores one-by-one

## Files Created for Testing

- **`src/utils/moduleLogger.ts`** - Module load tracking
- **`src/main.test.tsx`** - Test entry point
- **`src/App.test.tsx`** - Minimal app version
- **`index-test.html`** - Test HTML file
- **`FREEZE_DIAGNOSIS.md`** - Full diagnosis guide
- **`TEST_CHECKLIST.md`** - Step-by-step checklist
- **`QUICK_START.md`** - This file

## Common Freeze Locations

Based on diagnostics already done:

1. **In a Store** → RE-ENABLE STORES one-by-one in `src/store/index.ts`
2. **In useTradingEventBridge** → RE-ENABLE HOOK in `src/layouts/ClientLayout.tsx`
3. **In Sidebar Component** → Unlikely (already tested), but check imports
4. **In a Page Component** → Add to test routes one-by-one
5. **Circular Import** → Check module dependencies

## Next Steps

### If You Found the Problem:
1. Note the module/store/hook name
2. Check `TEST_CHECKLIST.md` for that component
3. Apply the fix listed there
4. Re-run test to verify

### If App Loads in Test Mode:
1. Follow `TEST_CHECKLIST.md` Step 3a
2. Enable stores one-by-one
3. Find which store causes freeze
4. Apply specific store fix

### If All Tests Pass:
1. Switch from test app to full app:
   - Edit: `src/main.test.tsx` line 26
   - Change `import('./App.test')` to `import('./App')`
2. Re-run test with full app
3. Identify which page route causes freeze
4. Add that page's imports to test version
5. Fix problematic imports in that page

## Pro Tips

- **Console is your friend** - Watch `[MODULE]` logs like a hawk
- **Slow modules OK** - A module taking 500ms is fine, as long as it finishes
- **Unfinished module = problem** - If `[MODULE] →` never gets `✓`, that's your freeze
- **One change at a time** - Only enable one store/component per test run
- **Clear cache if stuck** - `Ctrl+Shift+R` or `Cmd+Shift+R` to hard refresh

## Still Stuck?

If test version also freezes:
1. The problem is in core initialization (main.tsx, providers, or early imports)
2. Check for circular imports between modules
3. Check browser console for ANY errors (even before freeze)
4. Share the exact module name from "STILL LOADING" section

## Getting Back to Normal

Just run `npm run dev` normally - it uses the real app, not test version.

---

**Next:** Open your console and run the test! 🚀
