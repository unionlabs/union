import contracts from '~root/versions/contracts.json'

export const CHAINS = {
  sepolia: {
    chainId: "11155111",
    contracts: contracts[0]?.sepolia
  },
  osmosis: "osmo-test-5",
  stargaze: "elgafar-1"
}
