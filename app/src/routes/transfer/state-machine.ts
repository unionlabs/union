import {
  createPfmMemo,
  hexAddressToBech32,
  bech32AddressToHex,
  type TransferAssetsParameters
} from "@union/client"
import { getAddress } from "viem"
import { get } from "svelte/store"
import { raise } from "$lib/utilities"
import type { Chain } from "$lib/types.ts"
import { cosmosStore } from "$lib/wallet/cosmos"
import { setup, assign, fromPromise } from "xstate"
import type { ChainWalletStore } from "$lib/wallet/types"
import { sepoliaStore, wagmiConfig } from "$lib/wallet/evm"

type Network = "cosmos" | "evm"

export const transferStateMachine = setup({
  actors: {},
  types: {
    input: {} as {
      chains: Array<Chain>
      sepoliaStore: ChainWalletStore<"evm"> | undefined
      cosmosStore: ChainWalletStore<"cosmos"> | undefined
    },
    context: {} as {
      error: unknown
      chains: Array<Chain>
      AMOUNT: bigint | undefined
      RPC_URL: string | undefined
      NETWORK: Network | undefined
      RECIPIENT: string | undefined
      ASSET_SYMBOL: string | undefined
      PATH: [string, string] | undefined
      SOURCE_CHANNEL: string | undefined
      SOURCE_CHAIN_ID: string | undefined
      sepoliaStore: ChainWalletStore<"evm">
      ASSET_DENOM_ADDRESS: string | undefined
      cosmosStore: ChainWalletStore<"cosmos">
      DESTINATION_CHAIN_ID: string | undefined
      RELAY_CONTRACT_ADDRESS: string | undefined
      PAYLOAD: TransferAssetsParameters | undefined
    },
    events: {} as
      | {
          type: "SET_SOURCE_CHAIN"
          value: { chainId: string; network: Network }
        }
      | {
          type: "SET_ASSET"
          value: { symbol: string; denomAddress: string }
        }
      | { type: "SUCCESS" }
      | { type: "SHOW_RECEIPT" }
      | { type: "APPROVE_SPEND" }
      | { type: "TRANSFER_ASSET" }
      | { type: "APPROVE_TRANSFER" }
      | { type: "RECEIPT_RECEIVED" }
      | { type: "CONSTRUCT_PAYLOAD"; value: { chains: Array<Chain> } }
      | { type: "CONSTRUCT_PFM_PAYLOAD"; value: { chains: Array<Chain> } }
      | {
          type: "CREATE_PFM_MEMO"
          value: { port: string; receiver: string; channelId: string }
        }
      | { type: "SET_AMOUNT"; value: bigint }
      | { type: "APPROVAL_RECEIPT_RECEIVED" }
      | { type: "SWITCH_CHAIN"; value: string }
      | { type: "SET_RECIPIENT"; value: string }
      | { type: "TRANSFER_SIMULATION_APPROVED" }
      | { type: "SET_DESTINATION_CHAIN"; value: string }
  },
  guards: {
    IS_EVM: ({ context }) => context.NETWORK === "evm",
    IS_COSMOS: ({ context }) => context.NETWORK === "cosmos",
    EVM_WALLET_CONNECTED: ({ context }) => context.sepoliaStore.connectionStatus !== "connected",
    IS_NOT_PFM: ({ context }) =>
      ![context.SOURCE_CHAIN_ID, context.DESTINATION_CHAIN_ID].includes("union-testnet-8"),
    IS_PFM: ({ context }) =>
      ![context.SOURCE_CHAIN_ID, context.DESTINATION_CHAIN_ID].includes("union-testnet-8")
  }
}).createMachine({
  context: ({ input }) => ({
    PATH: undefined,
    error: undefined,
    AMOUNT: undefined,
    client: undefined,
    PAYLOAD: undefined,
    NETWORK: undefined,
    RPC_URL: undefined,
    RECIPIENT: undefined,
    chains: input.chains,
    ASSET_SYMBOL: undefined,
    DENOM_ADDRESS: undefined,
    SOURCE_CHANNEL: undefined,
    SOURCE_CHAIN_ID: undefined,
    ASSET_DENOM_ADDRESS: undefined,
    DESTINATION_CHAIN_ID: undefined,
    RELAY_CONTRACT_ADDRESS: undefined,
    cosmosStore: input?.cosmosStore ?? get(cosmosStore),
    sepoliaStore: input?.sepoliaStore ?? get(sepoliaStore)
  }),
  id: "transfer",
  initial: "START",
  states: {
    START: {
      tags: ["start"],
      id: "#TRANSFER-MACHINE-START",
      on: {
        SET_SOURCE_CHAIN: {
          actions: assign(({ event }) => ({
            NETWORK: event.value.network,
            SOURCE_CHAIN_ID: event.value.chainId
          }))
        },
        SET_DESTINATION_CHAIN: {
          actions: assign(({ event, context }) => {
            return {
              DESTINATION_CHAIN_ID: event.value,
              RECIPIENT: context.RECIPIENT // ?? recipient
            }
          })
        },
        SET_ASSET: {
          actions: [
            assign(({ event }) => ({
              ASSET_SYMBOL: event.value.symbol,
              ASSET_DENOM_ADDRESS: event.value.denomAddress
            }))
          ]
        },
        SET_AMOUNT: {
          actions: [assign(({ event }) => ({ AMOUNT: event.value }))]
        },
        SET_RECIPIENT: {
          tags: ["set-recipient"],
          actions: assign(({ event, context }) => {
            const sourceNetwork = context.NETWORK

            const destinationChain = context.chains.find(
              chain => chain.chain_id === context.DESTINATION_CHAIN_ID
            )
            const prefix = destinationChain?.addr_prefix
            const destinationNetwork = destinationChain?.rpc_type

            const senderAddress =
              sourceNetwork === "evm"
                ? context.sepoliaStore.address ?? wagmiConfig.getClient().account?.address
                : sourceNetwork === "cosmos"
                  ? context.cosmosStore.address
                  : raise("No account found")

            // if (!senderAddress) return raise("No account found")
            // if (!prefix) return raise("No prefix found")

            const recipient = () => {
              if (event.value) return event.value
              if (!(senderAddress && prefix)) return ""
              return sourceNetwork === "evm" && destinationNetwork === "evm"
                ? senderAddress
                : sourceNetwork === "cosmos" && destinationNetwork === "cosmos"
                  ? senderAddress
                  : sourceNetwork === "evm" && destinationNetwork === "cosmos"
                    ? hexAddressToBech32({
                        bech32Prefix: prefix,
                        address: getAddress(senderAddress)
                      })
                    : sourceNetwork === "cosmos" && destinationNetwork === "evm"
                      ? bech32AddressToHex({ address: senderAddress })
                      : raise("Invalid address")
            }

            return { RECIPIENT: event.value ?? recipient() }
          })
        },
        CONSTRUCT_PAYLOAD: {
          guard: "IS_NOT_PFM",
          tags: ["construct-payload"],
          id: "#TRANSFER-MACHINE-CONSTRUCT-PAYLOAD",
          actions: [
            assign(({ context, event }) => {
              const chains = context.chains ?? event.value.chains
              const amount = context.AMOUNT ?? raise("amount not found")
              const network = context.NETWORK ?? raise("Network not found")
              const recipient = context.RECIPIENT ?? raise("recipient not found")
              const denomAddress = context.ASSET_DENOM_ADDRESS ?? raise("denom address not found")

              const sourceChainId = context.SOURCE_CHAIN_ID ?? raise("Source chain not found")
              const destinationChainId =
                context.DESTINATION_CHAIN_ID ?? raise("Destination chain not found")

              const sourceChain = chains.find(chain => chain.chain_id === sourceChainId)
              const destinationChain = chains.find(chain => chain.chain_id === destinationChainId)
              if (!(sourceChain && destinationChain)) return raise("Chain not found")

              const ucsConfiguration = sourceChain?.ucs1_configurations[sourceChainId]

              return {
                sourceChainId,
                destinationChainId,
                PAYLOAD: {
                  network,
                  amount: amount,
                  recipient: recipient,
                  path: [sourceChainId, destinationChainId],
                  sourceChannel: ucsConfiguration?.channel_id ?? raise("Channel not found"),
                  relayContractAddress:
                    ucsConfiguration?.contract_address ?? raise("Contract not found"),
                  denomAddress: context.ASSET_DENOM_ADDRESS ?? raise("Denom address not found")
                }
              }
            })
          ]
        },
        CONSTRUCT_PFM_PAYLOAD: {
          guard: "IS_PFM",
          tags: ["construct-payload"],
          id: "#TRANSFER-MACHINE-CONSTRUCT-PFM-PAYLOAD",
          actions: [
            assign(({ context, event }) => {
              const chains = context.chains ?? event.value.chains
              console.log("chains", chains)
              if (!context.RECIPIENT) return raise("Recipient not found")
              const sourceNetwork = context.NETWORK ?? raise("Network not found")
              const sourceChainId = context.SOURCE_CHAIN_ID ?? raise("Source chain not found")
              const destinationChainId =
                context.DESTINATION_CHAIN_ID ?? raise("Destination chain not found")

              const sourceChain = chains.find(chain => chain.chain_id === sourceChainId)
              const destinationChain = chains.find(chain => chain.chain_id === destinationChainId)

              console.info("sourceChain", sourceChain)
              console.info("destinationChain", destinationChain)

              // if (!(sourceChain && destinationChain)) return raise("Chain not found")
              const prefix = destinationChain?.addr_prefix

              const ucsConfiguration = sourceChain?.ucs1_configurations["union-testnet-8"]

              const forward = ucsConfiguration?.forward[destinationChainId]

              const memo = createPfmMemo({
                port: forward?.port ?? raise("Port not found"),
                channel: forward?.channel_id ?? raise("Channel not found"),
                receiver:
                  sourceNetwork === "evm" ? context.RECIPIENT.slice(2) : context.RECIPIENT ?? ""
              })

              return {
                sourceChainId,
                destinationChainId,
                PAYLOAD: {
                  memo,
                  network: sourceNetwork,
                  path: [sourceChainId, "union-testnet-8"],
                  amount: context.AMOUNT ?? raise("Amount not found"),
                  recipient: context.RECIPIENT ?? raise("Recipient not found"),
                  sourceChannel: ucsConfiguration?.channel_id ?? raise("Channel not found"),
                  relayContractAddress:
                    ucsConfiguration?.contract_address ?? raise("Contract not found"),
                  denomAddress: context.ASSET_DENOM_ADDRESS ?? raise("Denom address not found")
                }
              }
            })
          ]
        }
      }
    },
    SUCCESS: {
      tags: ["success"],
      id: "#TRANSFER-MACHINE-SUCCESS"
    }
  },
  output: ({ context }) => context
})

// const actor = createActor(transferStateMachine)
// const subscription = actor.subscribe({
//   next: snapshot =>
//     snapshot.hasTag("success")
//       ? console.info(snapshot.context.client?.getBlockNumber().then(console.log))
//       : false
// })
// actor.start()

// actor.send({ type: "SET_SOURCE_CHAIN", value: { chainId: "80084", network: "evm" } })
// actor.send({ type: "SET_DESTINATION_CHAIN", value: "stride-internal-1" })
// actor.send({
//   type: "SET_ASSET",
//   value: { denomAddress: "0x0E4aaF1351de4c0264C5c7056Ef3777b41BD8e03", symbol: "HONEY" }
// })
// actor.send({ type: "SET_AMOUNT", value: 1n })
// actor.send({ type: "CONSTRUCT_PAYLOAD" })
// actor.send({ type: "SUCCESS" })
// actor.send({
//   type: "SET_EVM_CLIENT_PARAMETERS",
//   value: {
//     transport: fallback([http(sepolia.rpcUrls.default.http.at(0))])
//   }
// })
// actor.send({
//   type: "SET_CLIENT"
// })
