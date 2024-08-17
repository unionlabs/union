import {
  cosmosHttp,
  createPfmMemo,
  type OfflineSigner,
  hexAddressToBech32,
  bech32AddressToHex,
  createCosmosSdkClient,
  type EvmClientParameters,
  type CosmosClientParameters,
  type TransferAssetsParameters
} from "@union/client"
import { getAddress, http } from "viem"
import { get } from "svelte/store"
import { raise } from "$lib/utilities"
import type { Chain } from "$lib/types.ts"
import { cosmosStore } from "$lib/wallet/cosmos"
import { setup, assign, fromPromise } from "xstate"
import type { ChainWalletStore } from "$lib/wallet/types"
import { sepoliaStore, wagmiConfig } from "$lib/wallet/evm"

type Network = "cosmos" | "evm"

export const transferStateMachine = setup({
  actors: {
    createCosmosSdkClient: fromPromise(
      async ({
        input
      }: {
        input: {
          evm: EvmClientParameters
          cosmos: CosmosClientParameters
        }
      }) => createCosmosSdkClient(input)
    ),
    simulateTransaction: fromPromise(
      ({
        input
      }: {
        input: {
          client: ReturnType<typeof createCosmosSdkClient>
          parameters: TransferAssetsParameters
        }
      }) => input.client.simulateTransaction(input.parameters)
    )
  },
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
      evmClientParameters: EvmClientParameters | undefined
      cosmosClientParameters: CosmosClientParameters | undefined
      client: ReturnType<typeof createCosmosSdkClient> | undefined
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
      | { type: "SET_CLIENT" }
      | { type: "SHOW_RECEIPT" }
      | { type: "APPROVE_SPEND" }
      | { type: "TRANSFER_ASSET" }
      | { type: "APPROVE_TRANSFER" }
      | { type: "RECEIPT_RECEIVED" }
      | { type: "CONSTRUCT_PAYLOAD"; value: { chains: Array<Chain> } }
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
      | { type: "SET_EVM_CLIENT_PARAMETERS"; value: EvmClientParameters }
      | {
          type: "SET_COSMOS_CLIENT_PARAMETERS"
          value: {
            rpcUrl: string
            chainId: string
            account: OfflineSigner
            gasPrice: { amount: string; denom: string }
          }
        }
  },
  guards: {
    IS_EVM: ({ context }) => context.NETWORK === "evm",
    IS_COSMOS: ({ context }) => context.NETWORK === "cosmos",
    EVM_WALLET_CONNECTED: ({ context, event }) =>
      context.sepoliaStore.connectionStatus !== "connected" && window.ethereum !== undefined,
    ONE_OR_MORE_CLIENT_PARAMETERS_SET: ({ context }) =>
      context.evmClientParameters !== undefined || context.cosmosClientParameters !== undefined
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
    evmClientParameters: undefined,
    DESTINATION_CHAIN_ID: undefined,
    RELAY_CONTRACT_ADDRESS: undefined,
    cosmosClientParameters: undefined,
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
          actions: assign(({ event }) => ({
            ASSET_SYMBOL: event.value.symbol,
            ASSET_DENOM_ADDRESS: event.value.denomAddress
          }))
        },
        SET_AMOUNT: {
          actions: assign(({ event }) => ({ AMOUNT: event.value }))
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
          target: "SET_CLIENT",
          id: "#TRANSFER-MACHINE-CONSTRUCT-PAYLOAD",
          tags: ["construct-payload"],
          actions: assign(({ context, event }) => {
            const chains = context.chains ?? event.value.chains
            console.log("chains", chains)
            if (!context.RECIPIENT) return raise("Recipient not found")
            const sourceNetwork = context.NETWORK ?? raise("Network not found")
            const sourceChainId = context.SOURCE_CHAIN_ID ?? raise("Source chain not found")
            const destinationChainId =
              context.DESTINATION_CHAIN_ID ?? raise("Destination chain not found")

            const sourceChain = chains.find(chain => chain.chain_id === sourceChainId)
            // if (!sourceChain)
            //   return raise(
            //     `Source chain ${sourceChainId} not found ${sourceChainId} ${context.chains}`
            //   )
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
        },
        SET_EVM_CLIENT_PARAMETERS: {
          guard: "IS_EVM",
          target: "SET_CLIENT",
          actions: assign(({ event, context }) => {
            const account =
              event.value.account ?? wagmiConfig.getClient().account ?? raise("Account not found")
            return {
              evmClientParameters: {
                account,
                transport: http("https://sepolia.infura.io/v3/238b407ca9d049829b99b15b3fd99246"),
                chain:
                  (wagmiConfig.chains.find(chain => chain.id === Number(context.SOURCE_CHAIN_ID)) ||
                    event.value.chain) ??
                  raise("Chain not found")
              }
            }
          })
        },
        SET_COSMOS_CLIENT_PARAMETERS: {
          guard: "IS_COSMOS",
          target: "SET_CLIENT",
          actions: assign(({ event, context }) => {
            const offlineSigner =
              context.cosmosStore.connectedWallet === "keplr"
                ? window?.keplr?.getOfflineSigner(
                    `${event.value.chainId || context.SOURCE_CHAIN_ID}`,
                    { disableBalanceCheck: false }
                  )
                : context.cosmosStore.connectedWallet === "leap"
                  ? window?.leap?.getOfflineSigner(
                      `${event.value.chainId || context.SOURCE_CHAIN_ID}`,
                      { disableBalanceCheck: false }
                    )
                  : raise("Wallet not found")
            return {
              cosmosClientParameters: {
                account: offlineSigner,
                gasPrice: event.value.gasPrice,
                chainId: event.value.chainId || context.SOURCE_CHAIN_ID,
                transport: cosmosHttp(event.value.rpcUrl || context.RPC_URL)
              }
            }
          })
        }
      },
      target: "SET_CLIENT",
      onDone: [
        {
          target: "SET_CLIENT",
          guard: "ONE_OR_MORE_CLIENT_PARAMETERS_SET",
          actions: assign({
            client: ({ event, context }) =>
              createCosmosSdkClient({
                evm: context.evmClientParameters as EvmClientParameters,
                cosmos: context.cosmosClientParameters as CosmosClientParameters
              })
          })
        }
      ]
    },
    // CONSTRUCT_PAYLOAD: {
    //   id: "#TRANSFER-MACHINE-CONSTRUCT-PAYLOAD",
    //   tags: ["construct-payload"],
    //   always: [
    //     {
    //       actions: assign(({ context }) => {
    //         if (!context.RECIPIENT) return raise("Recipient not found")
    //         const sourceNetwork = context.NETWORK ?? raise("Network not found")
    //         const sourceChainId = context.SOURCE_CHAIN_ID ?? raise("Source chain not found")
    //         const destinationChainId =
    //           context.DESTINATION_CHAIN_ID ?? raise("Destination chain not found")

    //         const sourceChain = context.chains.find(chain => chain.chain_id === sourceChainId)
    //         const destinationChain = context.chains.find(
    //           chain => chain.chain_id === destinationChainId
    //         )

    //         console.info("sourceChain", sourceChain)
    //         console.info("destinationChain", destinationChain)

    //         if (!(sourceChain && destinationChain)) return raise("Chain not found")
    //         const prefix = destinationChain?.addr_prefix

    //         const ucsConfiguration = sourceChain?.ucs1_configurations
    //           .filter(config => config.destination_chain.chain_id === destinationChainId)
    //           .at(0)

    //         const forward = ucsConfiguration?.forward.find(
    //           item => item.destination_chain.chain_id === destinationChainId
    //         )

    //         const memo = createPfmMemo({
    //           port: forward?.port ?? raise("Port not found"),
    //           channel: forward?.channel_id ?? raise("Channel not found"),
    //           receiver:
    //             sourceNetwork === "evm" ? context.RECIPIENT.slice(2) : context.RECIPIENT ?? ""
    //         })

    //         return {
    //           sourceChainId,
    //           destinationChainId,
    //           PAYLOAD: {
    //             memo,
    //             network: sourceNetwork,
    //             path: [sourceChainId, destinationChainId],
    //             amount: context.AMOUNT ?? raise("Amount not found"),
    //             recipient: context.RECIPIENT ?? raise("Recipient not found"),
    //             sourceChannel: ucsConfiguration?.channel_id ?? raise("Channel not found"),
    //             relayContractAddress:
    //               ucsConfiguration?.contract_address ?? raise("Contract not found"),
    //             denomAddress: context.ASSET_DENOM_ADDRESS ?? raise("Denom address not found")
    //           }
    //         }
    //       })
    //     }
    //   ]
    // },
    SET_CLIENT: {
      tags: ["set-client"],
      id: "#TRANSFER-MACHINE-SET-CLIENT",
      output: ({ context }) => context.client,
      invoke: {
        src: "createCosmosSdkClient",
        id: "#CREATE-COSMOS-SDK-CLIENT",
        systemId: "#TRANSFER-MACHINE-SET-CLIENT",
        input: ({ context }) => ({
          evm: context.evmClientParameters as EvmClientParameters,
          cosmos: context.cosmosClientParameters as CosmosClientParameters
        }),
        onDone: {
          // target: "CONSTRUCT_PAYLOAD",
          actions: assign({ client: ({ event }) => event.output })
        }
      }
    },
    // CONSTRUCT_PAYLOAD: {
    //   tags: ["construct-payload"],
    //   id: "#TRANSFER-MACHINE-CONSTRUCT-PAYLOAD",
    //   output: ({ context }) => context.PAYLOAD,
    //   invoke: {
    //     src: "fetchUcsConfiguration",
    //     id: "#FETCH-UCS-CONFIGURATION",
    //     systemId: "#TRANSFER-MACHINE-CONSTRUCT-PAYLOAD",
    //     input: ({ context }) => ({
    //       SOURCE_CHAIN_ID: context.SOURCE_CHAIN_ID ?? raise("Source chain not found"),
    //       DESTINATION_CHAIN_ID: context.DESTINATION_CHAIN_ID ?? raise("Destination chain not found")
    //     }),
    //     onDone: {
    //       target: "SUCCESS",
    //       actions: assign({
    //         PAYLOAD: ({ event, context }) => {
    //           const sourceNetwork = context.NETWORK

    //           const destinationChain = context.chains.find(
    //             chain => chain.chain_id === event.output.PATH.at(1)
    //           )
    //           const prefix = destinationChain?.addr_prefix
    //           const destinationNetwork = destinationChain?.rpc_type

    //           const senderAddress =
    //             sourceNetwork === "evm"
    //               ? context.sepoliaStore.address ?? wagmiConfig.getClient().account?.address
    //               : sourceNetwork === "cosmos"
    //                 ? context.cosmosStore.address
    //                 : raise("No account found")

    //           if (!senderAddress) return raise("No account found")
    //           if (!prefix) return raise("No prefix found")
    //           const recipient =
    //             sourceNetwork === "evm" && destinationNetwork === "evm"
    //               ? senderAddress
    //               : sourceNetwork === "cosmos" && destinationNetwork === "cosmos"
    //                 ? senderAddress
    //                 : sourceNetwork === "evm" && destinationNetwork === "cosmos"
    //                   ? hexAddressToBech32({
    //                       bech32Prefix: prefix,
    //                       address: getAddress(senderAddress)
    //                     })
    //                   : sourceNetwork === "cosmos" && destinationNetwork === "evm"
    //                     ? bech32AddressToHex({ address: senderAddress })
    //                     : raise("Invalid address")

    //           const chain = context.chains.find(chain => chain.chain_id === context.SOURCE_CHAIN_ID)
    //           const ucsConfiguration = chain?.ucs1_configurations
    //             ?.filter(config => config.destination_chain.chain_id === "context.SOURCE_CHAIN_ID")
    //             .at(0)

    //           const forward = ucsConfiguration?.forward.find(
    //             item => item.destination_chain.chain_id === context.DESTINATION_CHAIN_ID
    //           )
    //           if (!(forward?.port && forward?.channel_id)) {
    //             return raise("Forward configuration not found")
    //           }
    //           const memo = createPfmMemo({
    //             port: forward?.port,
    //             channel: forward?.channel_id,
    //             receiver: context.DESTINATION_CHAIN_ID === "evm" ? recipient.slice(2) : recipient
    //           })

    //           return {
    //             memo,
    //             path: event.output.PATH,
    //             network: event.output.NETWORK,
    //             sourceChannel: event.output.SOURCE_CHANNEL,
    //             amount: context.AMOUNT ?? raise("Amount not found"),
    //             recipient: recipient ?? raise("Recipient not found"),
    //             relayContractAddress: event.output.RELAY_CONTRACT_ADDRESS,
    //             denomAddress: context.ASSET_DENOM_ADDRESS ?? raise("Denom address not found")
    //           }
    //         }
    //       })
    //     }
    //   }
    // },
    SUCCESS: {
      tags: ["success"],
      id: "#TRANSFER-MACHINE-SUCCESS",
      output: ({ context }) => context.client
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
