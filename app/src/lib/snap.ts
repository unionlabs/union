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
import { GasPrice, StargateClient } from '@cosmjs/stargate'
import { MsgExecuteContract } from 'cosmjs-types/cosmwasm/wasm/v1/tx'
import { Registry, makeAuthInfoBytes } from '@cosmjs/proto-signing'
import { TxBody } from 'cosmjs-types/cosmos/tx/v1beta1/tx'
import Long from 'long'
import { SignMode } from 'cosmjs-types/cosmos/tx/signing/v1beta1/signing'
import { toUtf8 } from '@cosmjs/encoding'
import { Any } from 'cosmjs-types/google/protobuf/any'
import { PublicKey } from 'cosmjs-types/tendermint/crypto/keys'

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
  if (get(snapAddress)?.length) return

  const chainAddressRequest = await getKey(CHAIN.UNION.ID)
  console.log(
    'wallet_invokeSnap - getChainAddressRequest',
    JSON.stringify(chainAddressRequest, undefined, 2)
  )
  const chainAddress = chainAddressRequest?.address
  pubKey.set(chainAddressRequest?.pubkey)
  console.log('wallet_invokeSnap - getChainAddress', JSON.stringify(chainAddress, undefined, 2))
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

export const snapTransaction = writable<string | null>(null)
export async function sendSnapTransaction() {
  if (!get(snapConnected)) return
  if (!get(snapChainInitialized)) return
  const ethereumAddress = get(wallet).address
  if (!ethereumAddress) return
  const offlineSigner = get(snapOfflineSigner)
  if (!offlineSigner) return

  const [accountData] = await offlineSigner.getAccounts()

  const stargateClient = await StargateClient.connect(URLS.UNION.RPC)
  const account = await stargateClient.getAccount(accountData.address)
  const [accountNumber, sequence] = [account?.accountNumber, account?.sequence]
  console.log(JSON.stringify({ accountNumber, sequence }, undefined, 2))

  const message = {
    typeUrl: '/cosmwasm.wasm.v1.MsgExecuteContract',
    value: MsgExecuteContract.fromPartial({
      sender: accountData.address,
      contract: CONTRACT.UNION.ADDRESS,
      msg: Buffer.from(
        JSON.stringify({
          transfer: {
            channel: CONTRACT.UNION.SOURCE_CHANNEL,
            receiver: '0xCa091fE8005596E64ba9Cf028a75755a2380021A'.slice(2),
            timeout: null,
            memo: "random more than four characters I'm transferring."
          }
        }),
        'utf-8'
      ),
      funds: [{ denom: UNO.NATIVE_DENOM, amount: '1000' }]
    })
  }

  const registry = new Registry()
  registry.register('/cosmwasm.wasm.v1.MsgExecuteContract', MsgExecuteContract)

  const transactionBody = TxBody.fromPartial({
    messages: [
      {
        typeUrl: message.typeUrl,
        value: message.value
      }
    ]
  })

  const bodyBytes = TxBody.encode(transactionBody).finish()

  const fee = {
    amount: [{ denom: UNO.NATIVE_DENOM, amount: '1000' }],
    gas: 200000
  }

  const gasPrice = GasPrice.fromString('0.001muno')

  const signer = [{ pubkey: accountData.pubkey, sequence: sequence }]

  //
  const authInfoBytes = makeAuthInfoBytes(
    Any.fromPartial({
      typeUrl: '/cosmos.crypto.secp256k1.PubKey',
      value: PublicKey.encode(accountData.pubkey).finish()
    }),
    fee.amount,
    fee.gas,
    undefined,
    undefined,
    SignMode.SIGN_MODE_DIRECT
  )

  const signed = await offlineSigner.signDirect(accountData.address, {
    chainId: CHAIN.UNION.ID,
    // account_number is the account number of the account in state
    accountNumber: Long.fromValue(accountNumber!),

    // auth_info_bytes is a protobuf serialization of an AuthInfo that matches the representation in TxRaw.
    authInfoBytes,
    // body_bytes is protobuf serialization of a TxBody that matches the representation in TxRaw.
    bodyBytes
  })

  console.log('wallet_invokeSnap - signDirect', JSON.stringify(signed, undefined, 2))
}
