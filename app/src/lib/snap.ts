import { CHAIN, CONTRACT, UNO, URLS } from '$/lib/constants'
import { get, writable } from 'svelte/store'
import { wallet } from '$/lib/wallet/config'
import {
  CosmjsOfflineSigner,
  connectSnap,
  getKey,
  getSnap,
  suggestChain
} from '@leapwallet/cosmos-snap-provider'
import { GasPrice, SigningStargateClient, type StargateClient } from '@cosmjs/stargate'
import { Tendermint37Client } from '@cosmjs/tendermint-rpc'
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate'

export const snapInstalled = writable(false)
export async function ensureSnapInstalled() {
  if (get(snapInstalled)) return

  const currentSnaps = await window.ethereum?.request({ method: 'wallet_getSnaps' })
  const installed = Object.hasOwn(currentSnaps, 'npm:@leapwallet/metamask-cosmos-snap')
  console.info('wallet_getSnaps - installed', installed)
  snapInstalled.set(installed)

  if (installed) return

  await window.ethereum.request({
    method: 'wallet_requestSnaps',
    params: { 'npm:@leapwallet/metamask-cosmos-snap': { version: '^0.1.18' } }
  })
  ensureSnapInstalled()
}

export const snapConnected = writable(false)
export async function ensureSnapConnected() {
  const snap = await getSnap()
  if (!snap) await connectSnap()
  const snapSecondCheck = await getSnap()
  snapConnected.set(snapSecondCheck !== undefined)
}

export const snapAddress = writable<string | null>(null)
const pubKey = writable<Uint8Array | null>(null)
export async function getSnapAddress() {
  if (!get(snapConnected)) return

  const chainAddressRequest = await getKey(CHAIN.UNION.ID)
  const chainAddress = chainAddressRequest?.address
  pubKey.set(chainAddressRequest?.pubkey)
  snapAddress.set(chainAddress)
}

export const snapChainConnected = writable(false)

export const snapChainInitialized = writable(false)
export async function ensureSnapChainInitialized() {
  if (!get(snapConnected)) return
  try {
    const key = await getKey(CHAIN.UNION.ID)
    snapChainInitialized.set(key !== undefined)
    snapChainConnected.set(true)
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : error
    console.error('wallet_invokeSnap - ensureSnapChainInitialized', errorMessage)
    snapChainInitialized.set(false)
  }
}

export async function suggestSnapChain() {
  if (!get(snapConnected)) return
  if (get(snapChainInitialized)) return

  /**
   * chainInfo should be structured like this
   * @link https://github.com/cosmos/chain-registry/blob/master/testnets/uniontestnet/chain.json
   */
  const suggestChainRequest = await suggestChain(
    {
      chainId: CHAIN.UNION.ID,
      chainName: CHAIN.UNION.NAME,
      bip44: { coinType: UNO.COIN_TYPE },
      bech32Config: { bech32PrefixAccAddr: UNO.ADDRESS_PREFIX }
    },
    { force: false }
  )

  console.log('wallet_invokeSnap - addChain', JSON.stringify(suggestChain, undefined, 2))
  snapChainConnected.set(suggestChainRequest.chainInfo !== undefined)
}

export const snapOfflineSigner = writable<CosmjsOfflineSigner | null>(null)
export async function initializeSnapOfflineSigner() {
  if (!get(snapConnected)) return
  if (!get(snapChainInitialized)) return

  const offlineSigner = new CosmjsOfflineSigner(CHAIN.UNION.ID)
  snapOfflineSigner.set(offlineSigner)
}

export const tendermintClient = writable<Tendermint37Client | null>(null)
export async function initializeTendermintClient() {
  if (!get(snapConnected)) return
  if (!get(snapChainInitialized)) return

  const _tendermintClient = await Tendermint37Client.connect(URLS.UNION.RPC)
  tendermintClient.set(_tendermintClient)
}

export const stargateOfflineSigner = writable<StargateClient | null>(null)
export async function initializeStargateClient() {
  if (!get(snapConnected)) return
  if (!get(snapChainInitialized)) return

  const offlineSigner = get(snapOfflineSigner)
  if (!offlineSigner) return

  const _tendermintClient = get(tendermintClient)
  if (!_tendermintClient) return

  const stargateClient = await SigningStargateClient.createWithSigner(
    _tendermintClient,
    offlineSigner,
    { gasPrice: GasPrice.fromString(`0.001${UNO.NATIVE_DENOM}`) }
  )
  stargateOfflineSigner.set(stargateClient)
}

export const signingCosmWasmClient = writable<SigningCosmWasmClient | null>(null)
export async function initializeSigningCosmWasmClient() {
  if (!get(snapConnected)) return
  if (!get(snapChainInitialized)) return

  const offlineSigner = get(snapOfflineSigner)
  if (!offlineSigner) return

  const _tendermintClient = get(tendermintClient)
  if (!_tendermintClient) return

  const _signingCosmWasmClient = await SigningCosmWasmClient.createWithSigner(
    _tendermintClient,
    offlineSigner,
    {
      gasPrice: GasPrice.fromString(`0.001${UNO.NATIVE_DENOM}`)
    }
  )
  signingCosmWasmClient.set(_signingCosmWasmClient)
}

export const unionTransactions = writable<Array<string>>([])
export async function sendAssetFromUnionToEthereum({ amount }: { amount: string }) {
  if (!get(snapConnected)) return
  if (!get(snapChainInitialized)) return
  const ethereumAddress = get(wallet).address

  if (!ethereumAddress) {
    console.error('[sendSnapTransaction] missing data. Initialize the client and signer first.')
    return
  }

  const offlineSigner = new CosmjsOfflineSigner(CHAIN.UNION.ID)
  const tendermintClient = await Tendermint37Client.connect(URLS.UNION.RPC)

  const signingCosmWasmClient = await SigningCosmWasmClient.createWithSigner(
    tendermintClient,
    offlineSigner,
    { gasPrice: GasPrice.fromString(`0.001${UNO.NATIVE_DENOM}`) }
  )

  const [{ address: unionAddress }] = await offlineSigner.getAccounts()

  const result = await signingCosmWasmClient.execute(
    unionAddress,
    CONTRACT.UNION.ADDRESS,
    {
      transfer: {
        channel: CONTRACT.UNION.SOURCE_CHANNEL,
        receiver: ethereumAddress.slice(2),
        timeout: null,
        memo: 'random more than four characters I am transferring.'
      }
    },
    'auto',
    undefined,
    [{ denom: UNO.NATIVE_DENOM, amount }]
  )

  unionTransactions.update(transactions => [...transactions, result.transactionHash])
}
