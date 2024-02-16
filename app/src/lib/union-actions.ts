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
import { config } from '$/lib/wallet/config'
import { fromBech32 } from '@cosmjs/encoding'
import { Comet38Client } from '@cosmjs/tendermint-rpc'
import { GasPrice, StargateClient } from '@cosmjs/stargate'
import { writable, type Writable, get } from 'svelte/store'
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate'
import { readContract, simulateContract, writeContract } from '@wagmi/core'

export const erc20balanceStore: Writable<bigint | null> = writable(null)
export async function getUnoERC20Balance(address: Address) {
  // const client = unionWalletClient(address)
  // const balance = await getBalance(client, {
  //   address,
  //   chainId: '11155111'
  // })
  // erc20balanceStore.set(balance)
  // return balance
  const denomAddress = await getDenomAddress()
  if (BigInt(denomAddress) === 0n) return 0n

  return readContract(config, {
    abi: erc20Abi,
    functionName: 'balanceOf',
    address: denomAddress,
    args: [address]
  })
}

export const cosmjsSigner = writable<CosmjsOfflineSigner | null>(null)
export async function _getOfflineSigner() {
  const offlineSigner = new CosmjsOfflineSigner('union-testnet-6')
  cosmjsSigner.set(offlineSigner)
}

export async function sendUnoFromUnionToSepolia(signer: CosmjsOfflineSigner, recipient: Address) {
  const tendermintClient = await Comet38Client.connect('https://union-testnet-rpc.polkachu.com')
  const cosmwasmClient = await SigningCosmWasmClient.createWithSigner(tendermintClient, signer, {
    gasPrice: GasPrice.fromString('0.001muno')
  })

  const [account] = await signer.getAccounts()
  const address = account?.address
  console.log(JSON.stringify(address, undefined, 2))

  const result = await cosmwasmClient.execute(
    address,
    'union14pfzjnvzacqsmgjyf0avksc8cr70hsyt5epzcp66tmjpswf8sq8sn5meuy',
    {
      transfer: {
        channel: 'channel-0',
        receiver: recipient.slice(2),
        timeout: null,
        memo: "random more than four characters I'm transferring."
      }
    },
    'auto',
    undefined,
    [{ denom: 'muno', amount: '123' }]
  )

  console.log(JSON.stringify(result, undefined, 2))

  return result
}

export async function sendAssetFromEthereumToUnion({
  receiver,
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
        bytesToHex(fromBech32(receiver).data),
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
    throw new Error(`error while sending ${amount} muno to ${receiver}: ${errorMessage}`)
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

export const unionBalanceStore: Writable<bigint | null> = writable(null)

export async function getUnoUnionBalance(address: string) {
  const signer = get(cosmjsSigner)
  const tendermintClient = await Comet38Client.connect('https://union-testnet-rpc.polkachu.com')
  const cosmwasmClient = await SigningCosmWasmClient.createWithSigner(tendermintClient, signer, {
    gasPrice: GasPrice.fromString('0.001muno')
  })
  const { amount } = await cosmwasmClient.getBalance(address, 'muno')
  return amount
}
