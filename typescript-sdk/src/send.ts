import {
  erc20Abi,
  type Hash,
  bytesToHex,
  type Account,
  type Address,
  type TransportConfig,
  type FallbackTransport,
} from 'viem'
import { raise } from '#/utilities'
import { usc01relayAbi } from '#/abi'
import { GasPrice } from '@cosmjs/stargate'
import { fromBech32 } from '@cosmjs/encoding'
import type { UnionClient } from '#/actions.ts'
import { Comet38Client } from '@cosmjs/tendermint-rpc'
import { UNION_RPC_URL, UCS01_EVM_ADDRESS } from '#/constants'
import { chainIds, type ChainId, chain } from '#/constants/chain.ts'
import type { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing'
import type { CosmjsOfflineSigner } from '@leapwallet/cosmos-snap-provider'
import { type ExecuteResult, SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate'

export interface ApproveAssetParameters {
  chainId: '1' | '11155111'
  assetId: Address
  signer: Account | Address
  amount: bigint
  spender?: Address
  simulate?: boolean
}

export async function approveAsset(
  client: UnionClient,
  { signer, assetId, amount, spender = UCS01_EVM_ADDRESS, simulate = true }: ApproveAssetParameters
): Promise<Hash> {
  try {
    const writeContractParameters = {
      account: signer,
      abi: erc20Abi,
      functionName: 'approve',
      address: assetId,
      args: [spender, amount],
      chain: client.chain,
    } as const

    if (!simulate) return await client.writeContract(writeContractParameters)

    const { request } = await client.simulateContract(writeContractParameters)
    const transactionHash = await client.writeContract(request)
    return transactionHash
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : error
    raise(`[approveAsset] error while approving ${amount} muno to ${spender}: ${errorMessage}`)
  }
}

export type SendAssetParameters<
  TChainId extends ChainId,
  TDenom extends string | undefined = TChainId extends '6' ? string : undefined,
  TGas extends `${string}${TDenom}` | undefined = TChainId extends '6'
    ? `${string}${TDenom}`
    : undefined,
  TTransportConfigType extends TransportConfig['type'] | undefined = TChainId extends '6'
    ? TransportConfig['type']
    : undefined,
> =
  | ({ chainId: '6' } & SendAssetFromUnionToEthereum<TDenom, TGas, TTransportConfigType>)
  | ({ chainId: '1' | '11155111' } & SendAssetFromEthereumToUnion)

export async function sendAsset<
  TChainId extends ChainId,
  TDenom extends string | undefined,
  TGas extends `${string}${TDenom}` | undefined,
  TTransportConfigType extends TransportConfig['type'] | undefined,
>(client: UnionClient, args: SendAssetParameters<TChainId, TDenom, TGas, TTransportConfigType>) {
  if (!chainIds.includes(args.chainId)) throw new Error(`Invalid chainId: ${args.chainId}`)
  else if (args.chainId === '6') return await sendAssetFromUnionToEthereum(client, args)
  else if (args.chainId === '11155111') return await sendAssetFromEthereumToUnion(client, args)
  else raise(`[sendAsset] chainId ${args.chainId} is not supported`)
}

interface SendAssetFromEthereumToUnion {
  assetId: Address
  receiver: string
  amount: bigint
  signer: Account | Address
  portId?: string
  channelId?: string
  simulate?: boolean
}

/**
 * Contract arguments:
 * @reference https://github.com/unionlabs/union/blob/main/evm/contracts/apps/ucs/01-relay/Relay.sol#L439-L444
 * - 'string calldata sourcePort',
 * - 'string calldata sourceChannel',
 * - `string calldata receiver`,
 * - `LocalToken[] calldata tokens`,
 * - `uint64 counterpartyTimeoutRevisionNumber`,
 * - `uint64 counterpartyTimeoutRevisionHeight`
 */
export async function sendAssetFromEthereumToUnion(
  client: UnionClient,
  {
    receiver,
    signer,
    amount,
    assetId, // denomAddress
    portId = chain.ethereum.sepolia.portId,
    channelId = chain.ethereum.sepolia.channelId,
    simulate = true,
  }: SendAssetFromEthereumToUnion
): Promise<Hash> {
  // TODO: make dynamic?
  const counterpartyTimeoutRevisionNumber = BigInt(chain.union.testnet.id)
  // TODO: make dynamic?
  const counterpartyTimeoutRevisionHeight = 800_000_000n
  try {
    const writeContractParameters = {
      account: signer,
      abi: usc01relayAbi,
      functionName: 'send',
      address: UCS01_EVM_ADDRESS,
      args: [
        portId,
        channelId,
        bytesToHex(fromBech32(receiver).data),
        [{ denom: assetId, amount }],
        counterpartyTimeoutRevisionNumber,
        counterpartyTimeoutRevisionHeight,
      ],
      chain: client.chain,
    } as const

    if (!simulate) {
      return await client.writeContract(writeContractParameters)
    }
    const { request } = await client.simulateContract(writeContractParameters)
    const transactionHash = await client.writeContract(request)
    return transactionHash
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : error
    raise(
      `[sendAssetFromEthereumToUnion] error while sending ${amount} muno to ${receiver} on ${client.transport.name} request: ${errorMessage}`
    )
  }
}

type OfflineSignerType<
  TransportConfigType extends TransportConfig['type'] | FallbackTransport | undefined,
> = TransportConfigType extends 'custom' ? CosmjsOfflineSigner : DirectSecp256k1HdWallet

type SendAssetFromUnionToEthereum<
  TDenom extends string | undefined,
  TGas extends `${string}${TDenom}` | undefined,
  TransportConfigType extends TransportConfig['type'] | undefined,
> = {
  assetId: string
  receiver: string
  amount: string
  denom: `${TDenom}`
  gasPrice?: TGas
  rpcUrl?: string
  memo?: string
  signer: OfflineSignerType<TransportConfigType>
}

export async function sendAssetFromUnionToEthereum<
  TDenom extends string | undefined,
  TGas extends `${string}${TDenom}` | undefined,
  TransportConfigType extends TransportConfig['type'] | undefined,
>(
  _client: UnionClient,
  {
    signer,
    assetId = chain.union.testnet.token.address,
    amount,
    denom,
    receiver,
    gasPrice,
    rpcUrl = UNION_RPC_URL,
    memo = "random more than four characters I'm transferring.",
  }: SendAssetFromUnionToEthereum<TDenom, TGas, TransportConfigType>
): Promise<ExecuteResult> {
  const tendermintClient = await Comet38Client.connect(rpcUrl)
  const cosmwasmClient = await SigningCosmWasmClient.createWithSigner(tendermintClient, signer, {
    gasPrice: GasPrice.fromString(gasPrice ?? `0.001${denom}`),
  })

  const [account] = await signer.getAccounts()
  const address = account?.address ?? raise('address is undefined')

  const result = await cosmwasmClient.execute(
    address,
    assetId,
    {
      transfer: {
        channel: chain.union.testnet.channelId,
        receiver: receiver.slice(2),
        timeout: null,
        memo,
      },
    },
    'auto',
    undefined,
    [{ denom: chain.union.testnet.token.denom, amount }]
  )

  return result
}
