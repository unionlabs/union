export * from "./v0/mod.ts"
import {
  erc20Abi,
  getAddress,
  createClient,
  type Address,
  type Account,
  walletActions,
  publicActions,
  type WalletClientConfig
} from "viem"
import {
  bech32AddressToHex,
  hexAddressToBech32,
  hexStringToUint8Array,
  uint8ArrayToHexString,
  convertByteArrayToHex
} from "./convert.ts"
import { GasPrice } from "@cosmjs/stargate"
import type { OfflineSigner } from "./types.ts"
import { offchainQuery } from "#query/off-chain.ts"
import { ucs01RelayAbi } from "./abi/ucs01-relay.ts"
import { raise, timestamp } from "./utilities/index.ts"
import { type cosmosHttp, rankCosmosRpcProviders } from "./transport.ts"
import { cosmosTransfer, cosmwasmTransfer, ibcTransfer } from "./transfer.ts"
import { truncateAddress, isValidEvmAddress, isValidBech32Address } from "./utilities/address.ts"

/**
 * We export this as a standalone so that it can be used to fetch data that get passed to `createUnionClient`
 */
export { offchainQuery }

export interface EvmClientParameters extends WalletClientConfig {}

export interface CosmosClientParameters {
  account: OfflineSigner
  transport: ReturnType<typeof cosmosHttp> | Array<ReturnType<typeof cosmosHttp>>
  gasPrice?: { amount: string; denom: string }
}

export function createUnionClient({
  evm,
  cosmos
}: {
  evm: EvmClientParameters
  cosmos: CosmosClientParameters
}) {
  return createClient(evm)
    .extend(walletActions)
    .extend(publicActions)
    .extend(() => ({ offchainQuery }))
    .extend(() => ({
      bech32AddressToHex,
      hexAddressToBech32,
      convertByteArrayToHex,
      hexStringToUint8Array,
      uint8ArrayToHexString,
      truncateAddress,
      isValidEvmAddress,
      isValidBech32Address
    }))
    .extend(client => ({
      async transferAssetFromEvm({
        sourceChainId,
        evmAccount = client.account,
        receiver,
        sourceChannel,
        amount,
        denomAddress,
        relayContractAddress,
        simulate = true,
        memo = timestamp()
      }: {
        sourceChainId: string
        evmAccount?: Account
        receiver: string
        sourceChannel: string
        amount: bigint
        denomAddress: Address
        relayContractAddress: Address
        simulate?: boolean
        memo?: string
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
           * @dev `send` function of UCS01 contract: https://github.com/unionlabs/union/blob/142e0af66a9b0218cf010e3f8d1138de9b778bb9/evm/contracts/apps/ucs/01-relay/Relay.sol#L51-L58
           */
          functionName: "send",
          address: relayContractAddress,
          /**
           * string calldata sourceChannel,
           * bytes calldata receiver,
           * LocalToken[] calldata tokens,
           * string calldata extension (memo),
           * IbcCoreClientV1Height.Data calldata timeoutHeight,
           * uint64 timeoutTimestamp
           */
          args: [
            sourceChannel,
            bech32AddressToHex({ address: receiver }),
            [{ denom: denomAddress, amount }],
            memo,
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
          gasPrice = _gasPrice,
          memo = timestamp()
        }: {
          network: "cosmos" | "evm"
          path: [string, string]
          receiver: string
          amount: bigint
          sourceChannel: string
          denomAddress: string
          relayContractAddress: string
          evmSigner?: Account
          cosmosSigner?: OfflineSigner
          gasPrice?: { amount: string; denom: string }
          memo?: string
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
              simulate: true,
              memo
            })
            return transactionHash
          }

          console.info(`Transferring ${amount} ${denomAddress} to ${receiver}`)

          const cosmosRpcTransport = await rankCosmosRpcProviders({
            transports: Array.isArray(cosmos.transport)
              ? cosmos.transport.flatMap(t => t({}).value?.url).filter(Boolean)
              : [cosmos.transport({}).value?.url].filter(Boolean),
            interval: 1_000,
            sampleCount: 10,
            timeout: 1_000
          }).rank()
          const cosmosRpcUrl = cosmosRpcTransport.at(0)?.rpcUrl

          const _gasPrice = GasPrice.fromString(`${gasPrice?.amount}${gasPrice?.denom}`)

          if (!cosmosSigner) raise("Cosmos signer not found")
          if (!cosmosRpcUrl) raise("Cosmos RPC URL not found")
          if (!gasPrice) raise("Gas price not found")

          if (sourceChainId && destinationChainId === "union-testnet-8") {
            const transfer = await cosmosTransfer({
              receiver,
              cosmosSigner,
              cosmosRpcUrl,
              gasPrice: _gasPrice,
              asset: { denom: denomAddress, amount: amount.toString() }
            })
            return transfer.transactionHash
          }

          const stamp = timestamp()

          if (network === "cosmos" && sourceChainId === "union-testnet-8") {
            const transfer = await cosmwasmTransfer({
              cosmosSigner,
              cosmosRpcUrl,
              gasPrice: _gasPrice,
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
              gasPrice: _gasPrice,
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

          raise("Invalid network")
        }
      }
    })
}
