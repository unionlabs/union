import {
  erc20Abi,
  getAddress,
  type Address,
  type Account,
  type WalletClient,
  type PublicActions
} from "viem"
import { ucs01RelayAbi } from "../abi/ucs-01.ts"
import { timestamp } from "../utilities/index.ts"
import { err, ok, type Result } from "neverthrow"
import type { ChainId } from "../client/types.ts"
import { bech32AddressToHex } from "../convert.ts"
import { simulateTransaction } from "../query/offchain/tenderly.ts"

export type TransferAssetFromEvmParams = {
  memo?: string
  amount: bigint
  account?: Account
  recipient: string
  approve?: boolean
  simulate?: boolean
  denomAddress: Address
  sourceChannel: string
  relayContractAddress: Address
  destinationChainId: ChainId | (string & {})
}

/**
 * TODO: add JSDoc with examples
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
    approve = false,
    simulate = true,
    relayContractAddress
  }: TransferAssetFromEvmParams
): Promise<Result<string, Error>> {
  account ||= client.account
  if (!account) return err(new Error("No account found"))

  denomAddress = getAddress(denomAddress)
  /* lowercasing because for some reason our ucs01 contract only likes lowercase address */
  relayContractAddress = getAddress(relayContractAddress).toLowerCase() as Address

  if (approve) {
    const approveResponse = await approveTransferAssetFromEvm(client, {
      amount,
      account,
      denomAddress,
      relayContractAddress
    })
    if (approveResponse.isErr()) return approveResponse
  }

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
    return ok(hash)
  }

  const { request } = await client.simulateContract(writeContractParameters)
  const hash = await client.writeContract(request)

  return ok(hash)
}

export type ApproveTransferAssetFromEvmParams = Pick<
  TransferAssetFromEvmParams,
  "amount" | "account" | "simulate" | "denomAddress" | "relayContractAddress"
>

/**
 * TODO: add JSDoc with examples
 */
export async function approveTransferAssetFromEvm(
  client: WalletClient & PublicActions,
  {
    amount,
    account,
    denomAddress,
    simulate = true,
    relayContractAddress
  }: ApproveTransferAssetFromEvmParams
): Promise<Result<string, Error>> {
  account ||= client.account
  if (!account) return err(new Error("No account found"))

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

  if (!simulate) {
    const { request: approveRequest } = await client.simulateContract(
      approveWriteContractParameters
    )
    const approveHash = await client.writeContract(approveRequest)
    return ok(approveHash)
  }

  const approveHash = await client.writeContract(approveWriteContractParameters)

  if (!approveHash) return err(new Error("Approval failed"))
  const receipt = await client.waitForTransactionReceipt({ hash: approveHash })

  return ok(approveHash)
}

/**
 * TODO: add JSDoc with examples
 */
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
): Promise<Result<string, Error>> {
  if (!account) return err(new Error("No account found"))

  denomAddress = getAddress(denomAddress)
  /* lowercasing because for some reason our ucs01 contract only likes lowercase address */
  relayContractAddress = getAddress(relayContractAddress).toLowerCase() as Address

  memo ||= timestamp()

  const gasEstimation = await simulateTransaction({
    memo,
    amount,
    recipient,
    denomAddress,
    sourceChannel,
    account: account,
    relayContractAddress
  })
  return ok(gasEstimation.toString())
}
