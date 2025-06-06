import * as AppRuntime from "$lib/runtime"
import { transferData as TransferData } from "$lib/transfer/shared/data/transfer-data.svelte"
import type { RunPromiseExitResult } from "$lib/utils/effect.svelte"
import type { Fees } from "@unionlabs/sdk/schema/fee"
import { Array as A, BigDecimal, Effect, Hash, Option as O, pipe, Struct } from "effect"
import { constant } from "effect/Function"

const FEE_MULTIPLIER = BigDecimal.make(12n, 1) // Union hardcoded fee

/**
 * Store containing transfer fee data for a given channel.
 *
 * NOTE:
 * - Fees are optional to represent presence of base data.
 *
 * TOOD:
 * - How to represent loading? (only show loading for side-effecting)
 */
const createFeeStore = () => {
  let toasts = $state.raw<ReadonlyArray<any>>([])

  $effect.root(() => {
    $effect(() => {
      toasts = [...toasts, 0]
    })
  })

  const baseFees = $derived(pipe(
    TransferData.channel,
    O.map(Struct.get("fees")),
    O.map(Struct.evolve({
      PACKET_SEND: O.getOrElse(constant(0n))<bigint>,
      PACKET_RECV: O.getOrElse(constant(0n))<bigint>,
      PACKET_SEND_LC_UPDATE_L0: O.getOrElse(constant(0n))<bigint>,
      PACKET_SEND_LC_UPDATE_L1: O.getOrElse(constant(0n))<bigint>,
      PACKET_SEND_LC_UPDATE_L2: O.getOrElse(constant(0n))<bigint>,
    })),
  ))

  let a!: RunPromiseExitResult<number, never>

  $effect.root(() => {
    a = AppRuntime.runPromiseExit$(
      () => Effect.promise(async () => Hash.hash(baseFees)),
    )
  })

  return {
    get toasts() {
      return toasts
    },
    get a() {
      return a
    },
  } as const
}

export const FeeStore = createFeeStore()
