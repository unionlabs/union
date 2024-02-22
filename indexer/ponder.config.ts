import { http } from "viem";
import { ucs01relayAbi } from "./src/abi";
import { createConfig } from "@ponder/core";

export default createConfig({
  networks: {
    sepolia: {
      chainId: 11155111,
      transport: http(process.env.PONDER_RPC_URL_1),
    },
  },
  contracts: {
    UCS01_RELAY: {
      network: "sepolia",
      abi: ucs01relayAbi,
      address: process.env.UCS01_EVM_ADDRESS,
      startBlock: process.env.INDEX_START_BLOCK || 5_338_569, // recent block
    },
  },
});
