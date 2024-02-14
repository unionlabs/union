import { sepolia } from 'viem/chains'
import { unionActions, getBalance, chain } from '../../../typescript-sdk/src/index'
import { createWalletClient, http, type Account, fallback, publicActions, type Address } from 'viem'
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
import { writable, type Writable } from 'svelte/store'

export function unionWalletClient(account?: Account | Address | undefined) {
  return createWalletClient({
    account,
    chain: sepolia,
    transport: fallback([
      http(
        'https://rpc.ankr.com/eth_sepolia/6c72c8d164912bed4694cb882fc4ca55574126511a4f5f66828a53fa2448a20a'
      ),
      http('https://ethereum-sepolia.publicnode.com')
    ])
  })
    .extend(publicActions)
    .extend(unionActions)
}

export const erc20balanceStore: Writable<bigint | null> = writable(null)
export async function getUnoERC20Balance(address: Address) {
  const client = unionWalletClient(address)
  const balance = await getBalance(client, {
    address,
    chainId: '11155111'
  })
  erc20balanceStore.set(balance)
  return balance
}

export const unionBalanceStore: Writable<bigint | null> = writable(null)

export async function getUnoUnionBalance(address: string) {
  const client = unionWalletClient()
  const balance = await getBalance(client, {
    address,
    chainId: '6'
  })
  unionBalanceStore.set(balance)
  return balance
}

export const cosmjsSigner = writable<CosmjsOfflineSigner | null>(null)
export async function _getOfflineSigner() {
  const offlineSigner = new CosmjsOfflineSigner('union-testnet-6')
  cosmjsSigner.set(offlineSigner)
}

export async function sendUnoFromUnionToSepolia(signer: CosmjsOfflineSigner, recipient: Address) {
  const client = unionWalletClient()
  // console.log(JSON.stringify(await signer.getAccounts(), undefined, 2))
  const result = await client.sendAsset({
    signer,
    chainId: '6',
    amount: '123',
    denom: 'muno',
    receiver: recipient,
    assetId: chain.union.testnet.token.address,
    gasPrice: '0.001muno'
  })
  console.log(JSON.stringify(result, undefined, 2))
  return result
}
