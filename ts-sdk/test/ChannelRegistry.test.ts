import { describe, it } from "@effect/vitest"
import { ChannelRegistry } from "@unionlabs/sdk/ChannelRegistry"
import { Effect, Schema } from "effect"
import { Chain } from "../src/schema/chain.js"

describe("ChannelRegistry", () => {
  it.layer(ChannelRegistry.Test)("Test", (it) => {
    it.effect("pick", () =>
      Effect.gen(function*() {
        const source = yield* Schema.decode(Chain)(
          {
            chain_id: "xion-testnet-2",
            universal_chain_id: "xion.xion-testnet-2",
            minter_address_display:
              "xion1ak8muzgf2nv5ukzeg2wpf2vls74247et33cfhmg87uvpey73xdeqexkzgc",
            display_name: "Xion Testnet",
            rpc_type: "cosmos",
            addr_prefix: "xion",
            testnet: true,
            features: [
              {
                channel_list: false,
                connection_list: false,
                index_status: false,
                packet_list: false,
                transfer_submission: false,
                transfer_list: false,
              },
            ],
            rpcs: [
              {
                type: "rpc",
                url: "https://rpc.xion-testnet-2.burnt.com",
              },
              {
                type: "rest",
                url: "https://api.xion-testnet-2.burnt.com",
              },
              {
                type: "grpc",
                url: "https://grpc.xion-testnet-2.burnt.com",
              },
            ],
            explorers: [
              {
                address_url: "https://www.mintscan.io/xion-testnet/account/",
                block_url: "https://www.mintscan.io/xion-testnet/block/",
                description: "Mintscan",
                display_name: "Mintscan",
                home_url: "https://www.mintscan.io/xion-testnet/",
                name: "mintscan",
                tx_url: "https://www.mintscan.io/xion-testnet/tx/",
              },
            ],
            editions: [
              {
                environment: "production",
                name: "app",
              },
            ],
          },
        )
        const destination = yield* Schema.decode(Chain)(
          {
            chain_id: "1328",
            universal_chain_id: "sei.1328",
            minter_address_display: "0x5fbe74a283f7954f10aa04c2edf55578811aeb03",
            display_name: "Sei Testnet",
            rpc_type: "evm",
            addr_prefix: "0x",
            testnet: true,
            features: [
              {
                channel_list: false,
                connection_list: false,
                index_status: false,
                packet_list: false,
                transfer_submission: false,
                transfer_list: false,
              },
            ],
            rpcs: [
              {
                type: "rpc",
                url: "https://evm-rpc.1328.sei.chain.kitchen",
              },
            ],
            explorers: [
              {
                address_url: "https://dashboard.tenderly.co/wallet/1328/",
                block_url: "https://dashboard.tenderly.co/block/1328/",
                description: "Tenderly",
                display_name: "Tenderly",
                home_url: "https://dashboard.tenderly.co/explorer/sei-atlantic-2",
                name: "tenderly",
                tx_url: "https://dashboard.tenderly.co/tx/",
              },
            ],
            editions: [
              {
                environment: "production",
                name: "app",
              },
            ],
          },
        )

        const result = yield* ChannelRegistry.pick(source, destination)

        console.log("channel", result)
      }))
  })
})
