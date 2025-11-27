#!/bin/bash

echo "=== Eclipse Market Pro Quick Diagnostic ==="
echo ""

echo "1. Checking Rust/Cargo setup..."
if command -v cargo &> /dev/null; then
    echo "✓ Cargo found: $(cargo --version)"
else
    echo "✗ Cargo not found"
    exit 1
fi

echo ""
echo "2. Checking Node.js/npm setup..."
if command -v node &> /dev/null; then
    echo "✓ Node.js found: $(node --version)"
else
    echo "✗ Node.js not found"
    exit 1
fi

if command -v npm &> /dev/null; then
    echo "✓ npm found: $(npm --version)"
else
    echo "✗ npm not found"
    exit 1
fi

echo ""
echo "3. Testing frontend build..."
npm run build
if [ $? -eq 0 ]; then
    echo "✓ Frontend builds successfully"
else
    echo "✗ Frontend build failed"
    exit 1
fi

echo ""
echo "4. Testing Rust backend build..."
cd src-tauri
cargo check
if [ $? -eq 0 ]; then
    echo "✓ Rust backend compiles successfully"
else
    echo "✗ Rust backend compilation failed"
    cd ..
    exit 1
fi
cd ..

echo ""
echo "5. Checking for common issues..."
echo ""

# Check if dist folder exists
if [ -d "dist" ]; then
    echo "✓ Frontend dist folder exists"
    echo "  Contents:"
    ls -la dist/ | head -10
else
    echo "✗ Frontend dist folder missing"
fi

echo ""
echo "6. Testing Tauri CLI..."
npx tauri --version
if [ $? -eq 0 ]; then
    echo "✓ Tauri CLI available"
else
    echo "✗ Tauri CLI not available"
fi

echo ""
echo "=== Quick Diagnostic Complete ==="
echo ""
echo "If all checks pass but the app still crashes:"
echo "1. Run: npm run tauri dev"
echo "2. Check browser console for JavaScript errors"
echo "3. Check terminal for Rust errors"
echo "4. Try building with: npm run tauri build"
echo ""
echo "For detailed debugging, run the enhanced startup logging version."