import {
  erc20Abi,
  getAddress,
  createClient,
  type Address,
  type Account,
  walletActions,
  publicActions,
  type createWalletClient
} from "viem"
import type { OfflineSigner } from "./types.ts"
import type { GasPrice } from "@cosmjs/stargate"
import { bech32AddressToHex } from "./convert.ts"
import { ucs01RelayAbi } from "./abi/ucs01-relay.ts"
import type { ChainId } from "./constants/testnet.ts"
import { raise, timestamp } from "./utilities/index.ts"
import { type cosmosHttp, rankCosmosRpcProviders } from "./transport.ts"
import { cosmosTransfer, cosmwasmTransfer, ibcTransfer } from "./transfer.ts"

export type EvmClientParameters = Parameters<typeof createWalletClient>[0]
export interface CosmosClientParameters {
  account: OfflineSigner
  transport: ReturnType<typeof cosmosHttp> | Array<ReturnType<typeof cosmosHttp>>
  gasPrice?: GasPrice
}

export function createUnionClient({
  evm,
  cosmos
}: {
  evmRpcUrl?: string
  cosmosRpcUrl?: string
  evmAccount?: Account
  cosmosSigner?: OfflineSigner
  evm: EvmClientParameters
  cosmos: CosmosClientParameters
}) {
  return createClient(evm)
    .extend(walletActions)
    .extend(publicActions)
    .extend(client => ({
      async transferAssetFromEvm({
        sourceChainId,
        evmAccount = client.account,
        receiver,
        sourceChannel,
        amount,
        denomAddress,
        relayContractAddress,
        simulate = true
      }: {
        sourceChainId: string
        evmAccount?: Account
        receiver: string
        sourceChannel: string
        amount: bigint
        denomAddress: Address
        relayContractAddress: Address
        simulate?: boolean
      }) {
        if (sourceChainId === "11115511" && !evmAccount) raise("EVM account not found")
        if (!evmAccount) raise("EVM account not found")
        const approve = await client.writeContract({
          abi: erc20Abi,
          account: evmAccount,
          chain: client.chain,
          address: denomAddress,
          functionName: "approve",
          args: [relayContractAddress, amount]
        })
        if (!approve) raise("Failed to approve")

        const writeContractParameters = {
          account: evmAccount,
          abi: ucs01RelayAbi,
          chain: client.chain,
          /**
           * @dev `send` function of UCS01 contract: https://github.com/unionlabs/union/blob/1b9e4a6551163e552d85405eb70917fdfdc14b55/evm/contracts/apps/ucs/01-relay/Relay.sol#L50-L56
           */
          functionName: "send",
          address: relayContractAddress,
          /**
           * string calldata sourceChannel,
           * bytes calldata receiver,
           * LocalToken[] calldata tokens,
           * IbcCoreClientV1Height.Data calldata timeoutHeight,
           * uint64 timeoutTimestamp
           */
          args: [
            sourceChannel,
            bech32AddressToHex({ address: receiver }),
            [{ denom: denomAddress, amount }],
            { revision_number: 9n, revision_height: BigInt(999_999_999) + 100n },
            0n
          ]
        } as const
        if (!simulate) {
          const hash = await client.writeContract({
            abi: erc20Abi,
            account: evmAccount,
            chain: client.chain,
            address: relayContractAddress,
            functionName: "transfer",
            args: [getAddress(receiver), amount]
          })
          return hash
        }
        const { request } = await client.simulateContract(writeContractParameters)

        const hash = await client.writeContract(request)
        return hash
      }
    }))
    .extend(client => {
      const evmAccount = client.account
      const _cosmosSigner = cosmos.account
      const _gasPrice = cosmos.gasPrice
      return {
        async transferAsset({
          network,
          receiver,
          amount,
          path,
          sourceChannel,
          denomAddress,
          relayContractAddress,
          evmSigner = evmAccount,
          cosmosSigner = _cosmosSigner,
          // cosmosRpcUrl,
          gasPrice = _gasPrice
        }: {
          network: "cosmos" | "evm"
          path: [ChainId, ChainId]
          receiver: string
          amount: bigint
          sourceChannel: string
          denomAddress: string
          relayContractAddress: string
          evmSigner?: Account
          cosmosSigner?: OfflineSigner
          // cosmosRpcUrl?: string
          gasPrice?: GasPrice
        }): Promise<string> {
          if (!path.includes("union-testnet-8")) {
            raise(
              "Either source or destination chain ID is not union-testnet-8. Must be union-testnet-8 until PFM is implemented"
            )
          }
          const [sourceChainId, destinationChainId] = path

          if (network === "evm") {
            if (!evmSigner) raise("EVM signer not found")
            const transactionHash = await client.transferAssetFromEvm({
              sourceChainId,
              evmAccount: evmSigner,
              receiver,
              sourceChannel,
              amount,
              denomAddress: getAddress(denomAddress),
              relayContractAddress: getAddress(relayContractAddress),
              simulate: true
            })
            return transactionHash
          }

          const cosmosRpcTransport = await rankCosmosRpcProviders({
            transports: Array.isArray(cosmos.transport)
              ? cosmos.transport.flatMap(t => t({}).value?.url).filter(Boolean)
              : [cosmos.transport({}).value?.url].filter(Boolean),
            interval: 1_000,
            sampleCount: 10,
            timeout: 1_000
          }).rank()
          const cosmosRpcUrl = cosmosRpcTransport.at(0)?.rpcUrl

          if (!cosmosSigner) raise("Cosmos signer not found")
          if (!cosmosRpcUrl) raise("Cosmos RPC URL not found")
          if (!gasPrice) raise("Gas price not found")

          if (sourceChainId && destinationChainId === "union-testnet-8") {
            const transfer = await cosmosTransfer({
              receiver,
              cosmosSigner,
              cosmosRpcUrl,
              gasPrice,
              asset: { denom: denomAddress, amount: amount.toString() }
            })
            return transfer.transactionHash
          }

          const stamp = timestamp()

          if (network === "cosmos" && sourceChainId === "union-testnet-8") {
            const transfer = await cosmwasmTransfer({
              cosmosSigner,
              cosmosRpcUrl,
              gasPrice,
              instructions: [
                {
                  contractAddress: relayContractAddress,
                  msg: {
                    transfer: {
                      channel: sourceChannel,
                      receiver: receiver.startsWith("0x") ? receiver.slice(2) : receiver,
                      memo: `${stamp} Sending ${amount} ${denomAddress} to ${receiver}`
                    }
                  },
                  funds: [{ amount: amount.toString(), denom: denomAddress }]
                }
              ]
            })
            return transfer.transactionHash
          }

          if (network === "cosmos" && destinationChainId === "union-testnet-8") {
            const [account] = await cosmosSigner.getAccounts()
            if (!account) raise("No cosmos signer account found")
            ibcTransfer({
              cosmosSigner,
              cosmosRpcUrl,
              gasPrice,
              messageTransfers: [
                {
                  sourceChannel,
                  sourcePort: "transfer",
                  token: { denom: denomAddress, amount: amount.toString() },
                  sender: account.address,
                  receiver,
                  timeoutHeight: { revisionHeight: 888888888n, revisionNumber: 8n },
                  memo: `${stamp} Sending ${amount} ${denomAddress} to ${receiver}`
                }
              ]
            })
          }

          throw new Error("Invalid network")
        }
      }
    })
}
