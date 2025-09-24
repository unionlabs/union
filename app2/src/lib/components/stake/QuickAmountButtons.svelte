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

<div class="flex gap-2">
  <button
    type="button"
    onclick={() => selectPercentage(25)}
    disabled={disabled || O.isNone(balance)}
    class="px-3 py-1.5 text-xs font-medium rounded-md bg-zinc-800 hover:bg-zinc-700 text-zinc-300 hover:text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
  >
    25%
  </button>
  <button
    type="button"
    onclick={() => selectPercentage(50)}
    disabled={disabled || O.isNone(balance)}
    class="px-3 py-1.5 text-xs font-medium rounded-md bg-zinc-800 hover:bg-zinc-700 text-zinc-300 hover:text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
  >
    50%
  </button>
  <button
    type="button"
    onclick={() => selectPercentage(75)}
    disabled={disabled || O.isNone(balance)}
    class="px-3 py-1.5 text-xs font-medium rounded-md bg-zinc-800 hover:bg-zinc-700 text-zinc-300 hover:text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
  >
    75%
  </button>
  <button
    type="button"
    onclick={selectMax}
    disabled={disabled || O.isNone(balance)}
    class="px-3 py-1.5 text-xs font-medium rounded-md bg-zinc-800 hover:bg-zinc-700 text-zinc-300 hover:text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
  >
    MAX
  </button>
</div>
