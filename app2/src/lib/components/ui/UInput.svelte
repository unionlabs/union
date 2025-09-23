<script lang="ts">
import Input from "$lib/components/ui/Input.svelte"
import Label from "$lib/components/ui/Label.svelte"
import { Utils } from "@unionlabs/sdk"
import type { Token } from "@unionlabs/sdk/schema"
import { BigDecimal, pipe } from "effect"
import { Struct } from "effect"
import * as A from "effect/Array"
import * as O from "effect/Option"

interface Props {
  id: string
  label: string
  placeholder?: string
  disabled?: boolean
  decimals?: number
  token?: O.Option<Token>
  balance?: O.Option<bigint>
  class?: string
  humanValue: string
  weiValue: O.Option<bigint>
}

let {
  id,
  label,
  placeholder = "Enter amount",
  disabled = false,
  decimals = 18,
  token = O.none(),
  balance = O.none(),
  class: className = "h-14 pl-4 text-left text-lg",
  humanValue = $bindable(),
  weiValue = $bindable(),
}: Props = $props()

// Get decimals from token if available, otherwise use provided decimals
const tokenDecimals = $derived(pipe(
  token,
  O.map(Struct.get("representations")),
  O.flatMap(A.head),
  O.map(Struct.get("decimals")),
  O.getOrElse(() => decimals),
))

// Convert human-readable input to BigDecimal
const inputAmount = $derived<O.Option<BigDecimal.BigDecimal>>(pipe(
  humanValue,
  BigDecimal.fromString,
))

// Convert BigDecimal to wei (bigint) using $effect to update the bindable
$effect(() => {
  weiValue = pipe(
    inputAmount,
    O.map(bd => {
      const result = BigDecimal.multiply(bd, BigDecimal.make(10n ** BigInt(tokenDecimals), 0))
      const normalized = BigDecimal.normalize(result)
      return normalized.scale >= 0
        ? normalized.value / (10n ** BigInt(normalized.scale))
        : normalized.value * (10n ** BigInt(-normalized.scale))
    }),
  )
})

// Input validation handler
function handleBeforeInput(event: InputEvent & { currentTarget: HTMLInputElement }) {
  const { inputType, data, currentTarget } = event
  const { value } = currentTarget
  const proposed = value + (data ?? "")

  const validShape = /^\d*[.,]?\d*$/.test(proposed)
  const validDecimalsDot = !proposed.includes(".")
    || proposed.split(".")[1].length <= tokenDecimals
  const validDecimalsComma = !proposed.includes(",")
    || proposed.split(",")[1].length <= tokenDecimals
  const isDelete = inputType.startsWith("delete")
  const validDecimals = validDecimalsComma && validDecimalsDot
  const noDuplicateLeadingZeroes = !proposed.startsWith("00")

  const allow = isDelete
    || (validDecimals && validShape && noDuplicateLeadingZeroes)

  if (!allow) {
    event.preventDefault()
  }
}

// Handle MAX button click
function handleMaxClick() {
  if (O.isSome(balance)) {
    // Convert balance (wei) to human readable format
    const balanceDecimal = pipe(
      BigDecimal.fromBigInt(balance.value),
      BigDecimal.unsafeDivide(BigDecimal.make(10n ** BigInt(tokenDecimals), 0)),
    )
    humanValue = Utils.formatBigDecimal(balanceDecimal)
  }
}
</script>

<div>
  <Label for={id}>{label}</Label>
  <div class="relative flex items-center">
    <input
      {id}
      type="text"
      required
      {disabled}
      autocorrect="off"
      {placeholder}
      spellcheck="false"
      autocomplete="off"
      inputmode="decimal"
      data-field="amount"
      onbeforeinput={handleBeforeInput}
      autocapitalize="none"
      pattern="^[0-9]*[.,]?[0-9]*$"
      value={humanValue}
      class="w-full p-2 bg-zinc-800/70 rounded-md focus:outline-none focus:ring-1 focus:ring-accent disabled:text-zinc-400 disabled:cursor-not-allowed transition-colors duration-200 {className}"
      aria-label={label}
      oninput={(event) => {
        humanValue = event.currentTarget.value
      }}
    />
    {#if O.isSome(balance)}
      <button
        type="button"
        onclick={handleMaxClick}
        class="absolute right-2 px-2 py-1 text-xs font-medium text-zinc-400 hover:text-zinc-300 bg-zinc-800/50 hover:bg-zinc-700/50 rounded border border-zinc-700/50 hover:border-zinc-600/50 transition-colors cursor-pointer"
        disabled={disabled}
      >
        MAX
      </button>
    {/if}
  </div>
</div>
