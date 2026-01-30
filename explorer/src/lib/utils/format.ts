import type { ChainAsset } from "$lib/chains/config"
import type { Coin } from "$lib/types/cosmos"

export const truncateHash = (hash: string | null | undefined, len = 8): string => {
  if (!hash) {
    return "-"
  }
  if (hash.length <= len * 2 + 3) {
    return hash
  }
  return `${hash.slice(0, len)}...${hash.slice(-len)}`
}

export const truncateAddress = (address: string | null | undefined, len = 8): string => {
  if (!address) {
    return "-"
  }
  if (address.length <= len * 2 + 3) {
    return address
  }
  return `${address.slice(0, len)}...${address.slice(-len)}`
}

export const formatTime = (timestamp: string | null | undefined): string => {
  if (!timestamp) {
    return "-"
  }
  const d = new Date(timestamp)
  return d.toLocaleString()
}

export const formatTimeAgo = (timestamp: string | null | undefined): string => {
  if (!timestamp) {
    return "-"
  }
  const d = new Date(timestamp)
  const now = new Date()
  const diff = now.getTime() - d.getTime()

  const seconds = Math.floor(diff / 1000)
  if (seconds < 60) {
    return `${seconds}s ago`
  }

  const minutes = Math.floor(seconds / 60)
  if (minutes < 60) {
    return `${minutes}m ago`
  }

  const hours = Math.floor(minutes / 60)
  if (hours < 24) {
    return `${hours}h ago`
  }

  const days = Math.floor(hours / 24)
  return `${days}d ago`
}

export const formatNumber = (num: string | number | bigint, decimals = 2): string => {
  const n = typeof num === "bigint" ? Number(num) : Number(num)
  if (isNaN(n)) {
    return "-"
  }

  if (n >= 1_000_000_000_000) {
    return `${(n / 1_000_000_000_000).toFixed(decimals)}T`
  }
  if (n >= 1_000_000_000) {
    return `${(n / 1_000_000_000).toFixed(decimals)}B`
  }
  if (n >= 1_000_000) {
    return `${(n / 1_000_000).toFixed(decimals)}M`
  }
  if (n >= 1_000) {
    return `${(n / 1_000).toFixed(decimals)}K`
  }
  return n.toFixed(decimals)
}

export const formatNumberWithCommas = (num: string | number | bigint): string => {
  const n = typeof num === "bigint" ? num.toString() : String(num)
  const parts = n.split(".")
  parts[0] = parts[0].replace(/\B(?=(\d{3})+(?!\d))/g, ",")
  return parts.join(".")
}

export const formatAmount = (amount: string | number, exponent = 6, compact = true): string => {
  try {
    const num = BigInt(String(amount).split(".")[0]) // Handle any decimal in input
    const divisor = BigInt(10 ** exponent)
    const whole = num / divisor
    const remainder = num % divisor

    // For very large numbers, work in BigInt space to avoid precision loss
    if (compact && whole >= 1_000_000_000_000n) {
      const trillions = whole / 1_000_000_000_000n
      const trillionRemainder = (whole % 1_000_000_000_000n) / 10_000_000_000n // 2 decimal places
      return `${trillions}.${trillionRemainder.toString().padStart(2, "0")}T`
    }
    if (compact && whole >= 1_000_000_000n) {
      const billions = whole / 1_000_000_000n
      const billionRemainder = (whole % 1_000_000_000n) / 10_000_000n
      return `${billions}.${billionRemainder.toString().padStart(2, "0")}B`
    }
    if (compact && whole >= 1_000_000n) {
      const millions = whole / 1_000_000n
      const millionRemainder = (whole % 1_000_000n) / 10_000n
      return `${millions}.${millionRemainder.toString().padStart(2, "0")}M`
    }
    if (compact && whole >= 1_000n) {
      const thousands = whole / 1_000n
      const thousandRemainder = (whole % 1_000n) / 10n
      return `${thousands}.${thousandRemainder.toString().padStart(2, "0")}K`
    }

    // For smaller numbers, convert to Number safely
    const decimalStr = remainder.toString().padStart(exponent, "0")
    const value = Number(whole) + Number(`0.${decimalStr}`)

    // For smaller numbers, show with appropriate decimal places
    if (value < 0.000001 && value > 0) {
      return "< 0.000001"
    }
    if (value < 0.01 && value > 0) {
      return value.toFixed(6)
    }
    if (value < 1) {
      return value.toFixed(4)
    }
    if (value < 1000) {
      return value.toFixed(2)
    }

    // Fallback for non-compact display
    return formatNumberWithCommas(value.toFixed(2))
  } catch {
    return "-"
  }
}

export const formatAmountFull = (amount: string | number, exponent = 6): string => {
  try {
    const num = BigInt(String(amount).split(".")[0])
    const divisor = BigInt(10 ** exponent)
    const whole = num / divisor
    const remainder = num % divisor
    const decimalStr = remainder.toString().padStart(exponent, "0").slice(0, 6)
    const wholeStr = whole.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ",")
    return `${wholeStr}.${decimalStr}`
  } catch {
    return "-"
  }
}

export const formatCoin = (coin: Coin, assets: ChainAsset[]): string => {
  const asset = assets.find((a) => a.base === coin.denom)
  if (asset) {
    return `${formatAmount(coin.amount, asset.exponent)} ${asset.symbol}`
  }
  // Handle IBC denoms
  if (coin.denom.startsWith("ibc/")) {
    return `${formatAmount(coin.amount, 6)} ${truncateHash(coin.denom, 4)}`
  }
  return `${coin.amount} ${coin.denom}`
}

export const formatPercent = (value: string | number): string => {
  const num = Number(value)
  if (isNaN(num)) {
    return "-"
  }
  return `${(num * 100).toFixed(2)}%`
}

export const formatVotingPower = (tokens: string, totalBonded: string): string => {
  const power = BigInt(tokens)
  const total = BigInt(totalBonded)
  if (total === 0n) {
    return "0%"
  }
  const percent = (power * 10000n) / total
  return `${(Number(percent) / 100).toFixed(2)}%`
}

export const getStatusColor = (status: string): string => {
  switch (status.toUpperCase()) {
    case "BOND_STATUS_BONDED":
    case "PROPOSAL_STATUS_PASSED":
    case "PASSED":
      return "text-green-400"
    case "BOND_STATUS_UNBONDING":
    case "PROPOSAL_STATUS_VOTING_PERIOD":
    case "VOTING_PERIOD":
      return "text-yellow-400"
    case "BOND_STATUS_UNBONDED":
    case "PROPOSAL_STATUS_REJECTED":
    case "REJECTED":
      return "text-red-400"
    case "PROPOSAL_STATUS_DEPOSIT_PERIOD":
    case "DEPOSIT_PERIOD":
      return "text-blue-400"
    default:
      return "text-muted-foreground"
  }
}

export const formatStatus = (status: string): string => {
  return status
    .replace("BOND_STATUS_", "")
    .replace("PROPOSAL_STATUS_", "")
    .replace(/_/g, " ")
    .toLowerCase()
    .replace(/\b\w/g, (l) => l.toUpperCase())
}
