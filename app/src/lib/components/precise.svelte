<script lang="ts">
import * as Tooltip from "$lib/components/ui/tooltip"
import type { Chain } from "$lib/types.ts"
import { findAsset } from "$lib/utilities/helpers.ts"

export let asset: any
export let chain: Chain
export let displayDecimals = 2
export let showToolTip = false
export let showSymbol = false

const formatBalance = (balance: bigint, decimals: number, abbreviate = false): string => {
  if (!balance || Number.isNaN(Number(balance))) return "0.00"
  const num = BigInt(balance)
  const divisor = BigInt(10 ** decimals)
  const rawNumber = num / divisor
  const remainder = num % divisor

  let baseFormattedNumber = rawNumber.toString()
  if (remainder !== BigInt(0)) {
    const fractionalPart = remainder.toString().padStart(decimals, "0").slice(0, decimals)
    baseFormattedNumber += `.${fractionalPart}`
  }

  return abbreviate
    ? abbreviateNumber(Number.parseFloat(baseFormattedNumber), displayDecimals)
    : baseFormattedNumber
}

const abbreviateNumber = (num: number, displayDecimals: number): string => {
  if (num >= 1e12) return `${(num / 1e12).toFixed(displayDecimals)}T`
  if (num >= 1e9) return `${(num / 1e9).toFixed(displayDecimals)}B`
  if (num >= 1e6) return `${(num / 1e6).toFixed(displayDecimals)}M`
  if (num >= 1e3) return `${(num / 1e3).toFixed(displayDecimals)}K`
  return num.toFixed(displayDecimals)
}

$: assetOnChain = findAsset(chain, asset.denom)
$: decimals = assetOnChain ? assetOnChain.decimals : asset.decimals
$: symbol = assetOnChain ? assetOnChain.display_symbol : asset.symbol
$: formatted = formatBalance(asset.balance, decimals, true)
$: precise = formatBalance(asset.balance, decimals, false)
</script>

{#key formatted}
  {#if showToolTip}
    <Tooltip.Root>
      <Tooltip.Trigger>
        <span class="cursor-crosshair">
          {formatted} {showSymbol ? symbol : ''}</span>
      </Tooltip.Trigger>
      <Tooltip.Content>
        <p>{precise}</p>
      </Tooltip.Content>
    </Tooltip.Root>
  {:else}
    <span>{formatted} {showSymbol ? symbol : ''}</span>
  {/if}
{/key}
