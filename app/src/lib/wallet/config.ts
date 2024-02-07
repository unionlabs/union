import { Buffer } from 'node:buffer'
import { mainnet, sepolia } from '@wagmi/core/chains'
import { walletConnect, injected } from '@wagmi/connectors'
import { http, createConfig, createStorage, fallback, unstable_connector } from '@wagmi/core'

// Node polyfills
globalThis.Buffer = Buffer

const projectId = '640277c8235dc052b811d0cb88515fa5'

export const config = createConfig({
  chains: [mainnet, sepolia],
  syncConnectedChain: true,
  multiInjectedProviderDiscovery: true,
  // storage: createStorage({ storage: localStorage }),
  connectors: [
    injected({
      target: 'metaMask',
      shimDisconnect: true,
      unstable_shimAsyncInject: 2500
    }),
    walletConnect({ projectId })
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
