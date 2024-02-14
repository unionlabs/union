import { Buffer } from 'node:buffer'
import { writable } from 'svelte/store'
import { mainnet, sepolia } from '@wagmi/core/chains'
import { walletConnect, injected, metaMask } from '@wagmi/connectors'
import {
  http,
  fallback,
  reconnect,
  getAccount,
  createConfig,
  watchAccount,
  unstable_connector
} from '@wagmi/core'

// Node polyfills
globalThis.Buffer = Buffer

const projectId = '640277c8235dc052b811d0cb88515fa5'

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
    }),
    metaMask(),
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

const accountStore = writable(getAccount(config))
