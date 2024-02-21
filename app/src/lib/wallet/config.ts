import '$/patch.ts'
import {
  http,
  fallback,
  reconnect,
  getAccount,
  createConfig,
  watchAccount,
  unstable_connector,
  connect as _connect,
  disconnect as _disconnect,
  switchChain as _switchChain,
  type GetAccountReturnType
} from '@wagmi/core'
import { CHAIN, UNO } from '$/lib/constants'
import { injected } from '@wagmi/connectors'
import { getDenomAddress } from '$/lib/union-actions'
import { mainnet, sepolia } from '@wagmi/core/chains'
import { writable, type Writable } from 'svelte/store'
import { getKey, getSnap, connectSnap, suggestChain } from '@leapwallet/cosmos-snap-provider'

const projectId = '640277c8235dc052b811d0cb88515fa5'

const chains = [sepolia] as const
export type ConfiguredChainId = (typeof chains)[number]['id']

export type Wallet = GetAccountReturnType
export type ConnectedWallet = Wallet & { status: 'connected' }

export type ConnectorType = 'injected'

export const config = createConfig({
  chains: [mainnet, sepolia],
  syncConnectedChain: true,
  multiInjectedProviderDiscovery: true,
  /**
   * TODO: set storage using `unstorage`
   */
  connectors: [
    injected({
      shimDisconnect: true,
      unstable_shimAsyncInject: 2500
    })
    // metaMask()
    // walletConnect({ projectId })
  ],
  transports: {
    [mainnet.id]: fallback([
      //
      http(),
      unstable_connector(injected)
    ]),
    [sepolia.id]: fallback([
      //
      unstable_connector(injected),
      http()
    ])
    /**
     * TODO: add custom transport for Union chain.
     * @see
     * - https://wagmi.sh/core/api/transports/custom#custom
     * - https://viem.sh/docs/clients/transports/custom.html
     */
  }
  /**
   * TODO: add custom client for Union chain.
   * @see
   * - https://viem.sh/docs/clients/custom.html
   * - https://wagmi.sh/core/api/createConfig#client
   */
})

const accountStore = writable(getAccount(config)) satisfies Writable<Wallet>
watchAccount(config, { onChange: accountStore.set })
reconnect(config)

export const wallet = { subscribe: accountStore.subscribe }

export async function connect(type: ConnectorType, chainId: ConfiguredChainId | undefined) {
  const connectors = config.connectors.filter(c => c.type === type)
  const connector = connectors[0] ?? connectors[1]

  if (connector) return _connect(config, { connector, chainId })
}

export const disconnect = () => _disconnect(config)

export const switchChain = (chainId: ConfiguredChainId) => _switchChain(config, { chainId })

export const snapInstalled = writable(false) satisfies Writable<boolean>
export const unoTokenAddedToMetaMask = writable(false) satisfies Writable<boolean>
export const connectedToUnion = writable(false) satisfies Writable<boolean>
export const unionAddress = writable('') satisfies Writable<string>

export async function updateSnapInstalled() {
  const snap = await getSnap()
  snapInstalled.set(snap !== undefined)
}

export async function connectLeapSnap() {
  const snap = await getSnap()
  if (snap === undefined) await connectSnap()
  await updateSnapInstalled()
}

export async function connectToUnion() {
  await suggestChain(
    {
      chainId: CHAIN.UNION.ID,
      chainName: CHAIN.UNION.NAME,
      bip44: { coinType: UNO.COIN_TYPE },
      bech32Config: { bech32PrefixAccAddr: UNO.ADDRESS_PREFIX }
    },
    { force: false }
  )
  connectedToUnion.set(true)
}

export async function checkConnectedToUnion() {
  try {
    const key = await getKey(CHAIN.UNION.ID)
    connectedToUnion.set(key !== undefined)
    unionAddress.set(key?.address)
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : error
    console.error(errorMessage)
    connectedToUnion.set(false)
  }
}

export async function addUnoERC20() {
  try {
    const denomAddress = await getDenomAddress()
    const hasBeenAdded = await window.ethereum.request({
      method: 'wallet_watchAsset',
      params: {
        type: 'ERC20',
        options: {
          /**
           * TODO: THIS SHOULD NOT BE HARDCODED. INSTEAD CALL unionWalletClient().getDenomAddress()
           */
          address: denomAddress,
          /**
           * TODO: this should be UNO but our latest deployment to Sepolia has this as symbol
           */
          symbol: 'UNO',
          decimals: 6,
          image: 'https://union.build/logo.png'
        }
      }
    })

    if (hasBeenAdded) unoTokenAddedToMetaMask.set(true)
  } catch {
    unoTokenAddedToMetaMask.set(false)
  }
}
