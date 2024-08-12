import { setup, assign, fromPromise } from "xstate"
import {
  cosmosHttp,
  offchainQuery,
  type OfflineSigner,
  hexAddressToBech32,
  createCosmosSdkClient,
  type EvmClientParameters,
  type CosmosClientParameters,
  type TransferAssetsParameters
} from "@union/client"
import { get } from "svelte/store"
import { raise } from "$lib/utilities"
import { cosmosStore } from "$lib/wallet/cosmos"
import type { ChainWalletStore } from "$lib/wallet/types"
import { sepoliaStore, wagmiConfig } from "$lib/wallet/evm"

type Network = "cosmos" | "evm"

type HubbleChain = Awaited<ReturnType<typeof offchainQuery.chain>>["data"][number]
type UcsConfiguration = {
  NETWORK: Network
  FORWARD: Pick<
    NonNullable<HubbleChain["ucs1_configurations"]>[number]["forward"][number],
    "channel_id" | "port"
  >
  SOURCE_CHANNEL: NonNullable<HubbleChain["ucs1_configurations"]>[number]["channel_id"]
  RELAY_CONTRACT_ADDRESS: NonNullable<
    HubbleChain["ucs1_configurations"]
  >[number]["contract_address"]
  PATH: [string, string]
}
const offChainPromiseLogic = fromPromise<
  UcsConfiguration,
  {
    SOURCE_CHAIN_ID: string
    DESTINATION_CHAIN_ID: string
  }
>(async ({ input, self, signal, system }) => {
  const {
    data: [sourceChainInfo]
  } = await offchainQuery.chain({
    includeContracts: true,
    includeEndpoints: true,
    chainId: input.SOURCE_CHAIN_ID
  })

  const ucsConfiguration = sourceChainInfo.ucs1_configurations
    ?.filter(config => config.destination_chain.chain_id === "union-testnet-8")
    .at(0)
  if (!ucsConfiguration) return raise("UCS configuration not found")

  const forward = ucsConfiguration.forward.find(
    item => item.destination_chain.chain_id === input.DESTINATION_CHAIN_ID
  )
  if (!forward) return raise("Forward configuration not found")
  return {
    NETWORK: sourceChainInfo.rpc_type,
    SOURCE_CHANNEL: ucsConfiguration.channel_id,
    RELAY_CONTRACT_ADDRESS: ucsConfiguration.contract_address,
    FORWARD: { channel_id: forward.channel_id, port: forward.port },
    RPC_URL: sourceChainInfo.rpcs?.find(rpc => rpc.type === "rpc")?.url,
    PATH: [ucsConfiguration.source_chain.chain_id, ucsConfiguration.destination_chain.chain_id]
  }
})

