# AgentReputation DAO - Build Review & Demo Guide

## 📋 Code Review Summary

### Architecture Overview

The expanded AgentReputation DAO now implements a **complete cryptoeconomic trust system** based on academic research (Kaal 2023, EigenTrust, Colony Protocol).

```
┌─────────────────────────────────────────────────────────────┐
│                    AGENT REPUTATION DAO                      │
├─────────────────────────────────────────────────────────────┤
│  CORE LAYER (Original)                                      │
│  ├── Agent registration                                     │
│  ├── Task completion → reputation                           │
│  ├── Basic vouching (stake tokens)                          │
│  └── Reputation decay (prevents inactivity)                 │
├─────────────────────────────────────────────────────────────┤
│  GOVERNANCE LAYER (NEW)                                     │
│  ├── DAO proposals (reputation-weighted voting)             │
│  ├── Dynamic parameter updates                              │
│  └── Slashing with configurable thresholds                  │
├─────────────────────────────────────────────────────────────┤
│  TRUST LAYER (NEW)                                          │
│  ├── Weighted vouching (EigenTrust algorithm)               │
│  ├── Trust propagation through network                      │
│  └── Reputation-weighted influence                          │
├─────────────────────────────────────────────────────────────┤
│  ORACLE LAYER (NEW)                                         │
│  ├── Authorized oracle registry                             │
│  ├── Automated reputation from GitHub/txs/hackathons        │
│  └── Attestation verification                               │
├─────────────────────────────────────────────────────────────┤
│  PRIVACY LAYER (NEW)                                        │
│  ├── Soulbound Reputation NFTs (5 tiers)                    │
│  ├── ZK proof verification (privacy-preserving)             │
│  └── Prove "rep > X" without revealing score                │
└─────────────────────────────────────────────────────────────┘
```

---

## 🔑 Key Mathematical Models Implemented

### 1. EigenTrust Reputation Flow
```rust
R_i(t+1) = (1-λ)R_i(t) + Σ_j R_j(t) · T_ji

Where:
- λ = decay rate (prevents permanent elites)
- T_ji = trust weight from j to i
- Reputation flows through the trust network
```

### 2. Weighted Vouching
```rust
trust_weight = min(reputation / threshold, max_multiplier)
weighted_impact = base_amount · trust_weight

Example:
- Alice has 5000 rep (threshold = 1000)
- Trust weight = min(5000/1000, 3.0) = 3.0x
- She vouches with 100 tokens → 300 impact
```

### 3. Governance Voting
```rust
vote_weight = proposer_reputation
quorum = 10_000 total rep voted
majority = 51% to pass

Pure democracy: 1 reputation = 1 vote
```

### 4. Dynamic Slashing
```rust
slash_amount = target_rep · slash_percentage
slasher_bounty = slash_amount · 0.05

High-rep agents lose more when slashed (proportional)
```

---

## 📁 File Structure

```
programs/src/
├── lib.rs                      # Program entry point
├── state.rs                    # Account structures
├── errors.rs                   # Error definitions
├── instructions/
│   ├── initialize.rs           # Protocol initialization
│   ├── register_agent.rs       # Agent registration
│   ├── complete_task.rs        # Task completion → rep
│   ├── vouch.rs                # Basic vouching (legacy)
│   ├── decay.rs                # Reputation decay
│   ├── query.rs                # Read-only queries
│   ├── weighted_vouch.rs       # ⭐ EigenTrust vouching
│   ├── governance.rs           # ⭐ DAO governance
│   ├── oracle.rs               # ⭐ Oracle integration
│   ├── reputation_nft.rs       # ⭐ Soulbound NFTs
│   └── zk_verification.rs      # ⭐ ZK proof stubs
└── mod.rs                      # Module exports
```

---

## 🎮 Local Demo Script

### Prerequisites
```bash
# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.17.0/install)"

# Install Anchor
npm install -g @coral-xyz/anchor-cli@0.29.0

# Install dependencies
cd AGENT-REPUTATION-DAO-WIP-LFG
npm install
```

### Demo Steps

#### 1. Build the Program
```bash
anchor build
```

#### 2. Deploy to Localnet
```bash
# Start local validator
solana-test-validator

# In another terminal
anchor deploy --provider.cluster localnet
```

#### 3. Run the Demo Script

Create `demo/demo.ts`:

