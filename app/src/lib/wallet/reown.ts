import { chains } from "./evm/config.ts"
import { createAppKit } from "@reown/appkit"
import { sepolia } from "@reown/appkit/networks"
import { WagmiAdapter } from "@reown/appkit-adapter-wagmi"

const WALLETCONNECT_PROJECT_ID = "49fe74ca5ded7142adefc69a7788d14a"

const networks = chains

const wagmiAdapter = new WagmiAdapter({
  projectId: WALLETCONNECT_PROJECT_ID,
  networks: [sepolia]
})

// 4. Trigger modal programaticaly

export function reownEventListeners() {
  // 3. Create the modal
  const appKit = createAppKit({
    networks: [sepolia],
    adapters: [wagmiAdapter],
    metadata: {
      name: "Union",
      url: "https://app.union.build",
      icons: ["https://app.union.build/apple-touch-icon.png"],
      description: "Union is the cross-ecosystem middleware for modular interoperability."
    },
    projectId: WALLETCONNECT_PROJECT_ID,
    features: {
      email: false,
      onramp: false,

      socials: false
    }
  })
  console.info(appKit)

  const reownConnectModalElement = document.getElementById("open-connect-modal")
  const reownNetworkModalElement = document.getElementById("open-network-modal")
  console.info("reownConnectModalElement", reownConnectModalElement)
  console.info("reownNetworkModalElement", reownNetworkModalElement)
  // @ts-expect-error
  reownConnectModalElement?.addEventListener("click", () => modal.open())
  // @ts-expect-error
  reownNetworkModalElement?.addEventListener("click", () => modal.open({ view: "Networks" }))
}
