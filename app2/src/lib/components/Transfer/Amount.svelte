<script lang="ts">
import Input from "$lib/components/ui/Input.svelte"
import { transfer } from "$lib/components/Transfer/transfer.svelte.ts"
import { Option } from "effect"
import { formatUnits } from "viem"
import Skeleton from "$lib/components/ui/Skeleton.svelte"

let maxEnabled = $derived(
  Option.isSome(transfer.sourceChain) && Option.isSome(transfer.baseToken) && !!transfer.raw.asset
)

let isLoading = $derived(
  Option.isSome(transfer.baseToken) &&
    Option.isSome(transfer.sortedBalances) &&
    Option.isNone(transfer.baseTokenBalance)
)

let displayBalance = $derived.by(() => {
  if (Option.isNone(transfer.baseToken)) return "0.00"
  if (Option.isNone(transfer.baseTokenBalance)) return "0.00"

  const baseToken = transfer.baseToken.value
  const balance = transfer.baseTokenBalance.value

  if (Option.isNone(balance.balance)) return "0.00"

  const decimals = baseToken.representations[0]?.decimals ?? 0
  return formatUnits(BigInt(balance.balance.value), decimals)
})

function setMaxAmount() {
  if (!maxEnabled || isLoading) return
  if (Option.isNone(transfer.baseToken) || Option.isNone(transfer.baseTokenBalance)) return

  const balance = transfer.baseTokenBalance.value
  if (Option.isNone(balance.balance)) return

  const baseToken = transfer.baseToken.value
  const maxDecimals = baseToken.representations[0]?.decimals ?? 0
  const formattedAmount = formatUnits(BigInt(balance.balance.value), maxDecimals)

  transfer.raw.amount = formattedAmount

  const inputElement = document.getElementById("amount") as HTMLInputElement
  if (inputElement) {
    inputElement.value = formattedAmount
    const event = new Event("input", { bubbles: true })
    inputElement.dispatchEvent(event)
  }
}
</script>

<Input id="amount"
       label="amount"
       type="text"
       required={true}
       disabled={!transfer.raw.asset}
       autocorrect="off"
       placeholder="0.00"
       spellcheck="false"
       autocomplete="off"
       inputmode="decimal"
       data-field="amount"
       autocapitalize="none"
       pattern="^[0-9]*[.]?[0-9]*$"
       value={transfer.raw.amount}
       oninput={(event) => {
                const input = event.currentTarget;
                const value = input.value;
                const maxDecimals = Option.isSome(transfer.baseToken)
                  ? (transfer.baseToken.value.representations[0]?.decimals ?? 0)
                  : 0;

                if (value === '' || (/^\d*\.?\d*$/.test(value) &&
                  (value.includes('.')
                    ? value.split('.')[1].length <= maxDecimals
                    : true)
                )) {
                  transfer.raw.updateField('amount', event);
                } else {
                  input.value = transfer.raw.amount;
                }
              }}
       class="text-center"
/>
<div class="flex w-full justify-between text-xs">
  <p>
    BALANCE:
    {#if isLoading}
      <Skeleton class="h-3 w-16 inline-block" />
    {:else}
      {displayBalance}
    {/if}
  </p>
  <button class="cursor-pointer hover:underline"
          onclick={setMaxAmount}
          disabled={!maxEnabled || isLoading}
  >
    USE MAX
  </button>
</div>