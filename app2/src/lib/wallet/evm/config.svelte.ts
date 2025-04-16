import {
  connect as _connect,
  type Connector,
  disconnect as _disconnect,
  getAccount,
  type GetAccountReturnType,
  reconnect,
  switchChain as _switchChain,
  watchAccount
} from "@wagmi/core"
import { AddressEvmCanonical } from "@unionlabs/sdk/schema"
import { wallets } from "$lib/stores/wallets.svelte"
import { Effect, Option } from "effect"
import type { Hex } from "viem"

import { type ConfiguredChainId, wagmiConfig } from "./wagmi-config.ts"
import { sepolia } from "@wagmi/core/chains"

export type Wallet = GetAccountReturnType
export type ConnectorType = "injected" | "walletConnect"

const LAST_CONNECTED_WALLET_KEY = "last-connected-wallet"

function getLastConnectedWalletId(): string | undefined {
  if (typeof window !== "undefined") {
    return window.localStorage.getItem(LAST_CONNECTED_WALLET_KEY) || undefined
  }
  return undefined
}

function setLastConnectedWalletId(walletId: string | undefined) {
  if (typeof window !== "undefined" && walletId) {
    window.localStorage.setItem(LAST_CONNECTED_WALLET_KEY, walletId)
  }
}

function clearLastConnectedWalletId() {
  if (typeof window !== "undefined") {
    window.localStorage.removeItem(LAST_CONNECTED_WALLET_KEY)
  }
}

class SepoliaStore {
  chain: string = $state("11155111")
  address: string | undefined = $state(undefined)
  connectionStatus: "connected" | "connecting" | "disconnected" | "reconnecting" =
    $state("disconnected")
  connectedWallet = $state(getLastConnectedWalletId())

  private unwatchAccount: (() => void) | undefined

  constructor() {
    this.reconnectLast()
  }

  startWatchingAccount = () => {
    if (this.unwatchAccount) {
      this.unwatchAccount()
      this.unwatchAccount = undefined
    }

    this.unwatchAccount = watchAccount(wagmiConfig, {
      onChange: account => {
        if (this.connectionStatus === "connected" && !account.address && this.address) {
          return
        }

        this.updateAccount({
          chain: account.chain?.name ?? "sepolia",
          address: account.address,
          connectionStatus: account.status,
          connectedWallet: account.connector?.id
        })
      }
    })
  }

  connect = async (walletId: string) => {
    try {
      this.connectionStatus = "connecting"

      if (this.unwatchAccount) {
        this.unwatchAccount()
      }

      const result = await evmConnect(walletId, sepolia.id)
      const account = getAccount(wagmiConfig)
      setLastConnectedWalletId(account.connector?.id)

      this.updateAccount({
        chain: account.chain?.name ?? "sepolia",
        address: account.address,
        connectionStatus: account.status,
        connectedWallet: account.connector?.id
      })

      this.startWatchingAccount()

      return result
    } catch (error) {
      console.error("Connection failed:", error)
      this.connectionStatus = "disconnected"
    }
  }

  disconnect = async () => {
    try {
      if (this.unwatchAccount) {
        this.unwatchAccount()
        this.unwatchAccount = undefined
      }

      await evmDisconnect()
      if (typeof window !== "undefined") {
        Object.keys(window.localStorage)
          .filter(key => key.startsWith("union-wagmi"))
          .forEach(key => window.localStorage.removeItem(key))
      }
      clearLastConnectedWalletId()
      this.updateAccount({
        chain: "sepolia",
        address: undefined,
        connectionStatus: "disconnected",
        connectedWallet: undefined
      })
    } catch (error) {
      console.error("Disconnect failed:", error)
    }
    Effect.sleep(2_000)
  }

