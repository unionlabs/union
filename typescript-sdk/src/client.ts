import {
  http,
  erc20Abi,
  getAddress,
  createClient,
  type Address,
  type Account,
  walletActions,
  publicActions
} from "viem"
import { sepolia } from "viem/chains"
import type { OfflineSigner } from "./types.ts"
import { raise, timestamp } from "./utilities.ts"
import type { GasPrice } from "@cosmjs/stargate"
import { bech32AddressToHex } from "./convert.ts"
import { ucs01RelayAbi } from "./abi/ucs01-relay.ts"
import type { ChainId } from "./constants/testnet.ts"
import { cosmosTransfer, cosmwasmTransfer, ibcTransfer } from "./transfer.ts"

export function createUnionClient({
  evmRpcUrl = "",
  cosmosRpcUrl,
  evmAccount,
  cosmosSigner
}: {
  evmRpcUrl?: string
  cosmosRpcUrl?: string
  evmAccount?: Account
  cosmosSigner?: OfflineSigner
}) {
  return createClient({
    account: evmAccount,
    chain: sepolia,
    transport: http(evmRpcUrl)
  })
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
      const _cosmosRpcUrl = cosmosRpcUrl
      const _evmAccount = evmAccount
      const _cosmosSigner = cosmosSigner
      return {
        async transferAsset({
          network,
          receiver,
          amount,
          path,
          sourceChannel,
          denomAddress,
          relayContractAddress,
          evmSigner = _evmAccount,
          cosmosSigner = _cosmosSigner,
          cosmosRpcUrl = _cosmosRpcUrl,
          gasPrice
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
          cosmosRpcUrl?: string
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
