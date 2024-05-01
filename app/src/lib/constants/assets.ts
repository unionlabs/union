export const chains = ["SEPOLIA", "UNION"] as const
export type Chain = (typeof chains)[number]

export const assets = {
  UNION: ["UNO"],
  SEPOLIA: ["ETH", "UNO"]
} as const satisfies Record<Chain, Array<string>>

export type Asset = (typeof assets)[Chain][number] // all assets
export type ChainAsset<T extends Chain> = (typeof assets)[T][number] // assets for a specific chain