export const transferStateMachine = setup({
  actors: {
    fetchUcsConfiguration: offChainPromiseLogic,
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
    context: {} as {
      error: unknown
      AMOUNT: bigint | undefined
      RPC_URL: string | undefined
      NETWORK: Network | undefined
      RECIPIENT: string | undefined
      DENOM_ADDRESS: string | undefined
      PATH: [string, string] | undefined
      SOURCE_CHANNEL: string | undefined
      SOURCE_CHAIN_ID: string | undefined
      sepoliaStore: ChainWalletStore<"evm">
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
          value: {
            chainId: string
            network: Network
          }
        }
      | { type: "SET_DESTINATION_CHAIN"; value: string }
      | { type: "SET_DENOM_ADDRESS"; value: string }
      | { type: "SET_AMOUNT"; value: bigint }
      | { type: "CONSTRUCT_PAYLOAD" }
      | { type: "SUCCESS" }
      | { type: "SWITCH_CHAIN"; value: string }
      | { type: "SET_RECIPIENT" }
      | { type: "APPROVE_TRANSFER" }
      | { type: "TRANSFER_ASSET" }
      | { type: "RECEIPT_RECEIVED" }
      | { type: "SHOW_RECEIPT" }
      | { type: "APPROVE_SPEND" }
      | { type: "SET_CLIENT" }
      | { type: "APPROVAL_RECEIPT_RECEIVED" }
      | { type: "TRANSFER_SIMULATION_APPROVED" }
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
  context: () => ({
    PATH: undefined,
    error: undefined,
    AMOUNT: undefined,
    PAYLOAD: undefined,
    NETWORK: undefined,
    RPC_URL: undefined,
    RECIPIENT: undefined,
    DENOM_ADDRESS: undefined,
    SOURCE_CHANNEL: undefined,
    SOURCE_CHAIN_ID: undefined,
    cosmosStore: get(cosmosStore),
    sepoliaStore: get(sepoliaStore),
    DESTINATION_CHAIN_ID: undefined,
    RELAY_CONTRACT_ADDRESS: undefined,
    evmClientParameters: undefined,
    cosmosClientParameters: undefined,
    client: undefined
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
          actions: assign(({ event }) => ({ DESTINATION_CHAIN_ID: event.value }))
        },
        SET_DENOM_ADDRESS: {
          actions: assign(({ event }) => ({ DENOM_ADDRESS: event.value }))
        },
        SET_AMOUNT: {
          actions: assign(({ event }) => ({ AMOUNT: event.value }))
        },
        SET_EVM_CLIENT_PARAMETERS: {
          guard: "IS_EVM",
          target: "SET_CLIENT",
          actions: assign(({ event, context }) => ({
            evmClientParameters: {
              account: event.value.account || wagmiConfig.getClient().account,
              transport: event.value.transport || wagmiConfig.getClient().transport,
              chain:
                wagmiConfig.chains.find(chain => chain.id === Number(context.SOURCE_CHAIN_ID)) ||
                event.value.chain ||
                wagmiConfig.getClient().chain
            }
          }))
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
      onDone: {
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
    },
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
          target: "CONSTRUCT_PAYLOAD",
          actions: assign({ client: ({ event }) => event.output })
        }
      }
    },
    CONSTRUCT_PAYLOAD: {
      tags: ["construct-payload"],
      id: "#TRANSFER-MACHINE-CONSTRUCT-PAYLOAD",
      output: ({ context }) => context.PAYLOAD,
      invoke: {
        src: "fetchUcsConfiguration",
        id: "#FETCH-UCS-CONFIGURATION",
        systemId: "#TRANSFER-MACHINE-CONSTRUCT-PAYLOAD",
        input: ({ context }) => ({
          SOURCE_CHAIN_ID: context.SOURCE_CHAIN_ID ?? raise("Source chain not found"),
          DESTINATION_CHAIN_ID: context.DESTINATION_CHAIN_ID ?? raise("Destination chain not found")
        }),
        onDone: {
          target: "SUCCESS",
          actions: assign({
            PAYLOAD: ({ event, context }) => {
              const recipient = Number.isInteger(context.SOURCE_CHAIN_ID)
                ? context.RECIPIENT
                : hexAddressToBech32({
                    address: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd",
                    bech32Prefix: "stride"
                  })
              return {
                path: event.output.PATH,
                network: event.output.NETWORK,
                sourceChannel: event.output.SOURCE_CHANNEL,
                amount: context.AMOUNT ?? raise("Amount not found"),
                recipient: recipient ?? raise("Recipient not found"),
                relayContractAddress: event.output.RELAY_CONTRACT_ADDRESS,
                denomAddress: context.DENOM_ADDRESS ?? raise("Denom address not found")
              }
            }
          })
        }
      }
    },
    SUCCESS: {
      tags: ["success"],
      // always: {
      //   actions: _ => {
      //     console.info(JSON.stringify({ _ }, undefined, 2))
      //   }
      // },
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

// actor.send({ type: "SET_SOURCE_CHAIN", value: { chainId: "11155111", network: "evm" } })
// actor.send({ type: "SET_DESTINATION_CHAIN", value: "stride-internal-1" })
// actor.send({ type: "SET_DENOM_ADDRESS", value: "0x0E4aaF1351de4c0264C5c7056Ef3777b41BD8e03" })
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
