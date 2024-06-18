import {
  erc20Abi,
  getAddress,
  type Address,
  type Account,
  publicActions,
  type WalletClient
} from "viem"
import { timestamp } from "../utilities/index.ts"
import { bech32AddressToHex } from "../convert.ts"
import { ucs01RelayAbi } from "../abi/ucs01-relay.ts"
import type { TransactionResponse } from "../types.ts"

/**
 * TODO:
 * - [ ] prefix logs with context to make it easier to debug
 */
export async function transferAssetFromEvm(
  client: WalletClient,
  {
    memo,
    amount,
    account,
    recipient,
    denomAddress,
    sourceChannel,
    simulate = true,
    relayContractAddress
  }: {
    memo?: string
    amount: bigint
    account?: Account
    recipient: string
    simulate?: boolean
    denomAddress: Address
    sourceChannel: string
    relayContractAddress: Address
  }
): Promise<TransactionResponse> {
  try {
    account ||= client.account
    if (!account) return { success: false, data: "No account found" }

    denomAddress = getAddress(denomAddress)
    /* lowercasing because for some reason our ucs01 contract only likes lowercase address */
    relayContractAddress = getAddress(relayContractAddress).toLowerCase() as Address

    const approve = await client.writeContract({
      abi: erc20Abi,
      account: account,
      chain: client.chain,
      address: denomAddress,
      functionName: "approve",
      args: [relayContractAddress, amount]
    })
    if (!approve) return { success: false, data: "Approval failed" }

    memo ||= timestamp()

    /**
     * @dev
     * `UCS01` contract `send` function:
     * - https://github.com/unionlabs/union/blob/142e0af66a9b0218cf010e3f8d1138de9b778bb9/evm/contracts/apps/ucs/01-relay/Relay.sol#L51-L58
     */
    const writeContractParameters = {
      account: account,
      abi: ucs01RelayAbi,
      chain: client.chain,
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
        bech32AddressToHex({ address: recipient }),
        [{ denom: denomAddress, amount }],
        memo,
        { revision_number: 9n, revision_height: 999_999_999n + 100n },
        0n
      ]
    } as const
    if (!simulate) {
      const hash = await client.writeContract(writeContractParameters)
      return { success: true, data: hash }
    }

    const { request } = await client.extend(publicActions).simulateContract(writeContractParameters)
    const hash = await client.writeContract(request)
    return { success: true, data: hash }
  } catch (error) {
    console.error(error)
    return {
      success: false,
      data: error instanceof Error ? error.message : "Unknown error"
    }
  }
}

export async function transferAssetFromEvmSimulate(
  client: WalletClient,
  {
    memo,
    amount,
    account,
    recipient,
    denomAddress,
    sourceChannel,
    relayContractAddress
  }: {
    memo?: string
    amount: bigint
    recipient: string
    denomAddress: Address
    sourceChannel: string
    account?: Account | Address
    relayContractAddress: Address
  }
): Promise<TransactionResponse> {
  try {
    account ||= client.account
    if (!account) return { success: false, data: "No account found" }

    denomAddress = getAddress(denomAddress)
    /* lowercasing because for some reason our ucs01 contract only likes lowercase address */
    relayContractAddress = getAddress(relayContractAddress).toLowerCase() as Address

    const approve = await client.writeContract({
      abi: erc20Abi,
      account: account,
      chain: client.chain,
      address: denomAddress,
      functionName: "approve",
      args: [relayContractAddress, amount]
    })
    if (!approve) return { success: false, data: "Approval failed" }

    memo ||= timestamp()
    const gasCostEstimation = await client.extend(publicActions).estimateContractGas({
      account: account,
      abi: ucs01RelayAbi,
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
        bech32AddressToHex({ address: recipient }),
        [{ denom: denomAddress, amount }],
        memo,
        { revision_number: 9n, revision_height: 999_999_999n + 100n },
        0n
      ]
    })

    return { success: true, data: gasCostEstimation.toString() }
  } catch (error) {
    console.error(error)
    return {
      success: false,
      data: error instanceof Error ? error.message : "Unknown error"
    }
  }
}
