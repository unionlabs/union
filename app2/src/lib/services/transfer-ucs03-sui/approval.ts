import type { ValidTransfer } from "@unionlabs/sdk/schema"
import { Effect } from "effect"

/**
 * Sui coins don’t use approve/allowance.
 * We short-circuit with a sentinel message so callers can skip receipt waits.
 */
export const approveTransfer = (_params: ValidTransfer["args"]) =>
  Effect.succeed("sui-no-approval-needed" as const)