  reconnectLast = async () => {
    const lastWalletId = getLastConnectedWalletId()
    if (!lastWalletId || typeof window === "undefined") return

    try {
      this.connectionStatus = "connecting"

      const lastConnector = wagmiConfig.connectors.find(c => c.id === lastWalletId)
      if (!lastConnector) {
        console.warn("Last connected connector not found:", lastWalletId)
        this.connectionStatus = "disconnected"
        return
      }

      await reconnect(wagmiConfig, { connectors: [lastConnector] })

      const account = getAccount(wagmiConfig)
      this.updateAccount({
        chain: account.chain?.name ?? "sepolia",
        address: account.address,
        connectionStatus: account.status,
        connectedWallet: account.connector?.id
      })

      this.startWatchingAccount()
    } catch (error) {
      console.error("Reconnection failed:", error)
      this.connectionStatus = "disconnected"
    }
  }

  updateAccount = (account: {
    chain: string
    address: `0x${string}` | undefined
    connectionStatus: "connected" | "connecting" | "disconnected" | "reconnecting"
    connectedWallet: string | undefined
  }) => {
    if (account.chain) this.chain = account.chain
    this.address = account.address
    if (account.address) {
      const evmAddressFromHex = (hexAddress: Hex) => {
        const normalized = hexAddress.slice(2).toLowerCase()
        return AddressEvmCanonical.make(`0x${normalized}`)
      }
      wallets.evmAddress = Option.some(evmAddressFromHex(account.address as `0x${string}`))
    } else {
      wallets.evmAddress = Option.none()
    }

    if (account.connectionStatus) this.connectionStatus = account.connectionStatus
    this.connectedWallet = account.connectedWallet
  }
}

export const sepoliaStore = new SepoliaStore()

const excludeWalletList = ["io.leapwallet.LeapWallet", "app.keplr"]

