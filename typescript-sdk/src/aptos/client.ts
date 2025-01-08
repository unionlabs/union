import {
  type AptosAccount,
  aptosTransferSimulate,
  aptosSameChainTransfer,
  transferAssetFromAptos,
  waitForTransactionReceipt,
  type AptosPublicAccountInfo
} from "./transfer.ts"
import { err, type Result } from "neverthrow"
import { bech32AddressToHex } from "../convert.ts"
import { cosmosChainId } from "../cosmos/client.ts"
import type { TransferAssetsParameters } from "../types.ts"
import { Aptos, Network, AptosConfig } from "@aptos-labs/ts-sdk"
import { createPfmMemo, getHubbleChainDetails } from "../pfm.ts"
import { createClient, fallback, type HttpTransport } from "viem"
import type { AptosBrowserWallet, AuthAccess } from "./wallet.ts"

export type { AptosAccount, AptosBrowserWallet }

export const aptosChainId = ["2"] as const
export type AptosChainId = `${(typeof aptosChainId)[number]}`

/**
 * This is kinda the same way creating a wallet client in viem works
 * @see https://viem.sh/docs/clients/wallet#1-initialize-a-wallet-client
 */

type AptosWindowTransport = AptosBrowserWallet

export type AptosClientParameters = {
  chainId: AptosChainId
} & (
  | { account: AptosAccount; transport: HttpTransport }
  | { account?: AptosPublicAccountInfo; transport: AptosWindowTransport }
)

async function getAptosClient(
  parameters: AptosClientParameters & { authAccess: "key" }
): Promise<{ authAccess: "key"; aptos: Aptos; signer: AptosAccount }>

async function getAptosClient(
  parameters: AptosClientParameters & { authAccess: "wallet" }
): Promise<{ authAccess: "wallet"; aptos: Aptos; signer: AptosBrowserWallet }>

async function getAptosClient(
  parameters: AptosClientParameters & { authAccess: AuthAccess }
): Promise<
  | { authAccess: "key"; aptos: Aptos; signer: AptosAccount }
  | { authAccess: "wallet"; aptos: Aptos; signer: AptosBrowserWallet }
> {
  if (parameters.authAccess === "key") {
    if (typeof parameters.transport !== "function") throw new Error("Invalid Aptos transport")
    const rpcUrl = parameters.transport({}).value?.url
    if (!rpcUrl) throw new Error("No Aptos RPC URL found")
    const config = new AptosConfig({ fullnode: rpcUrl, network: Network.TESTNET })
    return {
      authAccess: "key",
      aptos: new Aptos(config),
      signer: parameters.account as AptosAccount
    }
  }

  if (parameters.authAccess === "wallet") {
    if (typeof parameters.transport !== "object") throw new Error("Invalid Aptos transport")
    const networkInfo = await parameters.transport.getNetwork()
    const network = networkInfo.name.toLowerCase() === "mainnet" ? Network.MAINNET : Network.TESTNET
    const config = new AptosConfig({ fullnode: networkInfo.url, network })
    return {
      authAccess: "wallet",
      aptos: new Aptos(config),
      signer: parameters.transport as AptosBrowserWallet
    }
  }
  throw new Error("Invalid Aptos transport")
}

