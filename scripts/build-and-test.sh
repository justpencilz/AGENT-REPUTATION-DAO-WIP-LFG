#!/bin/bash
# Local build and test script
# Prerequisites: solana-cli, anchor, node

echo "🔨 Building AgentReputation DAO v2.0..."

# Verify tools
echo "Checking prerequisites..."
command -v solana >/dev/null 2>&1 || { echo "❌ solana-cli required"; exit 1; }
command -v anchor >/dev/null 2>&1 || { echo "❌ anchor required"; exit 1; }
command -v npm >/dev/null 2>&1 || { echo "❌ npm required"; exit 1; }

echo "✅ Prerequisites met"
echo ""

# Install dependencies
echo "📦 Installing dependencies..."
npm install

# Generate TypeScript types from IDL
echo ""
echo "📝 Generating TypeScript types..."
anchor build 2>&1 | tee build.log

# Check for build errors
if grep -q "error" build.log; then
    echo ""
    echo "❌ Build failed. Check build.log"
    exit 1
fi

echo ""
echo "✅ Build successful!"
echo ""

# Run tests
echo "🧪 Running tests..."
anchor test 2>&1 | tee test.log

# Check test results
if grep -q "passing" test.log; then
    echo ""
    echo "✅ All tests passing!"
else
    echo ""
    echo "⚠️ Some tests may have failed. Check test.log"
fi

echo ""
echo "📊 Build Summary:"
echo "  Program ID: $(grep 'declare_id!' programs/src/lib.rs | head -1)"
echo "  Size: $(du -h target/deploy/*.so | cut -f1)"
echo ""
echo "Next: Run ./scripts/deploy-devnet.sh"
