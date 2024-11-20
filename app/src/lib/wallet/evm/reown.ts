import {
  sepolia,
  scrollSepolia,
  arbitrumSepolia,
  berachainTestnetbArtio
} from "@reown/appkit/networks"
import { createAppKit } from "@reown/appkit"
import { WagmiAdapter } from "@reown/appkit-adapter-wagmi"

const projectId = "49fe74ca5ded7142adefc69a7788d14a"

export const networks = [sepolia, berachainTestnetbArtio, arbitrumSepolia, scrollSepolia]

const wagmiAdapter = new WagmiAdapter({
  projectId,
  networks
})

// 3. Configure the metadata
const metadata = {
  name: "Union",
  description: "Union App Testnet",
  url: "https://app.union.build",
  icons: ["https://union.build/logo.svg"]
}

// 3. Create the modal
export const reownModal = createAppKit({
  adapters: [wagmiAdapter],
  themeMode: "dark",
  themeVariables: {
    "--w3m-accent": "#A0ECFD",
    "--w3m-border-radius-master": "0.3px",
    "--w3m-font-family": "tt-supermolot-neue"
  },
  networks: [
    //
    sepolia,
    scrollSepolia,
    arbitrumSepolia,
    berachainTestnetbArtio
  ],
  metadata,
  projectId,
  enableEIP6963: true,
  enableWallets: true,
  enableCoinbase: true,
  enableInjected: true,
  enableWalletConnect: true,
  excludeWalletIds: [],
  features: {
    email: false,
    swaps: false,
    onramp: false,
    socials: false,
    analytics: false
  }
})