export const createAptosClient = (clientParameters: AptosClientParameters) => {
  return (
    createClient({ transport: fallback([]) })

      .extend(_ => ({
        getAptosClient: async ({ authAccess }: { authAccess: AuthAccess }) =>
          authAccess === "key"
            ? await getAptosClient({ ...clientParameters, authAccess: "key" })
            : await getAptosClient({ ...clientParameters, authAccess: "wallet" })
      }))
      //
      .extend(client => ({
        waitForTransactionReceipt: async ({ hash }: { hash: string }) => {
          const { aptos, signer, authAccess } = await client.getAptosClient({ authAccess: "key" })
          return await waitForTransactionReceipt({ aptos, hash })
        },
        //
        transferAsset: async (
          transferParameters: TransferAssetsParameters<AptosChainId>
        ): Promise<Result<string, Error>> => {
          const aptosClient = await client.getAptosClient({
            authAccess: transferParameters.authAccess
          })

          let {
            memo,
            amount,
            simulate,
            receiver,
            denomAddress,
            destinationChainId,
            relayContractAddress
          } = transferParameters

          if (!destinationChainId) return err(new Error("destinationChainId missing"))

          if (clientParameters.chainId === destinationChainId) {
            return await aptosSameChainTransfer({
              ...transferParameters,
              amount,
              simulate,
              receiver,
              denomAddress,
              ...aptosClient
            })
          }

          const chainDetails = await getHubbleChainDetails({
            destinationChainId,
            sourceChainId: clientParameters.chainId
          })
          if (chainDetails.isErr()) return err(chainDetails.error)

          if (chainDetails.value.transferType === "pfm") {
            if (!chainDetails.value.port) return err(new Error("Port not found in hubble"))
            const pfmMemo = createPfmMemo({
              port: chainDetails.value.port,
              channel: chainDetails.value.destinationChannel,
              /**
               * TODO:
               * check if normal Aptos hex address is valid here or do we need to do some transformation
               */
              receiver: cosmosChainId.includes(destinationChainId)
                ? bech32AddressToHex({ address: receiver })
                : receiver
            })
            if (pfmMemo.isErr()) return err(pfmMemo.error)
            memo = pfmMemo.value
          }

          const sourceChannel = chainDetails.value.sourceChannel
          relayContractAddress ??= chainDetails.value.relayContractAddress

          return await transferAssetFromAptos({
            ...transferParameters,
            memo,
            amount,
            simulate,
            receiver,
            denomAddress,
            sourceChannel,
            destinationChainId,
            relayContractAddress,
            ...aptosClient
          })
        },
        simulateTransaction: async (
          transferParameters: TransferAssetsParameters<AptosChainId>
        ): Promise<Result<string, Error>> => {
          const aptosClient = await client.getAptosClient({
            authAccess: transferParameters.authAccess
          })

          let {
            memo,
            amount,
            receiver,
            denomAddress,
            autoApprove: _,
            destinationChainId,
            relayContractAddress
          } = transferParameters

          if (!destinationChainId) return err(new Error("destinationChainId missing"))

          if (clientParameters.chainId === destinationChainId) {
            return await aptosTransferSimulate({
              path: "SAME_CHAIN",
              simulate: true,
              aptos: aptosClient.aptos,
              signer: aptosClient.signer,
              ...transferParameters
            })
          }

          const chainDetails = await getHubbleChainDetails({
            destinationChainId,
            sourceChainId: clientParameters.chainId
          })
          if (chainDetails.isErr()) return err(chainDetails.error)

          if (chainDetails.value.transferType === "pfm") {
            if (!chainDetails.value.port) return err(new Error("Port not found in hubble"))
            const pfmMemo = createPfmMemo({
              port: chainDetails.value.port,
              channel: chainDetails.value.destinationChannel,
              /**
               * TODO:
               * check if normal Aptos hex address is valid here or do we need to do some transformation
               */
              receiver: cosmosChainId.includes(destinationChainId)
                ? bech32AddressToHex({ address: receiver })
                : receiver
            })
            if (pfmMemo.isErr()) return err(pfmMemo.error)
            memo = pfmMemo.value
          }
          const sourceChannel = chainDetails.value.sourceChannel
          relayContractAddress ??= chainDetails.value.relayContractAddress

          return await aptosTransferSimulate({
            ...transferParameters,
            path: "CROSS_CHAIN",
            memo,
            amount,
            receiver,
            denomAddress,
            sourceChannel,
            destinationChainId,
            relayContractAddress,
            ...aptosClient
          })
        }
      }))
  )
}
