<script lang="ts">
import { Utils } from "@unionlabs/sdk"
import { BigDecimal, pipe } from "effect"
import * as O from "effect/Option"

interface Props {
  balance: O.Option<bigint>
  decimals: number
  onAmountSelect: (amount: string, wei: bigint) => void
  disabled?: boolean
}

let {
  balance,
  decimals,
  onAmountSelect,
  disabled = false,
}: Props = $props()

const selectPercentage = (percentage: number) =>
  pipe(
    balance,
    O.map(bal => {
      const amount = (bal * BigInt(percentage)) / 100n
      const amountDecimal = BigDecimal.make(amount, decimals)
      onAmountSelect(Utils.formatBigDecimal(amountDecimal), amount)
    }),
  )

const selectMax = () =>
  pipe(
    balance,
    O.map(bal => {
      const amountDecimal = BigDecimal.make(bal, decimals)
      onAmountSelect(Utils.formatBigDecimal(amountDecimal), bal)
    }),
  )
</script>

<div class="flex gap-1.5 text-[11px]">
  <button
    type="button"
    onclick={() => selectPercentage(25)}
    disabled={disabled || O.isNone(balance)}
    class="group inline-flex h-7 min-w-[48px] cursor-pointer items-center justify-center rounded border border-transparent bg-zinc-900 px-2 font-medium text-zinc-400 transition hover:border-zinc-700 hover:bg-zinc-800 hover:text-zinc-100 disabled:cursor-not-allowed disabled:opacity-40"
  >
    <span class="transition group-hover:scale-[1.03]">25%</span>
  </button>
  <button
    type="button"
    onclick={() => selectPercentage(50)}
    disabled={disabled || O.isNone(balance)}
    class="group inline-flex h-7 min-w-[48px] cursor-pointer items-center justify-center rounded border border-transparent bg-zinc-900 px-2 font-medium text-zinc-400 transition hover:border-zinc-700 hover:bg-zinc-800 hover:text-zinc-100 disabled:cursor-not-allowed disabled:opacity-40"
  >
    <span class="transition group-hover:scale-[1.03]">50%</span>
  </button>
  <button
    type="button"
    onclick={() => selectPercentage(75)}
    disabled={disabled || O.isNone(balance)}
    class="group inline-flex h-7 min-w-[48px] cursor-pointer items-center justify-center rounded border border-transparent bg-zinc-900 px-2 font-medium text-zinc-400 transition hover:border-zinc-700 hover:bg-zinc-800 hover:text-zinc-100 disabled:cursor-not-allowed disabled:opacity-40"
  >
    <span class="transition group-hover:scale-[1.03]">75%</span>
  </button>
  <button
    type="button"
    onclick={selectMax}
    disabled={disabled || O.isNone(balance)}
    class="group inline-flex h-7 min-w-[48px] cursor-pointer items-center justify-center rounded border border-transparent bg-zinc-900 px-2 font-semibold tracking-wide text-zinc-300 transition hover:border-accent/40 hover:bg-accent/10 hover:text-accent disabled:cursor-not-allowed disabled:opacity-40"
  >
    <span class="uppercase transition group-hover:scale-[1.05]">Max</span>
  </button>
</div>
