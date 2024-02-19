import { usc01relayAbi } from '$/lib/abi'
import { fromBech32 } from '@cosmjs/encoding'
import { writable, type Writable, get } from 'svelte/store'
import { CHAIN, CONTRACT, UNO, URLS } from '$/lib/constants.ts'
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate'
import { config, unionAddress } from '$/lib/wallet/config'
import { GasPrice } from '@cosmjs/stargate'

import { Tendermint37Client } from '@cosmjs/tendermint-rpc'
import { readContract, simulateContract, writeContract } from '@wagmi/core'
import { type Address, type Hash, bytesToHex, erc20Abi, getAddress } from 'viem'
import { snapAddress } from '$/lib/snap'
import { CosmjsOfflineSigner } from '@leapwallet/cosmos-snap-provider'

export const erc20balanceStore: Writable<bigint | null> = writable(null)
export async function getUnoERC20Balance(address: Address) {
  const denomAddress = await getDenomAddress()
  if (BigInt(denomAddress) === 0n) return 0n

  return readContract(config, {
    abi: erc20Abi,
    functionName: 'balanceOf',
    address: denomAddress,
    args: [address]
  })
}

export const cosmosOfflineSigner = writable<CosmjsOfflineSigner | null>(null)
export async function initiateCosmosOfflineSigner() {
  const offlineSigner = new CosmjsOfflineSigner(CHAIN.UNION.ID)
  cosmosOfflineSigner.set(offlineSigner)
}

export const cosmWasmClient = writable<SigningCosmWasmClient | null>(null)
export async function initCosmWasmClient() {
  const tendermintClient = await Tendermint37Client.connect(URLS.UNION.RPC)

  const offlineSigner = get(cosmosOfflineSigner)
  if (!offlineSigner) throw new Error('cosmos offline signer not initiated')
  const cosmwasmClient = await SigningCosmWasmClient.createWithSigner(
    tendermintClient,
    offlineSigner,
    { gasPrice: GasPrice.fromString(`0.001${UNO.NATIVE_DENOM}`) }
  )
  cosmWasmClient.set(cosmwasmClient)
}

export async function sendAssetFromEthereumToUnion({
  amount,
  simulate = true
}: {
  amount: bigint
  simulate?: boolean
}): Promise<Hash> {
  const _unionAddress = get(snapAddress)
  if (!_unionAddress) throw new Error('snap address not set')
  // TODO: make dynamic?
  const counterpartyTimeoutRevisionNumber = 6n
  // TODO: make dynamic?
  const counterpartyTimeoutRevisionHeight = 800_000_000n
  try {
    const denomAddress = await getDenomAddress()

    const writeContractParameters = {
      abi: usc01relayAbi,
      functionName: 'send',
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
    console.log(JSON.stringify({ transactionHash }, undefined, 2))
    return transactionHash
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : error
    throw new Error(`error while sending ${amount} muno to ${get(unionAddress)}: ${errorMessage}`)
  }
}

export async function getDenomAddress(): Promise<Address> {
  const [sourcePort, sourceChannel, denom] = [
    CONTRACT.SEPOLIA.PORT_ID,
    CONTRACT.SEPOLIA.SOURCE_CHANNEL,
    UNO.NATIVE_DENOM || 'muno'
  ]

  return readContract(config, {
    abi: usc01relayAbi,
    address: getAddress(CONTRACT.SEPOLIA.ADDRESS),
    functionName: 'getDenomAddress',
    args: [sourcePort, sourceChannel, `wasm.${CONTRACT.UNION.ADDRESS}/${sourceChannel}/${denom}`]
  })
}

export const unionBalanceStore: Writable<string | null> = writable(null)

export async function getUnoUnionBalance(address: string) {
  const response = await fetch(`${URLS.UNION.REST}/cosmos/bank/v1beta1/balances/${address}`)
  const data = (await response.json()) as { balances: Array<{ amount: string; denom: string }> }
  const unoBalance = data.balances.find(({ denom }) => denom === UNO.NATIVE_DENOM)
  if (!unoBalance) return '0'
  return unoBalance.amount
}