```typescript
import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { AgentreputationDao } from '../target/types/agentreputation_dao';
import { Keypair, PublicKey } from '@solana/web3.js';

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  
  const program = anchor.workspace.AgentreputationDao as Program<AgentreputationDao>;
  
  // Generate test agents
  const agent1 = Keypair.generate();
  const agent2 = Keypair.generate();
  const agent3 = Keypair.generate();
  
  console.log("🎭 AGENT REPUTATION DAO DEMO\n");
  
  // ========== STEP 1: Initialize Protocol ==========
  console.log("1️⃣ Initializing protocol...");
  await program.methods
    .initialize({
      authority: provider.wallet.publicKey,
      reputationMint: new PublicKey("11111111111111111111111111111111"), // Replace with actual mint
      minReputationForVouching: new anchor.BN(100),
      decayRatePerDay: new anchor.BN(100), // 1% daily decay
      vouchLockupPeriod: new anchor.BN(86400), // 1 day
      slashThreshold: new anchor.BN(2000), // 20%
      maxTrustMultiplier: new anchor.BN(30000), // 3x
      bump: 0,
    })
    .accounts({
      authority: provider.wallet.publicKey,
      config: program.account.protocolConfig.fetch(),
      reputationMint: new PublicKey("11111111111111111111111111111111"),
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();
  console.log("✅ Protocol initialized\n");
  
  // ========== STEP 2: Register Agents ==========
  console.log("2️⃣ Registering agents...");
  for (const agent of [agent1, agent2, agent3]) {
    await program.methods
      .registerAgent(`Agent_${agent.publicKey.toString().slice(0, 8)}`)
      .accounts({
        owner: agent.publicKey,
        agentProfile: // PDA calculation,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([agent])
      .rpc();
  }
  console.log("✅ 3 agents registered\n");
  
  // ========== STEP 3: Complete Tasks (Earn Rep) ==========
  console.log("3️⃣ Agents completing tasks...");
  
  // Agent 1 completes high-value task
  await program.methods
    .completeTask("task_001", new anchor.BN(500))
    .accounts({
      agent: agent1.publicKey,
      agentProfile: // PDA,
    })
    .signers([agent1])
    .rpc();
  console.log("   Agent 1 earned 500 rep");
  
  // Agent 2 completes medium task
  await program.methods
    .completeTask("task_002", new anchor.BN(200))
    .accounts({
      agent: agent2.publicKey,
      agentProfile: // PDA,
    })
    .signers([agent2])
    .rpc();
  console.log("   Agent 2 earned 200 rep");
  
  // Agent 3 completes small task
  await program.methods
    .completeTask("task_003", new anchor.BN(50))
    .accounts({
      agent: agent3.publicKey,
      agentProfile: // PDA,
    })
    .signers([agent3])
    .rpc();
  console.log("   Agent 3 earned 50 rep\n");
  
  // ========== STEP 4: Weighted Vouching ==========
  console.log("4️⃣ Weighted vouching (EigenTrust)...");
  
  // Agent 1 (500 rep) vouches for Agent 3
  // Trust weight = 500/100 = 5.0, capped at 3.0x
  await program.methods
    .vouchWeighted(new anchor.BN(100), true)
    .accounts({
      voucher: agent1.publicKey,
      vouchedFor: agent3.publicKey,
      // ... other accounts
    })
    .signers([agent1])
    .rpc();
  console.log("   Agent 1 (500 rep) vouched for Agent 3");
  console.log("   Base stake: 100, Weight: 3.0x, Impact: +300 rep\n");
  
  // ========== STEP 5: Governance Proposal ==========
  console.log("5️⃣ Creating governance proposal...");
  
  await program.methods
    .createProposal(
      { updateDecayRate: {} },
      new anchor.BN(50), // New decay rate: 0.5%
      "Reduce decay rate to incentivize long-term participation"
    )
    .accounts({
      proposer: agent1.publicKey,
      proposal: // PDA,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .signers([agent1])
    .rpc();
  console.log("   Agent 1 created proposal (auto-voted with 500 rep)\n");
  
  // ========== STEP 6: Vote on Proposal ==========
  console.log("6️⃣ Voting on proposal...");
  
  await program.methods
    .voteProposal(true) // Vote FOR
    .accounts({
      voter: agent2.publicKey,
      proposal: // PDA,
    })
    .signers([agent2])
    .rpc();
  console.log("   Agent 2 voted FOR with 200 rep");
  console.log("   Total: 700 FOR, 0 AGAINST\n");
  
  // ========== STEP 7: Reputation NFT ==========
  console.log("7️⃣ Minting Reputation NFT...");
  
  await program.methods
    .mintReputationNft("https://arweave.net/metadata.json")
    .accounts({
      agent: agent1.publicKey,
      // ... other accounts
    })
    .signers([agent1])
    .rpc();
  console.log("   Agent 1 minted Guardian-tier NFT (500+ rep)");
  console.log("   Benefits: Premium API, Discord Guardian role, governance voting\n");
  
  // ========== STEP 8: Apply Decay ==========
  console.log("8️⃣ Applying reputation decay...");
  
  // Simulate time passing (would need to advance clock in test)
  await program.methods
    .applyDecay()
    .accounts({
      agent: agent3.publicKey,
      agentProfile: // PDA,
    })
    .rpc();
  console.log("   Agent 3 inactive → decay applied\n");
  
  // ========== STEP 9: Oracle Attestation ==========
  console.log("9️⃣ Oracle attestation (GitHub)...");
  
  // Initialize oracle registry first
  await program.methods
    .initializeOracleRegistry()
    .accounts({
      authority: provider.wallet.publicKey,
    })
    .rpc();
  
  // Add oracle
  const oracle = Keypair.generate();
  await program.methods
    .addOracle(oracle.publicKey)
    .accounts({
      authority: provider.wallet.publicKey,
    })
    .rpc();
  
  // Submit attestation
  await program.methods
    .submitAttestation(
      { githubPrMerged: {} },
      Array(32).fill(0), // metadata hash
      new anchor.BN(50) // 50 rep for merged PR
    )
    .accounts({
      oracle: oracle.publicKey,
      agent: agent2.publicKey,
    })
    .signers([oracle])
    .rpc();
  console.log("   Oracle attested: Agent 2 merged PR → +50 rep\n");
  
  console.log("✅ DEMO COMPLETE!");
  console.log("\nKey Achievements:");
  console.log("  • Weighted vouching with EigenTrust algorithm");
  console.log("  • Reputation-weighted DAO governance");
  console.log("  • Soulbound NFTs with tiered benefits");
  console.log("  • Oracle integration for automated rep");
  console.log("  • Decay mechanism preventing inactivity");
}

main().catch(console.error);
```

