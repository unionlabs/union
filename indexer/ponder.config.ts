import { fallback, http } from "viem";
import { ucs01relayAbi } from "./src/abi";
import { createConfig } from "@ponder/core";

export default createConfig({
  networks: {
    sepolia: {
      chainId: 11155111,
      transport: fallback([
        http("https://rpc.sepolia.org"),
        http(process.env.PONDER_RPC_URL_1),
        http("https://ethereum-sepolia.publicnode.com"),
        http(
          "https://eth-sepolia.g.alchemy.com/v2/daqIOE3zftkyQP_TKtb8XchSMCtc1_6D"
        ),
      ]),
    },
  },
  contracts: {
    UCS01_RELAY: {
      network: "sepolia",
      abi: ucs01relayAbi,
      address: process.env.UCS01_EVM_ADDRESS,
      startBlock: 5320122,
    },
  },
});
