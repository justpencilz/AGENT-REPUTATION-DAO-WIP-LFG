#!/bin/bash
# Quick push script - GitHub Live Test (Option 3)

echo "🚀 Pushing AgentReputation DAO v2.0 to GitHub..."
echo ""

cd /home/ubuntu/.openclaw/workspace/AGENT-REPUTATION-DAO-WIP-LFG

# Check status
echo "📊 Git status:"
git status --short
echo ""

# Add all changes
echo "➕ Adding files..."
git add -A

# Commit
echo "💾 Committing..."
git commit -m "feat: AgentReputation DAO v2.0 - weighted vouching, governance, oracle, NFTs, ZK proofs

Major features:
- Weighted vouching (EigenTrust, up to 3x multiplier)
- DAO governance (reputation-weighted voting)
- Oracle integration (GitHub, hackathons, bug bounties)
- Soulbound Reputation NFTs (5 tiers)
- ZK proof stubs (privacy-preserving)

Academic foundation:
- Kaal (2023) Reputation as Capital
- EigenTrust algorithm
- Colony Protocol model

Note: Fixed compile errors (String → fixed arrays, imports)
Test locally with: anchor build"

# Push
echo "⬆️ Pushing to GitHub..."
git push origin main

echo ""
echo "✅ PUSHED!"
echo ""
echo "🔗 https://github.com/justpencilz/AGENT-REPUTATION-DAO-WIP-LFG"
echo ""
echo "Next (Option 3 - Local):"
echo "  git clone https://github.com/justpencilz/AGENT-REPUTATION-DAO-WIP-LFG.git"
echo "  cd AGENT-REPUTATION-DAO-WIP-LFG"
echo "  anchor build"
echo "  cd app && npm install && npm run dev"
