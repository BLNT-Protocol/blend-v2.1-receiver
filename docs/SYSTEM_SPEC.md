# Blend V2.1 receiver system specification

Normative terms `MUST`, `MUST NOT`, `SHOULD`, and `MAY` are acceptance criteria.

## Purpose and scope

The receiver MUST provide a single-purpose, temporary bridge between the incumbent Blend V2 backstop and the designated Blend V2.1 backstop. It MUST allow legacy BLND:USDC Comet V1 LP supplied by the YieldBlox Security Council (YSC) to establish the first emitter-swap balance condition, return that legacy LP to an immutable YSC recipient after the first swap, and immediately queue the second swap to the immutable V2.1 backstop.

The receiver MUST NOT implement governance, a treasury, an administrator, an upgrade authority, an arbitrary recipient, an arbitrary contract call, or a generic token sweep. It MUST NOT modify the emitter or either Blend backstop contract.

## Immutable construction

Construction MUST bind:

- the unchanged Blend V1 emitter;
- the emitter's incumbent V2 backstop at construction;
- the legacy BLND:USDC Comet V1 LP token used by that incumbent backstop;
- the designated V2.1 backstop;
- the BLND:USDC Comet V1.1 LP token used by V2.1;
- the existing BLND Stellar Asset Contract; and
- the YSC-controlled recipient to which legacy LP is returned.

The legacy token, V2.1 token, and BLND token MUST be distinct seven-decimal tokens. The BLND token MUST be a Stellar Asset Contract. The emitter, token, and V2.1 backstop addresses MUST be executable. The incumbent backstop, receiver, and V2.1 backstop MUST be distinct. Configuration MUST be immutable after construction.

## First-hop queue

Before the first hop is queued, YSC MUST transfer enough legacy V2 LP to the receiver for its raw token balance to be strictly greater than the incumbent V2 backstop's raw balance.

`queue_receiver()` MUST be permissionless. It MUST require the emitter still to target the incumbent backstop recorded at construction and require the emitter queue to be empty. It MUST call:

`emitter.queue_swap_backstop(receiver, v2_1_backstop_token)`

The V2.1 Comet V1.1 LP MUST therefore become the emitter's comparison token when the first hop finalizes. The receiver MUST NOT permit callers to substitute another swap token or candidate address.

The unchanged emitter enforces its 31-day delay and the legacy-LP balance comparison. The receiver MUST NOT bypass either requirement.

## Atomic advance and legacy-LP return

`advance_to_v2_1()` SHOULD be used to finalize the first hop. It MUST:

1. verify that the emitter queue names the receiver and the immutable V2.1 token;
2. invoke the emitter's unchanged `swap_backstop()`;
3. verify that the receiver became the current emitter backstop;
4. transfer any V2.1 LP held by the receiver to the immutable YSC recipient so it cannot obstruct the next comparison;
5. queue the immutable V2.1 backstop with the immutable V2.1 token, unless that exact swap is already queued; and
6. return the receiver's complete legacy-LP balance to the immutable YSC recipient.

These operations MUST be atomic. Failure to finalize the first hop, queue the exact V2.1 candidate, or complete an exact-balance transfer MUST roll back the emitter swap and every token movement in the invocation.

If a caller invokes the permissionless emitter swap directly, `advance_to_v2_1()` MAY continue from the state in which the receiver is already current. `return_legacy()` MUST provide an idempotent permissionless fallback that sends legacy LP only to the immutable YSC recipient and only after the emitter targets either the receiver or V2.1.

## Second-hop finalization and BLND

The unchanged emitter imposes a second 31-day delay before V2.1 can replace the receiver. `finalize_v2_1()` MUST validate that the queued candidate and token exactly match the immutable V2.1 configuration before invoking `emitter.swap_backstop()`.

The receiver MUST NOT call `emitter.drop` and MUST NOT request, create, or retain any one-time BLND allocation. No BLND drop or additional migration issuance is permitted.

The unchanged emitter necessarily mints normal one-BLND-per-second emissions to its current backstop immediately before completing a backstop swap. Consequently, second-hop finalization necessarily mints the receiver's normal emissions accrued during its temporary tenure. `finalize_v2_1()` MUST burn the receiver's entire BLND balance in the same atomic invocation after the emitter changes to V2.1. `burn_received_blnd()` MUST provide a permissionless fallback for BLND minted by direct emitter calls. The receiver MUST never forward those emissions to YSC, V2.1, a caller, or any treasury.

Thus the receiver creates no net BLND allocation, although the unchanged emitter's normal mint-and-burn events remain observable. Literal prevention of that intermediate mint is impossible without changing the V1 emitter.

## Permissionless operation and safety

Every lifecycle entrypoint MUST be permissionless and deterministic. Permissionless execution MUST NOT imply caller discretion: all candidate, token, and recipient addresses MUST come from immutable configuration.

Before invoking an emitter swap, the receiver MUST verify the complete queued `new_backstop` and `new_backstop_token` values. It MUST reject unexpected current-backstop or queued-swap states. Transfers and burns MUST verify exact post-call balances. Reentrant lifecycle calls MUST fail, and any failure MUST roll back the complete Soroban invocation.

The contract MUST expose no method capable of transferring legacy LP, V2.1 LP, or BLND to a caller-selected address. Assets other than the three configured tokens are outside the contract's accepted custody model and MUST NOT gain a recovery path.

## Operational limitations

The V1 emitter's own `queue_swap_backstop`, `cancel_swap_backstop`, and `swap_backstop` methods remain permissionless. The receiver cannot prevent a third party from queueing another balance-qualified address while the emitter queue is empty or from invoking an already-unlocked emitter swap directly. Operators SHOULD use the receiver's atomic lifecycle methods promptly and MUST verify the emitter queue immediately before every transition.

The second hop succeeds only while the V2.1 backstop's V2.1-LP balance remains strictly greater than the receiver's balance of that token. The receiver sweeps its own V2.1-LP balance before queueing and finalization, but it cannot prevent unsolicited token transfers between invocations.

## Public API

- `__constructor(emitter, legacy_backstop_token, v2_1_backstop, v2_1_backstop_token, blnd_token, ysc_recipient)`
- `queue_receiver()`
- `advance_to_v2_1() -> i128`
- `finalize_v2_1() -> i128`
- `return_legacy() -> i128`
- `burn_received_blnd() -> i128`
- `get_config() -> Config`