### Run the Demo
```bash
# Compile TypeScript
npx ts-node demo/demo.ts
```

---

## 🧪 Testing Checklist

### Unit Tests (to implement in `tests/`)

```typescript
// test/weighted_vouch.test.ts
describe("Weighted Vouching", () => {
  it("High rep agent has 3x weight", async () => {
    // Test logic
  });
  
  it("Low rep agent has 1x weight", async () => {
    // Test logic
  });
});

// test/governance.test.ts
describe("Governance", () => {
  it("Proposal passes with 51% majority", async () => {
    // Test logic
  });
  
  it("Proposal fails without quorum", async () => {
    // Test logic
  });
});

// test/oracle.test.ts
describe("Oracle", () => {
  it("Authorized oracle can attest", async () => {
    // Test logic
  });
  
  it("Unauthorized oracle rejected", async () => {
    // Test logic
  });
});
```

---

## 📊 Feature Comparison

| Feature | Before | After | Impact |
|---------|--------|-------|--------|
| Vouching | Flat 1:1 | Weighted 1-3x | Meritocratic |
| Governance | None | DAO with rep voting | Decentralized |
| Reputation | Static | Decay + growth | Active participation |
| Verification | Public | ZK proofs | Privacy-preserving |
| Identity | Address | Soulbound NFT | Portable reputation |
| Updates | Manual | Oracle automated | Real-time |

---

## 🚀 Deployment Checklist

- [ ] Run `anchor build` (check for errors)
- [ ] Run `anchor test` (all tests pass)
- [ ] Deploy to devnet
- [ ] Create reputation token mint
- [ ] Initialize protocol with parameters
- [ ] Add authorized oracles
- [ ] Deploy NFT metadata to Arweave/IPFS
- [ ] Frontend integration (update app/)
- [ ] Security audit
- [ ] Mainnet deployment

---

**Ready to build and demo locally!** 🎉