export const evmWalletsInformation = wagmiConfig.connectors
  .map(connector => {
    const id = connector.id.toLowerCase()
    const name = connector.name.toLowerCase()
    return {
      ...connector,
      name: name.includes("injected") ? "Browser Wallet" : connector.name,
      icon: (id.includes("walletconnect")
        ? "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'%3E%3Cpath fill='%23268fff' d='M4.91 7.52a10.18 10.18 0 0 1 14.18 0l.47.46a.48.48 0 0 1 0 .7l-1.61 1.57a.25.25 0 0 1-.36 0l-.65-.63a7.1 7.1 0 0 0-9.88 0l-.7.68a.26.26 0 0 1-.35 0L4.4 8.72a.48.48 0 0 1 0-.7zm17.5 3.26 1.44 1.4a.48.48 0 0 1 0 .7l-6.46 6.33a.51.51 0 0 1-.71 0l-4.59-4.5a.13.13 0 0 0-.18 0l-4.59 4.5a.51.51 0 0 1-.7 0L.14 12.88a.48.48 0 0 1 0-.7l1.43-1.4a.51.51 0 0 1 .71 0l4.59 4.5c.05.04.13.04.18 0l4.59-4.5a.51.51 0 0 1 .7 0l4.6 4.5c.04.04.12.04.17 0l4.6-4.5a.5.5 0 0 1 .7 0' /%3E%3C/svg%3E%0A"
        : name.includes("injected")
          ? "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 32 32'%3E%3Cpath fill='%23f7f7f7' fill-rule='evenodd' d='m15.65 3.64-9.6 4.8 10.2 5.1 10.2-5.1-9.6-4.8a1.35 1.35 0 0 0-1.2 0ZM28 10.46l-10.5 5.25v12.81l9.75-4.87a1.35 1.35 0 0 0 .75-1.21V10.46ZM15 28.53V15.7L4.5 10.46v11.97a1.35 1.35 0 0 0 .74 1.22L15 28.53Zm-.48 2.55-10.4-5.2A3.85 3.85 0 0 1 2 22.42V10.05A3.85 3.85 0 0 1 4.13 6.6l10.4-5.2a3.85 3.85 0 0 1 3.43 0l10.4 5.2a3.85 3.85 0 0 1 2.14 3.45v12.39a3.85 3.85 0 0 1-2.13 3.44l-10.4 5.2a3.85 3.85 0 0 1-3.45 0Z' clip-rule='evenodd'/%3E%3C/svg%3E"
          : name === "metamask"
            ? "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' fill='none' height='33' viewBox='0 0 35 33' width='35'%3E%3Cg stroke-linecap='round' stroke-linejoin='round' stroke-width='.25'%3E%3Cpath d='m32.9582 1-13.1341 9.7183 2.4424-5.72731z' fill='%23e17726' stroke='%23e17726' /%3E%3Cg fill='%23e27625' stroke='%23e27625'%3E%3Cpath d='m2.66296 1 13.01714 9.809-2.3254-5.81802z' /%3E%3Cpath d='m28.2295 23.5335-3.4947 5.3386 7.4829 2.0603 2.1436-7.2823z' /%3E%3Cpath d='m1.27281 23.6501 2.13055 7.2823 7.46994-2.0603-3.48166-5.3386z' /%3E%3Cpath d='m10.4706 14.5149-2.0786 3.1358 7.405.3369-.2469-7.969z' /%3E%3Cpath d='m25.1505 14.5149-5.1575-4.58704-.1688 8.05974 7.4049-.3369z' /%3E%3Cpath d='m10.8733 28.8721 4.4819-2.1639-3.8583-3.0062z' /%3E%3Cpath d='m20.2659 26.7082 4.4689 2.1639-.6105-5.1701z' /%3E%3C/g%3E%3Cpath d='m24.7348 28.8721-4.469-2.1639.3638 2.9025-.039 1.231z' fill='%23d5bfb2' stroke='%23d5bfb2' /%3E%3Cpath d='m10.8732 28.8721 4.1572 1.9696-.026-1.231.3508-2.9025z' fill='%23d5bfb2' stroke='%23d5bfb2' /%3E%3Cpath d='m15.1084 21.7842-3.7155-1.0884 2.6243-1.2051z' fill='%23233447' stroke='%23233447' /%3E%3Cpath d='m20.5126 21.7842 1.0913-2.2935 2.6372 1.2051z' fill='%23233447' stroke='%23233447' /%3E%3Cpath d='m10.8733 28.8721.6495-5.3386-4.13117.1167z' fill='%23cc6228' stroke='%23cc6228' /%3E%3Cpath d='m24.0982 23.5335.6366 5.3386 3.4946-5.2219z' fill='%23cc6228' stroke='%23cc6228' /%3E%3Cpath d='m27.2291 17.6507-7.405.3369.6885 3.7966 1.0913-2.2935 2.6372 1.2051z' fill='%23cc6228' stroke='%23cc6228' /%3E%3Cpath d='m11.3929 20.6958 2.6242-1.2051 1.0913 2.2935.6885-3.7966-7.40495-.3369z' fill='%23cc6228' stroke='%23cc6228' /%3E%3Cpath d='m8.392 17.6507 3.1049 6.0513-.1039-3.0062z' fill='%23e27525' stroke='%23e27525' /%3E%3Cpath d='m24.2412 20.6958-.1169 3.0062 3.1049-6.0513z' fill='%23e27525' stroke='%23e27525' /%3E%3Cpath d='m15.797 17.9876-.6886 3.7967.8704 4.4833.1949-5.9087z' fill='%23e27525' stroke='%23e27525' /%3E%3Cpath d='m19.8242 17.9876-.3638 2.3584.1819 5.9216.8704-4.4833z' fill='%23e27525' stroke='%23e27525' /%3E%3Cpath d='m20.5127 21.7842-.8704 4.4834.6236.4406 3.8584-3.0062.1169-3.0062z' fill='%23f5841f' stroke='%23f5841f' /%3E%3Cpath d='m11.3929 20.6958.104 3.0062 3.8583 3.0062.6236-.4406-.8704-4.4834z' fill='%23f5841f' stroke='%23f5841f' /%3E%3Cpath d='m20.5906 30.8417.039-1.231-.3378-.2851h-4.9626l-.3248.2851.026 1.231-4.1572-1.9696 1.4551 1.1921 2.9489 2.0344h5.0536l2.962-2.0344 1.442-1.1921z' fill='%23c0ac9d' stroke='%23c0ac9d' /%3E%3Cpath d='m20.2659 26.7082-.6236-.4406h-3.6635l-.6236.4406-.3508 2.9025.3248-.2851h4.9626l.3378.2851z' fill='%23161616' stroke='%23161616' /%3E%3Cpath d='m33.5168 11.3532 1.1043-5.36447-1.6629-4.98873-12.6923 9.3944 4.8846 4.1205 6.8983 2.0085 1.52-1.7752-.6626-.4795 1.0523-.9588-.8054-.622 1.0523-.8034z' fill='%23763e1a' stroke='%23763e1a' /%3E%3Cpath d='m1 5.98873 1.11724 5.36447-.71451.5313 1.06527.8034-.80545.622 1.05228.9588-.66255.4795 1.51997 1.7752 6.89835-2.0085 4.8846-4.1205-12.69233-9.3944z' fill='%23763e1a' stroke='%23763e1a' /%3E%3Cpath d='m32.0489 16.5234-6.8983-2.0085 2.0786 3.1358-3.1049 6.0513 4.1052-.0519h6.1318z' fill='%23f5841f' stroke='%23f5841f' /%3E%3Cpath d='m10.4705 14.5149-6.89828 2.0085-2.29944 7.1267h6.11883l4.10519.0519-3.10487-6.0513z' fill='%23f5841f' stroke='%23f5841f' /%3E%3Cpath d='m19.8241 17.9876.4417-7.5932 2.0007-5.4034h-8.9119l2.0006 5.4034.4417 7.5932.1689 2.3842.013 5.8958h3.6635l.013-5.8958z' fill='%23f5841f' stroke='%23f5841f' /%3E%3C/g%3E%3C/svg%3E%0A"
            : name.includes("coinbase")
              ? "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='56' height='56' fill='none'%3E%3Cpath d='M28 56c15.464 0 28-12.536 28-28S43.464 0 28 0 0 12.536 0 28s12.536 28 28 28Z' fill='%231B53E4'/%3E%3Cpath fill-rule='evenodd' clip-rule='evenodd' d='M7 28c0 11.598 9.402 21 21 21s21-9.402 21-21S39.598 7 28 7 7 16.402 7 28Zm17.234-6.766a3 3 0 0 0-3 3v7.533a3 3 0 0 0 3 3h7.533a3 3 0 0 0 3-3v-7.533a3 3 0 0 0-3-3h-7.533Z' fill='%23fff'/%3E%3C/svg%3E"
              : connector.icon) as string,
      type: connector.type as ConnectorType,
      download: ""
    }
  })
  .filter(wallet => !excludeWalletList.includes(wallet.id)) satisfies Array<Connector>

export type EvmWalletId = (typeof evmWalletsInformation)[number]["id"]

export async function evmConnect(
  evmWalletId: EvmWalletId,
  chainId: ConfiguredChainId = sepolia.id
) {
  const connector = wagmiConfig.connectors.find(connector => connector.id === evmWalletId)
  if (connector) {
    return await _connect(wagmiConfig, { connector, chainId })
  }
  throw new Error(`Connector ${evmWalletId} not found`)
}

export async function evmDisconnect() {
  try {
    const connector = getAccount(wagmiConfig).connector
    if (connector) {
      await _disconnect(wagmiConfig, { connector })
    } else {
      await _disconnect(wagmiConfig)
    }
  } catch (error) {
    console.error("Error during disconnect:", error)
    throw error
  }
}

export const evmSwitchChain = (chainId: ConfiguredChainId) => _switchChain(wagmiConfig, { chainId })
