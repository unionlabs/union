import { sepolia } from 'viem/chains'
import { type Address, type Hash, bytesToHex, erc20Abi } from 'viem'
import {
  getKey,
  getSnap,
  getSnaps,
  connectSnap,
  suggestChain,
  signArbitrary,
  getOfflineSigner,
  CosmjsOfflineSigner,
  experimentalSuggestChain
} from '@leapwallet/cosmos-snap-provider'
import { usc01relayAbi } from '$/lib/abi'
import { config, unionAddress, wallet } from '$/lib/wallet/config'
import { fromBech32 } from '@cosmjs/encoding'
import { Comet38Client, Tendermint37Client } from '@cosmjs/tendermint-rpc'
import { writable, type Writable, get } from 'svelte/store'
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate'
import { GasPrice, SigningStargateClient } from '@cosmjs/stargate'
import { readContract, simulateContract, writeContract } from '@wagmi/core'

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
  const offlineSigner = new CosmjsOfflineSigner('union-testnet-6')
  cosmosOfflineSigner.set(offlineSigner)
}

export async function sendUnoFromUnionToSepolia() {
  setTimeout(() => {
    console.log('sending uno from union to sepolia.')
  })
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

  const tendermintClient = await Tendermint37Client.connect(
    // 'https://rpc.testnet.bonlulu.uno'
    'https://union-testnet-rpc.polkachu.com'
  )
  const cosmwasmClient = await SigningCosmWasmClient.createWithSigner(
    tendermintClient,
    offlineSigner,
    { gasPrice: GasPrice.fromString('0.001muno') }
  )
  const stargateClient = await SigningStargateClient.createWithSigner(
    tendermintClient,
    offlineSigner,
    { gasPrice: GasPrice.fromString('0.001muno') }
  )

  const address = account?.address
  const txResponse = await stargateClient.sendTokens(
    address,
    'union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv',
    [{ denom: 'muno', amount: '1000' }],
    'auto'
  )
  console.log(JSON.stringify({ txResponse }, undefined, 2))
  const result = await cosmwasmClient.execute(
    address,
    'union14pfzjnvzacqsmgjyf0avksc8cr70hsyt5epzcp66tmjpswf8sq8sn5meuy',
    {
      transfer: {
        channel: 'channel-0',
        receiver: ethereumAddress?.slice(2),
        timeout: null,
        memo: "random more than four characters I'm transferring."
      }
    },
    'auto',
    undefined,
    [{ denom: 'muno', amount: '1000' }]
  )
  console.log(JSON.stringify({ result }, undefined, 2))
}

export async function sendAssetFromEthereumToUnion({
  amount,
  portId = 'ucs01-relay',
  channelId = 'channel-0',
  simulate = true
}: {
  receiver: string
  amount: bigint
  portId?: string
  channelId?: string
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
      address: '0x7f7AC7d5a1a2bD54dBA53a22209C3f96699Ed63c',
      args: [
        portId,
        channelId,
        bytesToHex(fromBech32(get(unionAddress)).data),
        [{ denom: denomAddress, amount: 3n }],
        counterpartyTimeoutRevisionNumber,
        counterpartyTimeoutRevisionHeight
      ]
    } as const

    if (!simulate) {
      return await writeContract(config, writeContractParameters)
    }
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
    process.env.UCS01_SEPOLIA_PORT_ID || 'ucs01-relay',
    process.env.UCS01_SEPOLIA_SOURCE_CHANNEL || 'channel-0',
    process.env.UNION_NATIVE_DENOM || 'muno'
  ]
  const UNION_CONTRACT_ADDRESS =
    process.env.UCS01_UNION_ADDRESS ||
    'union14pfzjnvzacqsmgjyf0avksc8cr70hsyt5epzcp66tmjpswf8sq8sn5meuy'

  return readContract(config, {
    abi: usc01relayAbi,
    address: '0x7f7AC7d5a1a2bD54dBA53a22209C3f96699Ed63c',
    functionName: 'getDenomAddress',
    args: [sourcePort, sourceChannel, `wasm.${UNION_CONTRACT_ADDRESS}/${sourceChannel}/${denom}`]
  })
}

export const unionBalanceStore: Writable<string | null> = writable(null)

export async function getUnoUnionBalance(address: string) {
  const signer = get(cosmosOfflineSigner)
  const tendermintClient = await Comet38Client.connect('https://union-testnet-rpc.polkachu.com')
  const cosmwasmClient = await SigningCosmWasmClient.createWithSigner(
    tendermintClient,
    // @ts-expect-error
    signer,
    { gasPrice: GasPrice.fromString('0.001muno') }
  )
  const { amount } = await cosmwasmClient.getBalance(address, 'muno')
  return amount
}
