<script lang="ts">
import * as O from "effect/Option"
import { Utils } from "@unionlabs/sdk"
import { BigDecimal } from "effect"

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

function selectPercentage(percentage: number) {
  if (O.isSome(balance)) {
    const amount = (balance.value * BigInt(percentage)) / 100n
    const amountDecimal = BigDecimal.make(amount, decimals)
    onAmountSelect(Utils.formatBigDecimal(amountDecimal), amount)
  }
}

function selectMax() {
  if (O.isSome(balance)) {
    const amountDecimal = BigDecimal.make(balance.value, decimals)
    onAmountSelect(Utils.formatBigDecimal(amountDecimal), balance.value)
  }
}
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