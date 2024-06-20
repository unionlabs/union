<script lang="ts">
import * as Tooltip from "$lib/components/ui/tooltip"

//Takes supportedAsset info from chain and a asset to construct formatted balance

export let asset: any
export let supportedAsset: any
export let displayDecimals = 2
export let showToolTip = false
export let showSymbol = false

const formatBalance = (balance: any, decimals: number, abbreviate = false): string => {
  if (balance === undefined || balance === null || Number.isNaN(Number(balance))) return "0.00"
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

$: balance = asset.balance
$: decimals = supportedAsset.decimals
$: symbol = supportedAsset.display_symbol

$: formatted = formatBalance(balance, decimals, true)
$: precise = formatBalance(balance, decimals, false)
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

