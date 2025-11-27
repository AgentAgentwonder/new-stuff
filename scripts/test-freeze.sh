#!/bin/bash

# Test Freeze Diagnosis Script
# Runs the app in test mode with module load logging

echo "🧪 Starting Eclipse Market Pro in TEST MODE..."
echo ""
echo "📋 Module load logs will appear in browser console"
echo "📊 Detailed report will print after 1 second"
echo ""
echo "⚠️  If app freezes, check console for last 'Loading:' message"
echo ""
echo "Starting dev server..."
echo ""

npm run dev -- --open index-test.html
