import {
  erc20Abi,
  getAddress,
  type Address,
  type Account,
  type WalletClient,
  type PublicActions,
  type ContractFunctionRevertedErrorType
} from "viem"
import { ucs01RelayAbi } from "../abi/ucs-01.ts"
import { timestamp } from "../utilities/index.ts"
import { bech32AddressToHex } from "../convert.ts"
import type { TransactionResponse } from "../types.ts"

/**
 * TODO:
 * - [ ] prefix logs with context to make it easier to debug
 */
export async function transferAssetFromEvm(
  client: WalletClient & PublicActions,
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

    const approveWriteContractParameters = {
      account,
      abi: erc20Abi,
      chain: client.chain,
      address: denomAddress,
      functionName: "approve",
      args: [relayContractAddress, amount]
    } as const

    const { request: approveRequest } = await client.simulateContract(
      approveWriteContractParameters
    )
    const approveHash = await client.writeContract(approveRequest)

    if (!approveHash) return { success: false, data: "Approval failed" }
    const receipt = await client.waitForTransactionReceipt({ hash: approveHash })
    console.info(`[transferAssetFromEvm] Approval transaction hash: ${receipt.transactionHash}`)

    memo ||= timestamp()

    /**
     * @dev
     * `UCS01` contract `send` function:
     * - https://github.com/unionlabs/union/blob/142e0af66a9b0218cf010e3f8d1138de9b778bb9/evm/contracts/apps/ucs/01-relay/Relay.sol#L51-L58
     */
    const writeContractParameters = {
      account,
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
        recipient.startsWith("0x")
          ? getAddress(recipient)
          : bech32AddressToHex({ address: recipient }),
        [{ denom: denomAddress, amount }],
        memo,
        { revision_number: 9n, revision_height: BigInt(999_999_999) + 100n },
        0n
      ]
    } as const
    if (!simulate) {
      const hash = await client.writeContract(writeContractParameters)
      return { success: true, data: hash }
    }

    const { request } = await client.simulateContract(writeContractParameters)
    const hash = await client.writeContract(request)

    return { success: true, data: hash }
  } catch (error) {
    const castedError = error as ContractFunctionRevertedErrorType
    console.error(JSON.stringify(error, undefined, 2))
    return {
      success: false,
      data: error instanceof Error ? error.message : "Unknown error"
    }
  }
}

export async function transferAssetFromEvmSimulate(
  client: WalletClient & PublicActions,
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
    account?: Address
    denomAddress: Address
    sourceChannel: string
    relayContractAddress: Address
  }
): Promise<TransactionResponse> {
  try {
    if (!account) return { success: false, data: "No account found" }

    denomAddress = getAddress(denomAddress)
    /* lowercasing because for some reason our ucs01 contract only likes lowercase address */
    relayContractAddress = getAddress(relayContractAddress).toLowerCase() as Address

    memo ||= timestamp()
    const gasCostEstimation = await client.estimateContractGas({
      account,
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
        recipient.startsWith("0x")
          ? getAddress(recipient)
          : bech32AddressToHex({ address: recipient }),
        [{ denom: denomAddress, amount }],
        memo,
        { revision_number: 9n, revision_height: BigInt(999_999_999) + 100n },
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
