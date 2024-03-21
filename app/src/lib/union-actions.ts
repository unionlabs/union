import {
  erc20Abi,
  BaseError,
  type Hash,
  getAddress,
  bytesToHex,
  type Address,
  type Account,
  ContractFunctionRevertedError
} from "viem"
import toast from "svelte-french-toast"
import { snapAddress } from "$/lib/snap"
import { ucs01relayAbi } from "$/lib/abi"
import { writable, get } from "svelte/store"
import { fromBech32 } from "@cosmjs/encoding"
import { CONTRACT, UNO } from "$/lib/constants"
import { config, unionAddress, wallet } from "$/lib/wallet/config"
import { readContract, simulateContract, writeContract } from "@wagmi/core"

export async function getUnoERC20Balance(address: string) {
  const passedAddress = getAddress(address)
  const denomAddress = await getDenomAddress()
  if (BigInt(denomAddress) === 0n) return 0n

  return readContract(config, {
    abi: erc20Abi,
    functionName: "balanceOf",
    address: denomAddress,
    args: [passedAddress]
  })
}

export async function approveUnoERC20Spending({ amount }: { amount: bigint }) {
  const ethereumAddress = get(wallet).address
  if (!ethereumAddress) return
  const denomAddress = await getDenomAddress()
  if (BigInt(denomAddress) === 0n) return

  const contractParameters = {
    abi: erc20Abi,
    functionName: "approve",
    address: denomAddress,
    args: [getAddress(CONTRACT.SEPOLIA.ADDRESS), 69_420n],
    account: get(wallet) as unknown as Account
  } as const

  const { request } = await simulateContract(config, contractParameters)
  return writeContract(config, request)
}

export const sepoliaTransactions = writable<Array<string>>([])
export async function sendAssetFromEthereumToUnion({
  amount,
  simulate = true
}: {
  amount: bigint
  simulate?: boolean
}): Promise<Hash> {
  const _unionAddress = get(snapAddress)
  if (!_unionAddress) throw new Error("snap address not set")
  const ethereumAddress = get(wallet).address
  if (!ethereumAddress) throw new Error("ethereum address not set")

  const _approvalResult = await approveUnoERC20Spending({ amount })
  // TODO: make dynamic?
  const counterpartyTimeoutRevisionNumber = 6n
  // TODO: make dynamic?
  const counterpartyTimeoutRevisionHeight = 800_000_000n
  try {
    const sepoliaUnoBalance = await getUnoERC20Balance(ethereumAddress)
    if (sepoliaUnoBalance < amount) {
      throw new Error(
        `insufficient balance to send ${amount} muno to ${get(unionAddress)}: ${sepoliaUnoBalance}`
      )
    }

    const denomAddress = await getDenomAddress()

    const writeContractParameters = {
      abi: ucs01relayAbi,
      functionName: "send",
      address: getAddress(CONTRACT.SEPOLIA.ADDRESS),
      args: [
        CONTRACT.SEPOLIA.PORT_ID,
        CONTRACT.SEPOLIA.SOURCE_CHANNEL,
        bytesToHex(fromBech32(_unionAddress).data),
        [{ denom: denomAddress, amount }],
        counterpartyTimeoutRevisionNumber,
        counterpartyTimeoutRevisionHeight
      ]
    } as const
    if (!simulate) return await writeContract(config, writeContractParameters)

    const { request } = await simulateContract(config, writeContractParameters)
    const transactionHash = await writeContract(config, request)

    sepoliaTransactions.update(transactions => [...transactions, transactionHash])
    return transactionHash
  } catch (error) {
    if (error instanceof BaseError) {
      const revertError = error.walk(error => error instanceof ContractFunctionRevertedError)
      if (revertError instanceof ContractFunctionRevertedError) {
        const errorName = revertError.data?.errorName ?? ""
        toast.error(`error while sending ${amount} UNO [${errorName}-${revertError?.signature}]`, {
          position: "bottom-center"
        })
      }
    }
    const errorMessage = error instanceof Error ? error.message : error
    throw new Error(`error while sending ${amount} muno to ${get(unionAddress)}: ${errorMessage}`)
  }
}

export async function getDenomAddress(): Promise<Address> {
  const [sourcePort, sourceChannel, denom] = [
    CONTRACT.SEPOLIA.PORT_ID,
    CONTRACT.SEPOLIA.SOURCE_CHANNEL,
    UNO.NATIVE_DENOM
  ]

  return await readContract(config, {
    abi: ucs01relayAbi,
    address: getAddress(CONTRACT.SEPOLIA.ADDRESS),
    functionName: "getDenomAddress",
    args: [sourcePort, sourceChannel, `wasm.${CONTRACT.UNION.ADDRESS}/${sourceChannel}/${denom}`]
  })
}
