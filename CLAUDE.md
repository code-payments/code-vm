# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Code VM is a Solana blockchain program written in Rust, purpose-built for payments. It reduces transaction fees by up to 95% and account rent by 80% by operating on compressed virtual account representations. The VM is designed specifically for the Flipcash mobile payment app and is currently running on Solana mainnet (audited by OtterSec).

## Build and Test Commands

```bash
# Build the Solana BPF program
cargo build-sbf

# Run on-chain integration tests
cargo test-sbf -- --nocapture

# Generate IDL (requires Anchor, Bun.js, npm dependencies)
make idl
```

**Required versions:**
- rustc 1.76.0
- solana-cli 1.18.9

## Architecture

The codebase is organized as a Cargo workspace with two main crates:

### `program/` - On-chain Solana Program
- `src/lib.rs` - Program entrypoint, routes to instruction handlers
- `src/instruction/` - 12+ instruction handlers (init_vm, exec, compress, deposit, withdraw, etc.)
- `src/opcode/` - 9 opcodes executed via ExecIx (transfer, withdraw, relay, airdrop, etc.)
- `tests/` - Integration tests (21 test files)

### `api/` - Shared API & SDK Library
- `src/instruction.rs` - Instruction definitions and parsing
- `src/opcode.rs` - Opcode definitions and parsing
- `src/state.rs` - Account state type exports
- `src/sdk.rs` - Client SDK helpers (off-chain only)
- `src/consts.rs` - Constants (PDA seeds, limits)
- `src/cvm/state/` - Account state structs (Memory, Relay, Storage, etc.)
- `src/cvm/account/` - Virtual account implementations
- `src/types/` - Primitives (Hash, Signature, MerkleTree, SliceAllocator)

### `idl/` - IDL Generation
- Uses a dummy Anchor program to generate `code_vm.json`
- Contains ImHex patterns for binary inspection (`*.hexpat`)

## Key Concepts

**Instruction Flow:** Transaction → `process_instruction()` → CodeInstruction enum dispatch → Handler → Opcode/state mutation

**Account Types:** CodeVmAccount, MemoryAccount, StorageAccount, RelayAccount, UnlockStateAccount, WithdrawReceiptAccount

**Memory Banks:** Supports up to 4 memory banks (A, B, C, D) per transaction for virtual account storage

**Compression:** Accounts can be compressed off-chain with Merkle proofs and decompressed on-demand

**Binary Efficiency:** All instruction data uses tight struct packing (Pod/Zeroable traits from bytemuck)

## Dependencies

Key crates:
- `steel 1.3` - Solana instruction framework
- `spl-token 4.x` - SPL token operations
- `curve25519-dalek 4.1.3` - Cryptography
- `borsh 0.10.3` - Serialization
