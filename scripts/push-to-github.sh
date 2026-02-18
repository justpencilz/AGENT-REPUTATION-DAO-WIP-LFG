#!/bin/bash
# Git push preparation script for AGENT-REPUTATION-DAO-WIP-LFG
# Run this locally to push changes to GitHub

echo "🚀 Preparing to push AgentReputation DAO v2.0..."

# Check if we're in the right directory
if [ ! -f "Anchor.toml" ]; then
    echo "❌ Error: Not in project root. Run from AGENT-REPUTATION-DAO-WIP-LFG/"
    exit 1
fi

# Check git status
echo "📊 Git status:"
git status

# Add all changes
echo ""
echo "➕ Adding files..."
git add programs/src/instructions/*.rs
git add programs/src/lib.rs
git add programs/src/state.rs
git add programs/src/errors.rs
git add programs/src/instructions/mod.rs
git add TASKS.md
git add demo/REVIEW_AND_DEMO.md

# Show what's being committed
echo ""
echo "📋 Files to commit:"
git diff --cached --stat

# Commit with descriptive message
echo ""
echo "💾 Committing..."
git commit -m "feat: AgentReputation DAO v2.0 - weighted vouching, governance, oracle, NFTs, ZK proofs

Major features implemented:
- Weighted vouching with EigenTrust algorithm (up to 3x multiplier)
- DAO governance with reputation-weighted voting
- Oracle integration for automated reputation (GitHub, hackathons, etc.)
- Soulbound Reputation NFTs (5 tiers: Novice → Legend)
- ZK proof verification stubs for privacy-preserving reputation
- Dynamic slashing with configurable thresholds
- Trust propagation through network graph

Academic foundation:
- Kaal (2023) 'Reputation as Capital'
- EigenTrust algorithm
- Colony Protocol model

Files added:
- instructions/weighted_vouch.rs (270 lines)
- instructions/governance.rs (280 lines)
- instructions/oracle.rs (190 lines)
- instructions/reputation_nft.rs (200 lines)
- instructions/zk_verification.rs (180 lines)
- demo/REVIEW_AND_DEMO.md (comprehensive guide)
- TASKS.md (project tracking)

Next: Cross-chain bridges, testing, documentation"

# Push to origin
echo ""
echo "⬆️ Pushing to GitHub..."
git push -u origin main

echo ""
echo "✅ Push complete!"
echo ""
echo "Verify at: https://github.com/justpencilz/AGENT-REPUTATION-DAO-WIP-LFG"
