#!/bin/bash

echo "=== Eclipse Market Pro Startup Crash Diagnosis ==="
echo ""

# Check if we're in the right directory
if [ ! -f "package.json" ]; then
    echo "Error: Please run this script from the project root directory"
    exit 1
fi

echo "1. Testing minimal app build..."
echo ""

# Test minimal app build
cd src-tauri-test
cargo build --release
if [ $? -eq 0 ]; then
    echo "✓ Minimal backend builds successfully"
else
    echo "✗ Minimal backend build failed"
    exit 1
fi

cd ..

echo ""
echo "2. Testing original app build..."
echo ""

# Test original app build
cd src-tauri
cargo build --release
if [ $? -eq 0 ]; then
    echo "✓ Original backend builds successfully"
else
    echo "✗ Original backend build failed"
    echo "This might be the root cause of the startup crash"
    exit 1
fi

cd ..

echo ""
echo "3. Checking frontend build..."
echo ""

npm run build
if [ $? -eq 0 ]; then
    echo "✓ Frontend builds successfully"
else
    echo "✗ Frontend build failed"
    exit 1
fi

echo ""
echo "4. Testing minimal app startup..."
echo ""

# Try to run the minimal app
cd src-tauri-test
cargo run --release &
MINIMAL_PID=$!
sleep 3

# Check if the process is still running
if kill -0 $MINIMAL_PID 2>/dev/null; then
    echo "✓ Minimal app started successfully"
    kill $MINIMAL_PID
    wait $MINIMAL_PID 2>/dev/null
else
    echo "✗ Minimal app failed to start"
fi

cd ..

echo ""
echo "5. Checking for common issues..."
echo ""

# Check for missing dependencies
echo "Checking Rust toolchain..."
rustc --version
cargo --version

echo ""
echo "Checking Node.js and npm..."
node --version
npm --version

echo ""
echo "Checking Tauri CLI..."
npx tauri --version

echo ""
echo "6. Memory and disk space check..."
echo "Available memory:"
free -h 2>/dev/null || echo "Memory check not available on this platform"

echo "Available disk space:"
df -h . 2>/dev/null || echo "Disk space check not available on this platform"

echo ""
echo "=== Diagnosis Complete ==="
echo ""
echo "If all tests passed but the app still crashes silently,"
echo "the issue might be:"
echo "1. A runtime error in React components"
echo "2. An issue with Tauri command registration"
echo "3. A problem with window creation"
echo "4. An issue with state initialization"
echo ""
echo "Next steps:"
echo "1. Run the minimal app to see if it works"
echo "2. Check browser console for JavaScript errors"
echo "3. Add more logging to isolate the failing component"
echo "4. Test with progressively more complex versions"