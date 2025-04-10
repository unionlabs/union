<script lang="ts">
import Input from "$lib/components/ui/Input.svelte"
import { transfer } from "$lib/components/Transfer/transfer.svelte.ts"
import { Option } from "effect"
import { formatUnits } from "viem"
import { wallets } from "$lib/stores/wallets.svelte.ts"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import Label from "$lib/components/ui/Label.svelte"

type Props = {
  type: "source" | "destination"
  disabled?: boolean
}
let { type, disabled = false }: Props = $props()

let chainWallet = $derived.by(() => {
  if (Option.isSome(transfer.sourceChain)) {
    return wallets.getAddressForChain(transfer.sourceChain.value)
  }
  return Option.none()
})

function allDataReadyForBalance() {
  return (
    Option.isSome(transfer.sourceChain) &&
    Option.isSome(transfer.baseToken) &&
    Option.isSome(transfer.baseTokenBalance) &&
    Option.isSome(transfer.baseTokenBalance.value.balance)
  )
}

let displayBalance = $derived.by(() => {
  if (
    Option.isSome(transfer.sourceChain) &&
    Option.isSome(transfer.baseToken) &&
    Option.isSome(transfer.baseTokenBalance)
  ) {
    const bal = transfer.baseTokenBalance.value.balance
    if (Option.isSome(bal)) {
      const baseToken = transfer.baseToken.value
      const decimals = baseToken.representations[0]?.decimals ?? 0
      const raw = bal.value
      return formatUnits(BigInt(raw), decimals)
    }
  }
  return "" // Not loaded yet
})

function setMaxAmount() {
  if (Option.isNone(transfer.baseToken)) return
  if (Option.isNone(transfer.baseTokenBalance)) return
  if (Option.isNone(transfer.baseTokenBalance.value.balance)) return

  const baseToken = transfer.baseToken.value
  const decimals = baseToken.representations[0]?.decimals ?? 0

  const rawBalance = transfer.baseTokenBalance.value.balance.value
  const formattedAmount = formatUnits(BigInt(rawBalance), decimals)

  transfer.raw.amount = formattedAmount

  const inputElement = document.getElementById("amount") as HTMLInputElement
  if (inputElement) {
    inputElement.value = formattedAmount
    inputElement.dispatchEvent(new Event("input", { bubbles: true }))
  }
}
</script>


  <div class="w-full">
    {#if Option.isSome(chainWallet)}
    {#if type === "source"}
      <div class="flex w-full justify-between items-center text-xs gap-1 pb-1">
        <div class="flex gap-1">
          <Label>BALANCE:</Label>
          {#if !transfer.raw.source || !transfer.raw.asset}
            0
          {:else if !allDataReadyForBalance()}
            <Skeleton class="h-3 w-16 inline-block"/>
          {:else}
            {displayBalance}
          {/if}
        </div>
        <button
                class="cursor-pointer  text-xs text-sky-400 hover:text-sky-200"
                onclick={setMaxAmount}
        >
          MAX
        </button>
      </div>
    {/if}
{/if}
    <Input
            id="amount"
            type="text"
            required
            disabled={!transfer.raw.asset || disabled}
            autocorrect="off"
            placeholder="0.00"
            spellcheck="false"
            autocomplete="off"
            inputmode="decimal"
            data-field="amount"
            autocapitalize="none"
            pattern="^[0-9]*[.]?[0-9]*$"
            value={transfer.raw.amount}
            class="h-14 text-center text-lg"
            oninput={(event) => {
    const input = event.currentTarget
    const value = input.value

    const maxDecimals = Option.isSome(transfer.baseToken)
      ? transfer.baseToken.value.representations[0]?.decimals ?? 0
      : 0

    if (
      value === '' ||
      (
        /^\d*\.?\d*$/.test(value) &&
        (!value.includes('.') || value.split('.')[1].length <= maxDecimals)
      )
    ) {
      transfer.raw.updateField('amount', event)
    } else {
      input.value = transfer.raw.amount
    }
  }}
    />
  </div>
