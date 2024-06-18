<script lang="ts">
import * as Tooltip from "$lib/components/ui/tooltip"
import type { Chain } from "$lib/types.ts"
import { findAsset } from "$lib/utilities/helpers.ts"

export let asset: any
export let chain: Chain
export let displayDecimals = 2
export let toolTip = false
export let symbol = false

const format = (balance: string | bigint, decimals: number, abbreviate: boolean): string => {
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

  return abbreviate ? abbreviateNumber(Number.parseFloat(baseFormattedNumber)) : baseFormattedNumber
}

const abbreviateNumber = (num: number): string => {
  if (num >= 1e12) return `${(num / 1e12).toFixed(displayDecimals)}T`
  if (num >= 1e9) return `${(num / 1e9).toFixed(displayDecimals)}B`
  if (num >= 1e6) return `${(num / 1e6).toFixed(displayDecimals)}M`
  if (num >= 1e3) return `${(num / 1e3).toFixed(displayDecimals)}K`
  return num.toFixed(displayDecimals)
}

$: console.log(chain, asset)
$: info = findAsset(chain, asset.denom)
$: formatted = format(asset.balance, info ? info.decimals : asset.decimals, true)
$: precise = format(asset.balance, info ? info.decimals : asset.decimals, false)
</script>

{#key formatted}
  {#if toolTip}
    <Tooltip.Root>
      <Tooltip.Trigger>
        <span class="cursor-crosshair">
          {formatted} {symbol ? info ? info.display_symbol : asset.symbol : ''}</span>
      </Tooltip.Trigger>
      <Tooltip.Content>
        <p>{precise}</p>
      </Tooltip.Content>
    </Tooltip.Root>
  {:else}
    <span>{formatted} {symbol ? info ? info.display_symbol : asset.symbol : ''}</span>
  {/if}
{/key}
