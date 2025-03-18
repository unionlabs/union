<script lang="ts">
import Input from "$lib/components/ui/Input.svelte"
import { transfer } from "$lib/components/Transfer/transfer.svelte.ts"
import { Option } from "effect"
import { balancesStore, createKey, type BalanceKey } from "$lib/stores/balances.svelte.ts"
import { wallets } from "$lib/stores/wallets.svelte.ts"
import { formatUnits } from "viem"
import Skeleton from "$lib/components/ui/Skeleton.svelte"

let sourceChain = $derived(
  Option.isSome(transfer.sourceChain) ? Option.getOrNull(transfer.sourceChain) : null
)
let baseToken = $derived(
  Option.isSome(transfer.baseToken) ? Option.getOrNull(transfer.baseToken) : null
)
let maxEnabled = $derived(!!sourceChain && !!baseToken && !!transfer.raw.asset)

let balanceState = $state({
  isLoading: false,
  amount: BigInt("0")
})

let balanceKey = $state<Option.Option<BalanceKey>>(Option.none())

$effect(() => {
  if (!(sourceChain && baseToken)) return

  const addressOption = wallets.getAddressForChain(sourceChain)
  if (!Option.isSome(addressOption)) return

  const address = addressOption.value

  const newBalanceKey = createKey(sourceChain.universal_chain_id, address, baseToken.denom)

  if (Option.isNone(balanceKey) || balanceKey.value !== newBalanceKey) {
    balanceState.isLoading = true
    balancesStore.fetchBalance(sourceChain, address, baseToken.denom)
    balanceKey = Option.some(newBalanceKey)
  }

  const balance = balancesStore.getBalance(sourceChain.universal_chain_id, address, baseToken.denom)

  balanceState.isLoading = false

  if (Option.isSome(balance)) {
    balanceState.amount = balance.value
  } else {
    balanceState.amount = BigInt("0")
  }
})

let displayBalance = $derived.by(() => {
  if (!baseToken) return "0.00"
  const decimals = baseToken.representations[0]?.decimals ?? 0
  return formatUnits(balanceState.amount, decimals)
})

function setMaxAmount() {
  if (!(sourceChain && baseToken && transfer.raw.asset) || balanceState.isLoading) return

  const maxDecimals = baseToken.representations[0]?.decimals ?? 0
  const formattedAmount = formatUnits(balanceState.amount, maxDecimals)

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
    {#if balanceState.isLoading}
      <Skeleton class="h-3 w-16 inline-block" />
    {:else}
      {displayBalance}
    {/if}
  </p>
  <button class="cursor-pointer hover:underline"
          onclick={setMaxAmount}
          disabled={!maxEnabled || balanceState.isLoading}
  >
    USE MAX
  </button>
</div>