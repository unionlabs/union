import { ucs01relayAbi } from "./src/abi";
import { createConfig } from "@ponder/core";
import { fallback, getAddress, http } from "viem";

export default createConfig({
  options: {
    maxHealthcheckDuration: 240, // default is 240
  },
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
      address: getAddress(process.env.UCS01_EVM_ADDRESS),
      startBlock: Number(process.env.INDEX_START_BLOCK),
    },
  },
});
