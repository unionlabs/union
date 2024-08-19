import {
  createPfmMemo,
  hexAddressToBech32,
  bech32AddressToHex,
  type TransferAssetsParameters
} from "@union/client"
import { getAddress } from "viem"
import { get } from "svelte/store"
import { raise } from "$lib/utilities"
import { setup, assign } from "xstate"
import { cosmosStore } from "$lib/wallet/cosmos"
import type { Asset, Chain } from "$lib/types.ts"
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
      ASSET: Asset | undefined
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
      | { type: "SET_DESTINATION_CHAIN"; value: string }
      | { type: "SET_ASSET"; value: Asset }
      | { type: "SET_AMOUNT"; value: bigint }
      | { type: "SET_RECIPIENT"; value: string }
      | { type: "SET_RELAY_CONTRACT_ADDRESS"; value: string }
      | { type: "SET_SOURCE_CHANNEL"; value: string }
      //
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
      | { type: "APPROVAL_RECEIPT_RECEIVED" }
      | { type: "SWITCH_CHAIN"; value: string }
      | { type: "TRANSFER_SIMULATION_APPROVED" }
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
    ASSET: undefined,
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
              RECIPIENT: context.RECIPIENT
            }
          })
        },
        SET_ASSET: {
          actions: [assign(({ event }) => ({ ASSET: event.value }))]
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
        SET_RELAY_CONTRACT_ADDRESS: {
          actions: [assign(({ event }) => ({ RELAY_CONTRACT_ADDRESS: event.value }))]
        },
        SET_SOURCE_CHANNEL: {
          actions: [assign(({ event }) => ({ SOURCE_CHANNEL: event.value }))]
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
              const sourceChannel = ucsConfiguration?.channel_id ?? raise("Channel not found")
              const relayContractAddress =
                ucsConfiguration?.contract_address ?? raise("Contract not found")

              return {
                sourceChainId,
                destinationChainId,
                PAYLOAD: {
                  amount,
                  network,
                  recipient,
                  denomAddress,
                  sourceChannel,
                  relayContractAddress,
                  path: [sourceChainId, destinationChainId]
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

export const transferStates = [
  "PRE_TRANSFER",
  "FLIPPING",
  "SWITCHING_TO_CHAIN",
  "APPROVING_ASSET",
  "AWAITING_APPROVAL_RECEIPT",
  "SIMULATING_TRANSFER",
  "CONFIRMING_TRANSFER",
  "AWAITING_TRANSFER_RECEIPT",
  "TRANSFERRING",
  "TRANSFERRED"
] as const

export type TransferState = (typeof transferStates)[number]

export interface TransferStates {
  //
}

export const transferAnimationMachine = setup({
  types: {
    input: {} as { defaultDelay: number },
    context: {} as {
      STEP_ORDER: number
      delay: number
      defaultDelay: number
      error: unknown | undefined
      warning: unknown | undefined
      data: string | undefined
    },
    events: {} as
      | { type: "reset" }
      | {
          data?: string
          error?: Error
          delay?: number
          type: "advance"
          warning?: Error
          target?: TransferState
        }
  }
}).createMachine({
  context: ({ input }) => ({
    STEP_ORDER: 0,
    data: undefined,
    error: undefined,
    warning: undefined,
    defaultDelay: input.defaultDelay,
    delay: input.defaultDelay
  }),
  id: "form",
  initial: "PRE_TRANSFER",
  states: {
    PRE_TRANSFER: {
      on: {
        advance: {
          target: "FLIPPING",
          actions: [
            assign(({ context, event }) => ({
              delay: event.delay ?? context.delay,
              STEP_ORDER: transferStates.indexOf("PRE_TRANSFER")
            })),
            ({ context, event }) => {
              console.log("Advancing from PRE_TRANSFER with delay:", event.delay)
            }
          ]
        }
      },
      description: "The initial state before the transfer process begins."
    },
    FLIPPING: {
      on: {
        advance: {
          after: {},
          target: "SWITCHING_TO_CHAIN",
          actions: [
            ({ context, event }) => {
              console.log("Advancing from FLIPPING with delay:", event.delay)
            },
            assign(({ context, event }) => ({
              delay: event.delay ?? context.delay,
              warning: event.warning ?? context.warning,
              STEP_ORDER: transferStates.indexOf("FLIPPING")
            }))
          ]
        }
      },
      description: "The state where the form animation flips."
    },
    SWITCHING_TO_CHAIN: {
      on: {
        advance: {
          target: "APPROVING_ASSET",
          actions: [
            ({ context, event }) => {
              console.log("Advancing from SWITCHING_TO_CHAIN with delay:", event.delay)
            },
            assign(({ context, event }) => ({
              delay: event.delay ?? context.delay,

              STEP_ORDER: transferStates.indexOf("SWITCHING_TO_CHAIN")
            }))
          ]
        }
      },
      description: "The state where the form switches to a different chain."
    },
    APPROVING_ASSET: {
      on: {
        advance: {
          target: "AWAITING_APPROVAL_RECEIPT",
          actions: [
            ({ context, event }) => {
              console.log("Advancing from APPROVING_ASSET with delay:", event.delay)
            },
            assign(({ context, event }) => ({
              delay: event.delay ?? context.delay,
              STEP_ORDER: transferStates.indexOf("APPROVING_ASSET")
            }))
          ]
        }
      },
      description: "The state where the asset is being approved."
    },
    AWAITING_APPROVAL_RECEIPT: {
      on: {
        advance: {
          target: "SIMULATING_TRANSFER",
          actions: [
            ({ context, event }) => {
              console.log("Advancing from AWAITING_APPROVAL_RECEIPT with delay:", event.delay)
            },
            assign(({ context, event }) => ({
              delay: event.delay ?? context.delay,
              error: event.error ?? context.error,
              data: event.data ?? context.data,
              STEP_ORDER: transferStates.indexOf("AWAITING_APPROVAL_RECEIPT")
            }))
          ]
        }
      },
      description: "The state where the form is waiting for the approval receipt."
    },
    SIMULATING_TRANSFER: {
      on: {
        advance: {
          target: "CONFIRMING_TRANSFER",
          actions: [
            ({ context, event }) => {
              console.log("Advancing from SIMULATING_TRANSFER with delay:", event.delay)
            },
            assign(({ context, event }) => ({
              delay: event.delay ?? context.delay,
              STEP_ORDER: transferStates.indexOf("SIMULATING_TRANSFER")
            }))
          ]
        }
      },
      description: "The state where the transfer is being simulated."
    },
    CONFIRMING_TRANSFER: {
      on: {
        advance: {
          target: "AWAITING_TRANSFER_RECEIPT",
          actions: [
            ({ context, event }) => {
              console.log("Advancing from CONFIRMING_TRANSFER with delay:", event.delay)
            },
            assign(({ context, event }) => ({ delay: event.delay ?? context.delay }))
          ]
        }
      },
      description: "The state where the transfer is being confirmed."
    },
    AWAITING_TRANSFER_RECEIPT: {
      on: {
        advance: {
          target: "TRANSFERRING",
          actions: [
            ({ context, event }) => {
              console.log("Advancing from AWAITING_TRANSFER_RECEIPT with delay:", event.delay)
            },
            assign(({ context, event }) => ({
              delay: event.delay ?? context.delay,
              error: event.error ?? context.error,
              data: event.data ?? context.data,
              STEP_ORDER: transferStates.indexOf("AWAITING_TRANSFER_RECEIPT")
            }))
          ]
        }
      },
      description: "The state where the form is waiting for the transfer receipt."
    },
    TRANSFERRING: {
      on: {
        advance: {
          target: "TRANSFERRED",
          actions: [
            ({ context, event }) => {
              console.log("Advancing from TRANSFERRING with delay:", event.delay)
            },
            assign(({ context, event }) => ({
              delay: event.delay ?? context.delay,
              STEP_ORDER: transferStates.indexOf("TRANSFERRING")
            }))
          ]
        }
      },
      description: "The state where the asset is being transferred."
    },
    TRANSFERRED: {
      on: {
        reset: {
          target: "PRE_TRANSFER",
          actions: [
            ({ context, event }) => {
              console.log("Resetting form to PRE_TRANSFER")
            },
            assign({ delay: () => 0, STEP_ORDER: transferStates.indexOf("PRE_TRANSFER") })
          ]
        }
      },
      description: "The final state where the asset has been successfully transferred."
    }
  }
})

// export const transferAnimationMachine = setup({
//   actors: {
//     sleep: fromPromise(async ({ input, self, signal, system, emit }) => await sleep(7_000))
//   },

//   delays: {
//     sleep: ({ context, event }) => {
//       console.info(`delaying for ${event.type === "sleep" ? event.value.delay ?? 0 : 0}ms`)
//       return event.type === "sleep" ? event.value.delay ?? 0 : 0
//     }
//   },
//   types: {
//     input: {} as { delay?: number },
//     events: {} as
//       | { type: "reset" }
//       | { type: "sleep"; value: { delay?: number } }
//       | { type: "nextStep"; value: { state: TransferState } },
//     context: {} as {
//       delay: number
//       ORDER: number
//       STATE: TransferState
//       error: unknown | undefined
//       LAST_EVENT: "reset" | "sleep" | "nextStep"
//     }
//   },
//   guards: {
//     IS_SLEEP_EVENT: ({ event }) => event.type === "sleep"
//   }
// }).createMachine({
//   initial: "idle",
//   id: "transfer-animation",
//   context: ({ input, self, spawn }) => ({
//     ORDER: 0,
//     error: undefined,
//     LAST_EVENT: "reset",
//     STATE: "PRE_TRANSFER",

//     delay: input?.delay ?? 0
//   }),
//   // on: {
//   //   "*": {
//   //     actions: async ({ context, event, self, system }) => {
//   //       if (typeof event?.value.delay === "number") await sleep(event.value.delay)
//   //       return context
//   //     }
//   //   }
//   // },
//   on: {
//     reset: {
//       target: ".idle"
//     },
//     nextStep: {
//       actions: [
//         assign(({ event }) => ({
//           delay: 0,
//           LAST_EVENT: "nextStep",
//           STATE: event.value.state,
//           ORDER: transferStates.indexOf(event.value.state)
//         }))
//       ]
//     },
//     sleep: {
//       actions: [assign(({ event }) => ({ LAST_EVENT: "sleep", delay: event.value.delay }))]
//       // guard: {
//       //   type: "IS_SLEEP_EVENT",
//       //   params: {}
//       // }
//     }
//   },
//   always: {
//     actions: assign(({ context, event, self, system }) => {
//       console.info(context, event, self, system)
//       return context
//     })
//   },
//   states: {
//     idle: {},
//     animate: {
//       tags: ["animate"],
//       id: "#animation-machine-animate",
//       on: {
//         sleep: {
//           actions: assign(({ event }) => ({ LAST_EVENT: "sleep", delay: event.value.delay }))
//         },
//         nextStep: {
//           actions: [
//             assign(({ event }) => ({
//               delay: 0,
//               LAST_EVENT: "nextStep",
//               STATE: event.value.state,
//               ORDER: transferStates.indexOf(event.value.state)
//             }))
//           ]
//         }
//       }
//     }
//     // sleep: {
//     //   tags: ["sleep"],
//     //   id: "#animation-machine-sleep"
//     //   // invoke:
//     //   // after: {
//     //   //   sleep: {
//     //   //     target: "nextStep",
//     //   //     // actions: _ => (_.type === "sleep" ? _.event.value.delay : 0)
//     //   //   }
//     //   // }
//     // }
//   }
// })

// // const actor = createActor(transferStateMachine)
// // const subscription = actor.subscribe({
// //   next: snapshot =>
// //     snapshot.hasTag("success")
// //       ? console.info(snapshot.context.client?.getBlockNumber().then(console.log))
// //       : false
// // })
// // actor.start()

// // actor.send({ type: "SET_SOURCE_CHAIN", value: { chainId: "80084", network: "evm" } })
// // actor.send({ type: "SET_DESTINATION_CHAIN", value: "stride-internal-1" })
// // actor.send({
// //   type: "SET_DENOM_ADDRESS_AND_SYMBOL",
// //   value: { denomAddress: "0x0E4aaF1351de4c0264C5c7056Ef3777b41BD8e03", symbol: "HONEY" }
// // })
// // actor.send({ type: "SET_AMOUNT", value: 1n })
// // actor.send({ type: "CONSTRUCT_PAYLOAD" })
// // actor.send({ type: "SUCCESS" })
// // actor.send({
// //   type: "SET_EVM_CLIENT_PARAMETERS",
// //   value: {
// //     transport: fallback([http(sepolia.rpcUrls.default.http.at(0))])
// //   }
// // })
// // actor.send({
// //   type: "SET_CLIENT"
// // })
