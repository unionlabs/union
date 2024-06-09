import contracts from "~root/versions/contracts.json"

export const network = ["evm", "cosmos"] as const
export type Network = (typeof network)[number]

export const chainId = ["11155111", "union-testnet-8", "osmo-test-5", "elgafar-1"] as const
export type ChainId = (typeof chainId)[number]

export const CHAINS = {
  sepolia: {
    chainId: "11155111",
    contracts: contracts[0]?.sepolia
  },
  osmosis: "osmo-test-5",
  stargaze: "elgafar-1"
}
