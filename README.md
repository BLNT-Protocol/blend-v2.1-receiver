# Blend V2.1 Receiver

This repository contains a purpose-built Soroban receiver for a two-hop migration from the incumbent Blend V2 backstop to Blend V2.1.

The receiver temporarily holds enough legacy BLND:USDC Comet V1 LP to qualify as the V1 emitter's next backstop. The first swap names the V2.1 BLND:USDC Comet V1.1 LP as the emitter's new comparison token. Once the receiver becomes current, it atomically queues the designated V2.1 backstop and returns all legacy LP to a fixed YSC recipient.

The contract has no administrator, governance, treasury, upgrade authority, arbitrary recipient, or generic recovery method. It never calls `emitter.drop`. Normal BLND emissions that the unchanged emitter necessarily mints during the receiver's temporary tenure are burned rather than allocated.

See [docs/SYSTEM_SPEC.md](docs/SYSTEM_SPEC.md) for the normative lifecycle and safety requirements.

## Build and test

Rust 1.91.1, Soroban SDK 27.0.3, and the `wasm32v1-none` target are pinned.

```bash
make test
make build
```
