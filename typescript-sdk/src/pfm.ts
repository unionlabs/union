import type { ChainId } from "./types.ts"
import { err, ok, Result } from "neverthrow"
import { sepolia } from "./mod.ts"
import { holesky } from "viem/chains"

export const createPfmMemo: (_args: {
  port: string
  channel: string
  receiver: string
}) => Result<string, Error> = Result.fromThrowable(
  ({
    port,
    channel,
    receiver
  }: {
    port: string
    channel: string
    receiver: string
  }): string =>
    JSON.stringify({
      forward: {
        port,
        channel,
        receiver: receiver.startsWith("0x") ? receiver.slice(2) : receiver
      }
    }),
  error => new Error("Failed to create PFM memo", { cause: error })
)

/** Temporarily mocked
 */
export async function getHubbleChainDetails({
  sourceChainId,
  destinationChainId
}: {
  sourceChainId: ChainId | (string & {})
  destinationChainId: ChainId | (string & {})
}): Promise<
  Result<
    {
      port?: string
      sourceChannel: number
      destinationChannel: number
      relayContractAddress: string
      destinationUCS03Address: string
      transferType: "direct" | "pfm"
    },
    Error
  >
> {
  // if (sourceChainId === destinationChainId) {
  //   return err(new Error("Source and destination chains cannot be the same"))
  // }

  // const { data: chains } = await offchainQuery.chains({
  //   includeContracts: true,
  //   includeEndpoints: true
  // })

  /** Will be moved to hubble soon.
   */
  const CHAINS = [
    {
      testnet: true,
      chain_id: sepolia.id.toString(),
      rpc_type: "evm",
      addr_prefix: "0x",
      display_name: "Sepolia",
      ucs3_config: {
        address: "0x84F074C15513F15baeA0fbEd3ec42F0Bd1fb3efa",
        channels: {
          [holesky.id.toString()]: 4
        }
      }
    },
    {
      testnet: true,
      chain_id: holesky.id.toString(),
      rpc_type: "evm",
      addr_prefix: "0x",
      display_name: "Holesky",
      ucs3_config: {
        address: "0x7b7872fec715c787a1be3f062adedc82b3b06144",
        channels: {
          [sepolia.id.toString()]: 9,
          "union-testnet-9": 8,
          "elgafar-1": 11
        }
      }
    },
    {
      testnet: true,
      chain_id: "union-testnet-9",
      rpc_type: "cosmos",
      addr_prefix: "union",
      display_name: "Union Testnet 9",
      ucs3_config: {
        address: "union19hspxmypfxsdsnxttma8rxvp7dtcmzhl9my0ee64avg358vlpawsdvucqa",
        channels: {
          [holesky.id.toString()]: 7
        }
      }
    },
    {
      testnet: true,
      chain_id: "elgafar-1",
      rpc_type: "cosmos",
      addr_prefix: "stars",
      display_name: "Stargaze Testnet",
      ucs3_config: {
        address: "stars1vv5v4sk4tzxs9a0685j4shdqazj44dla8rfu6np40h9tneuruq0s3rs6kq",
        channels: {
          [holesky.id.toString()]: 15
        }
      }
    }
  ]

  const sourceChain = CHAINS.find(c => c.chain_id === sourceChainId)
  const destinationChain = CHAINS.find(c => c.chain_id === destinationChainId)
  // const transferType = "direct"

  if (!sourceChain) return err(new Error("source chain not found in hubble"))
  if (!destinationChain) return err(new Error("destination chain not found in hubble"))

  const sourceChannel = sourceChain.ucs3_config.channels[destinationChainId]
  const destinationChannel = destinationChain.ucs3_config.channels[sourceChainId]

  if (!sourceChannel)
    return err(new Error(`no source channel to go from ${sourceChainId} to ${destinationChainId}`))
  if (!destinationChannel)
    return err(
      new Error(`no destination channel to go from ${sourceChainId} to ${destinationChainId}`)
    )

  // const checkAgainst = sourceChainId === "union-testnet-8" ? destinationChainId : "union-testnet-8"
  // const ucsConfiguration = chain.ucs1_configurations
  //   ?.filter(config => config.destination_chain.chain_id === checkAgainst)
  //   .at(0)

  // if (!ucsConfiguration) return err(new Error("UCS configuration not found"))

  // if (transferType === "direct") {
  return ok({
    transferType: "direct",
    sourceChannel,
    destinationChannel,
    relayContractAddress: sourceChain.ucs3_config.address,
    destinationUCS03Address: destinationChain.ucs3_config.address
  })
  // }

  // const forward = ucsConfiguration.forwards.find(
  //   item => item.destination_chain.chain_id === destinationChainId
  // )

  // if (!forward) return err(new Error("Forward configuration not found"))
  // return ok({
  //   transferType,
  //   port: forward.port_id,
  //   destinationChannel: forward.channel_id,
  //   sourceChannel: ucsConfiguration.channel_id,
  //   relayContractAddress: ucsConfiguration.contract_address,
  //   destinationChainId: ucsConfiguration.destination_chain.chain_id
  // })
}
