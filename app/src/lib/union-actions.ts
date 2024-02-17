import { usc01relayAbi } from '$/lib/abi'
import { fromBech32 } from '@cosmjs/encoding'
import { writable, type Writable, get } from 'svelte/store'
import { CHAIN, CONTRACT, UNO, URLS } from '$/lib/constants.ts'
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate'
import { config, unionAddress, wallet } from '$/lib/wallet/config'
import { GasPrice, SigningStargateClient } from '@cosmjs/stargate'
import { CosmjsOfflineSigner } from '@leapwallet/cosmos-snap-provider'
import { Comet38Client, Tendermint37Client } from '@cosmjs/tendermint-rpc'
import { readContract, simulateContract, writeContract } from '@wagmi/core'
import { type Address, type Hash, bytesToHex, erc20Abi, getAddress } from 'viem'

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

/**
 * TODO: figure out why .execute doesn't run
 */
export async function sendUnoFromUnionToSepolia() {
  const offlineSigner = get(cosmosOfflineSigner)
  const ethereumAddress = get(wallet).address
  console.log({ offlineSigner, ethereumAddress })
  if (!offlineSigner) throw new Error('cosmos offline signer not initiated')
  const [account] = await offlineSigner.getAccounts()

  // const stargateClient = await SigningStargateClient.connectWithSigner(
  //   'https://union-testnet-rpc.polkachu.com',
  //   offlineSigner,
  //   { gasPrice: GasPrice.fromString('0.001muno') }
  // )

  const tendermintClient = await Tendermint37Client.connect(URLS.UNION.RPC)
  const cosmwasmClient = await SigningCosmWasmClient.createWithSigner(
    tendermintClient,
    offlineSigner,
    { gasPrice: GasPrice.fromString(`0.001${UNO.NATIVE_DENOM}`) }
  )
  const stargateClient = await SigningStargateClient.createWithSigner(
    tendermintClient,
    offlineSigner,
    { gasPrice: GasPrice.fromString(`0.001${UNO.NATIVE_DENOM}`) }
  )

  const address = account?.address
  const result = await cosmwasmClient.execute(
    address,
    CONTRACT.UNION.ADDRESS,
    {
      transfer: {
        channel: CONTRACT.UNION.SOURCE_CHANNEL,
        receiver: ethereumAddress?.slice(2),
        timeout: null,
        memo: "random more than four characters I'm transferring."
      }
    },
    'auto',
    undefined,
    [{ denom: UNO.NATIVE_DENOM, amount: '1000' }]
  )
  console.log(JSON.stringify({ result }, undefined, 2))
}

export async function sendAssetFromEthereumToUnion({
  amount,
  simulate = true
}: {
  amount: bigint
  simulate?: boolean
}): Promise<Hash> {
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
        bytesToHex(fromBech32(get(unionAddress)).data),
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
  const signer = get(cosmosOfflineSigner)
  const tendermintClient = await Comet38Client.connect(URLS.UNION.RPC)
  const cosmwasmClient = await SigningCosmWasmClient.createWithSigner(
    tendermintClient,
    // @ts-expect-error
    signer,
    { gasPrice: GasPrice.fromString(`0.001${UNO.NATIVE_DENOM}`) }
  )
  const { amount } = await cosmwasmClient.getBalance(address, UNO.NATIVE_DENOM)
  return amount
}
