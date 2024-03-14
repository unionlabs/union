import { ucs01relayAbi } from "./src/abi"
import { createConfig } from "@ponder/core"
import { Transport, erc20Abi, fallback, http, webSocket } from "viem"

const UCS01_EVM_ADDRESS = process.env.UCS01_EVM_ADDRESS
const UNO_ERC20_ADDRESS = process.env.UNO_ERC20_ADDRESS

if (!(UCS01_EVM_ADDRESS && UNO_ERC20_ADDRESS)) {
  // biome-ignore lint/nursery/noConsole: <explanation>
  console.error("Missing UCS01_EVM_ADDRESS or UNO_ERC20_ADDRESS")
  process.exit(1)
}

export default createConfig({
  options: {
    maxHealthcheckDuration: 4 * 60 // default
  },
  networks: {
    sepolia: {
      chainId: 11155111,
      transport: fallback([
        http(
          "https://rpc.ankr.com/eth_sepolia/6c72c8d164912bed4694cb882fc4ca55574126511a4f5f66828a53fa2448a20a"
        ),
        webSocket("wss://eth-sepolia.g.alchemy.com/v2/N7GR7dabsLMjanJMh5Ur0fAJ8AA9KdGz"),
        http("https://gateway.tenderly.co/public/sepolia"),
        http("https://sepolia.gateway.tenderly.co")
      ])
    }
  },
  contracts: {
    UCS01_RELAY: {
      network: "sepolia",
      abi: ucs01relayAbi,
      address: process.env.UCS01_EVM_ADDRESS,
      startBlock: 5320122
    },
    UNO_ERC20: {
      network: "sepolia",
      abi: erc20Abi,
      address: process.env.UNO_ERC20_ADDRESS,
      startBlock: 5320122
    }
  }
})
