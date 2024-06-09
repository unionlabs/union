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
import { raise } from "#utilities.ts"
import { sepolia } from "viem/chains"
import { bech32AddressToHex } from "#convert.ts"
import { ucs01RelayAbi } from "./abi/ucs01-relay.ts"
import type { ChainId } from "#constants/testnet.ts"
import type { NoRepetition, OfflineSigner } from "#types.ts"
import { cosmosTransfer, } from "#transfer.ts"
import type { GasPrice } from "@cosmjs/stargate"

export function createUnionClient({
  evmRpcUrl,
  cosmosRpcUrl,
  evmAccount,
  cosmosSigner
}: {
  evmRpcUrl: string
  cosmosRpcUrl: string
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
    .extend(client => ({
      async transferAssetFromCosmos({
        sourceChainId,
        destinationChainId,
        cosmosSigner,
        receiver,
        sourceChannel,
        amount,
        denomAddress,
        relayContractAddress,
        simulate = true
      }: {
        sourceChainId: string
        destinationChainId: string
        cosmosSigner?: OfflineSigner
        receiver: string
        sourceChannel: string
        amount: bigint
        denomAddress: Address
        relayContractAddress: Address
        simulate?: boolean
      }) {
        if (![sourceChainId, destinationChainId].includes("union-testnet-8")) {
          raise(
            "Either source or destination chain ID is not union-testnet-8. Must be union-testnet-8 until PFM is implemented"
          )
        }

        if (sourceChainId === "union-testnet-8") {
          if (!cosmosSigner) raise("Cosmos signer not found")
          const [account] = await cosmosSigner.getAccounts()
          if (!account) raise("No account found")
          const signingClient = await SigningCosmWasmClient.connectWithSigner(
            cosmosRpcUrl,
            cosmosSigner,
            {
              gasPrice: { amount: "0.0025", denom: "muno" }
            }
          )
          const response = await signingClient.executeMultiple(
            account.address,
            instructions,
            "auto"
          )
          return response
        }
      }
    }))
    .extend(client => ({
      async transferAsset({
        network,
        receiver,
        amount,
        path,
        denomAddress,
        cosmosSigner,
        cosmosRpcUrl,
        gasPrice
      }: {
        network: "cosmos" | "evm"
        receiver: string
        amount: bigint
        denomAddress: string
        path: NoRepetition<ChainId>
        cosmosSigner: OfflineSigner
        cosmosRpcUrl: string
        gasPrice: GasPrice
      }) {
        // @ts-expect-error
        if (!path.includes("union-testnet-8")) {
          raise(
            "Either source or destination chain ID is not union-testnet-8. Must be union-testnet-8 until PFM is implemented"
          )
        }
        const [sourceChainId, destinationChainId] = path

        if (network === "evm") {
          throw new Error("Not implemented")
        }

        if (network === "cosmos" && sourceChainId === "union-testnet-8") {
          throw new Error("Cosmwasm transfer not implemented")
        }

        if (network === "cosmos" && destinationChainId === "union-testnet-8") {
          throw new Error("IBC transfer not implemented")
        }

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
      }
    }))
}
