import { ucs03ZkgmAbi } from "../abi/ucs-03.ts"
import { timestamp } from "../utilities/index.ts"
import { err, ok, type Result } from "neverthrow"
import type { Hex, HexAddress } from "../types.ts"
import { bech32AddressToHex } from "../convert.ts"
import { simulateTransaction } from "../query/offchain/tenderly.ts"
import {
  erc20Abi,
  getAddress,
  type Account,
  type WalletClient,
  type PublicActions,
  toHex
} from "viem"

export type EvmTransferParams = {
  sourceChannelId: number
  receiver: string
  baseToken: HexAddress
  baseAmount: bigint
  quoteToken: HexAddress
  quoteAmount: bigint
  account?: Account
  simulate?: boolean
  ucs03address: HexAddress
}

/**
 * transfer an asset from evm
 * @example
 * ```ts
 * const transfer = await transferAssetFromEvm(client, {
 *   memo: "test",
 *   amount: 1n,
 *   account: evmAccount,
 *   sourceChannel: "channel-1",
 *   receiver: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd",
 *   denomAddress: "0x779877A7B0D9E8603169DdbD7836e478b4624789",
 *   relayContractAddress: "0x2222222222222222222222222222222222222222"
 * })
 * ```
 */
export async function transferAssetFromEvm(
  client: WalletClient & PublicActions,
  {
    account,
    receiver,
    baseToken,
    baseAmount,
    quoteToken,
    quoteAmount,
    sourceChannelId,
    simulate = true,
    ucs03address
  }: EvmTransferParams
): Promise<Result<Hex, Error>> {
  account ||= client.account
  if (!account) return err(new Error("No account found"))

  // add a salt to each transfer to prevent hash collisions
  // important because ibc-union does not use sequence numbers
  // such that intents are possible based on deterministic packet hashes
  const rawSalt = new Uint8Array(32)
  crypto.getRandomValues(rawSalt)
  const salt = toHex(rawSalt)

  /**
   * @dev
   * `UCS03` zkgm contract `transfer` function:
   * - https://github.com/unionlabs/union/blob/0fd24893d4a1173e9c6e150c826c162871d63262/evm/contracts/apps/ucs/03-zkgm/Zkgm.sol#L301
   */
  const writeContractParameters = {
    account,
    abi: ucs03ZkgmAbi,
    chain: client.chain,
    functionName: "transfer",
    address: ucs03address,
    /**
      "channelId": "uint32"
      "receiver": "bytes"
      "baseToken": "address"
      "baseAmount": "uint256"
      "quoteToken": "bytes"
      "quoteAmount": "uint256"
      "timeoutHeight": "uint64"
      "timeoutTimestamp": "uint64"
      "salt": "bytes32"
     */
    args: [
      sourceChannelId,
      receiver.startsWith("0x") ? getAddress(receiver) : bech32AddressToHex({ address: receiver }),
      baseToken,
      baseAmount,
      quoteToken,
      quoteAmount,
      0n, // TODO: customize timeoutheight
      "0x000000000000000000000000000000000000000000000000fffffffffffffffa", // TODO: make non-hexencoded timestamp
      salt
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

export type EvmApproveTransferParams = {
  amount: bigint
  account?: Account
  receiver: HexAddress
  denomAddress: HexAddress
  simulate?: boolean
}

/**
 * approve a transfer asset from evm
 * if transferring to a different chain, `receiver` is the relayer contract address
 * if transferring to the same chain, `receiver` is the recipient address
 *
 * @example
 * ```ts
 * const transfer = await evmApproveTransferAsset(client, {
 *   amount: 1n,
 *   simulate: true,
 *   autoApprove: true,
 *   account: privateKeyToAccount(`0x${PRIVATE_KEY}`),
 *   receiver: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd",
 *   denomAddress: "0x779877A7B0D9E8603169DdbD7836e478b4624789",
 * })
 * ```
 */
export async function evmApproveTransferAsset(
  client: WalletClient & PublicActions,
  { amount, account, receiver, denomAddress, simulate = true }: EvmApproveTransferParams
): Promise<Result<Hex, Error>> {
  account ||= client.account
  if (!account) return err(new Error("No account found"))

  const approvalParameters = {
    account,
    abi: erc20Abi,
    chain: client.chain,
    functionName: "approve",
    address: getAddress(denomAddress),
    args: [getAddress(receiver), amount]
  } as const

  if (!simulate) {
    const approveHash = await client.writeContract(approvalParameters)
    if (!approveHash) return err(new Error("Approval failed"))
    return ok(approveHash)
  }

  const { request } = await client.simulateContract(approvalParameters)
  if (!request) return err(new Error("Simulation failed"))

  const approveHash = await client.writeContract(request)
  if (!approveHash) return err(new Error("Approval failed"))

  const _receipt = await client.waitForTransactionReceipt({ hash: approveHash })

  return ok(approveHash)
}

export async function evmSameChainTransfer(
  client: WalletClient & PublicActions,
  {
    baseAmount: amount,
    account,
    receiver,
    baseToken: denomAddress,
    simulate = true
  }: Omit<EvmTransferParams, "memo" | "sourceChannel" | "relayContractAddress" | "autoApprove">
): Promise<Result<Hex, Error>> {
  account ||= client.account
  if (!account) return err(new Error("No account found"))

  denomAddress = getAddress(denomAddress)

  const transferParameters = {
    account,
    abi: erc20Abi,
    chain: client.chain,
    functionName: "transfer",
    address: getAddress(denomAddress),
    args: [getAddress(receiver), amount]
  } as const

  if (!simulate) {
    const hash = await client.writeContract({
      account,
      abi: erc20Abi,
      chain: client.chain,
      functionName: "transfer",
      address: getAddress(denomAddress),
      args: [getAddress(receiver), amount]
    })
    if (!hash) return err(new Error("Transfer failed"))
    return ok(hash)
  }

  const { request } = await client.simulateContract(transferParameters)
  const transferHash = await client.writeContract(request)

  const _receipt = await client.waitForTransactionReceipt({ hash: transferHash })

  return ok(transferHash)
}

/**
 * simulate a transfer asset from evm
 * @example
 * ```ts
 * const transfer = await transferAssetFromEvmSimulate(client, {
 *   memo: "test",
 *   amount: 1n,
 *   account: evmAccount,
 *   sourceChannel: "channel-1",
 *   receiver: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd",
 *   denomAddress: "0x779877A7B0D9E8603169DdbD7836e478b4624789",
 *   relayContractAddress: "0x2222222222222222222222222222222222222222",
 * })
 * ```
 */
export async function transferAssetFromEvmSimulate(
  _client: WalletClient & PublicActions,
  {
    memo,
    amount,
    account,
    receiver,
    denomAddress,
    sourceChannel,
    relayContractAddress
  }: {
    memo?: string
    amount: bigint
    receiver: string
    account?: HexAddress
    denomAddress: HexAddress
    sourceChannel: number
    relayContractAddress: HexAddress
  }
): Promise<Result<string, Error>> {
  if (!account) return err(new Error("No account found"))

  denomAddress = getAddress(denomAddress)
  /* lowercasing because for some reason our ucs01 contract only likes lowercase address */
  relayContractAddress = getAddress(relayContractAddress).toLowerCase() as HexAddress

  memo ??= timestamp()

  const gasEstimation = await simulateTransaction({
    memo,
    amount,
    receiver,
    denomAddress,
    sourceChannel,
    account: account,
    relayContractAddress
  })
  return ok(gasEstimation.toString())
}
