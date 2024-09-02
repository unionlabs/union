import { err, ok, Result } from "neverthrow"
import type { ChainId } from "./client/types.ts"
import { offchainQuery } from "./query/offchain/hubble.ts"

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
      sourceChannel: string
      // memo?: string | undefined
      destinationChannel: string
      destinationChainId: ChainId
      relayContractAddress: string
      transferType: "direct" | "pfm"
    },
    Error
  >
> {
  if (sourceChainId === destinationChainId) {
    return err(new Error("Source and destination chains cannot be the same"))
  }

  const transferType = [sourceChainId, destinationChainId].includes("union-testnet-8")
    ? "direct"
    : "pfm"

  const {
    data: [data]
  } = await offchainQuery.chain({
    chainId: sourceChainId,
    includeContracts: true,
    includeEndpoints: true
  })

  if (!data) return err(new Error("Chain not found in hubble"))

  const ucsConfiguration = data.ucs1_configurations
    ?.filter(config => config.destination_chain.chain_id === "union-testnet-8")
    .at(0)

  if (!ucsConfiguration) return err(new Error("UCS configuration not found"))

  if (transferType === "direct") {
    return ok({
      transferType,
      memo: undefined,
      sourceChannel: ucsConfiguration.channel_id,
      destinationChannel: ucsConfiguration.channel_id,
      relayContractAddress: ucsConfiguration.contract_address,
      destinationChainId: ucsConfiguration.destination_chain.chain_id
    })
  }

  const forward = ucsConfiguration.forward.find(
    item => item.destination_chain.chain_id === destinationChainId
  )

  if (!forward) return err(new Error("Forward configuration not found"))
  return ok({
    transferType,
    port: forward.port,
    destinationChannel: forward.channel_id,
    sourceChannel: ucsConfiguration.channel_id,
    relayContractAddress: ucsConfiguration.contract_address,
    destinationChainId: ucsConfiguration.destination_chain.chain_id
  })
}
