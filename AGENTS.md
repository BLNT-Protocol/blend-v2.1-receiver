# Repository instructions

## Purpose

This repository contains one immutable Soroban receiver contract for the two-hop migration from the incumbent Blend V2 backstop to the designated Blend V2.1 backstop. Preserve that narrow purpose.

## Required invariants

- Do not add an administrator, governance mechanism, treasury, upgrade authority, arbitrary call facility, arbitrary recipient, or generic token-recovery method.
- Keep the emitter, incumbent token, V2.1 backstop, V2.1 token, BLND token, and YSC recipient immutable after construction.
- The first emitter swap must select the V2.1 Comet V1.1 LP address as `new_backstop_token`.
- Legacy V2 LP may only leave the receiver for the immutable YSC recipient and only after the receiver has become the emitter backstop.
- Prefer `advance_to_v2_1` so first-hop finalization, second-hop queueing, and legacy-LP return remain atomic.
- Never invoke `emitter.drop`, mint BLND, or expose a path that allocates receiver-period BLND. Normal BLND forced by the unchanged emitter must be burned.
- Validate queued targets before invoking `emitter.swap_backstop`.
- Preserve exact token-balance checks around transfers and burns.
- Failed cross-contract calls must roll back the complete operation.

## Development

- Keep Rust, Soroban SDK, Stellar CLI, and action revisions pinned.
- Run `make test` and `make build` after contract changes.
- Update `docs/SYSTEM_SPEC.md` whenever behavior or assumptions change.
- Treat the system specification's normative requirements as acceptance criteria.
- Do not add deployment addresses, private keys, account configuration, or unreviewed generated artifacts to source control.
