import { wallets } from "$lib/stores/wallets.svelte"
import { runSync } from "$lib/runtime"
import { Ucs05 } from "@unionlabs/sdk"
import { Effect, Option } from "effect"
import * as S from "effect/Schema"

import {
  getWallets,
  type WalletWithFeatures,
} from "@mysten/wallet-standard"

import { Ed25519PublicKey } from "@mysten/sui/keypairs/ed25519"

import { SuiClient } from "@mysten/sui/client"
import { Transaction } from "@mysten/sui/transactions"
import { fromB64 } from "@mysten/sui/utils"

export const suiWalletsInformation = [
  {
    id: "slush",
    name: "Slush Wallet",
    icon: "/images/icons/slush.svg", 
    deepLink: "https://slush.app",
    download: "https://slush.app",
  },
] as const

export type SuiWalletId = (typeof suiWalletsInformation)[number]["id"]


function pickSuiWallet(targetId: SuiWalletId | undefined) {
  const registry = getWallets().get()

  const byId  = targetId
    ? registry.find((w) => w.id?.toLowerCase?.() === targetId.toLowerCase())
    : undefined

  const byName = registry.find((w) => /slush/i.test(w.name))

  return byId ?? byName ?? registry[0]
}

function inferSuiChainTag(rpcUrl: string | URL | undefined) {
  const u = rpcUrl ? rpcUrl.toString() : ""
  if (/testnet/i.test(u)) return "sui:testnet"
  if (/devnet|local/i.test(u)) return "sui:devnet"
  return "sui:mainnet"
}

function makeWalletStandardSigner(
  wallet: WalletWithFeatures,
  account: (typeof wallet)["accounts"][number],
  rpcUrl?: string | URL,
) {
  const chainTag = inferSuiChainTag(rpcUrl)

  return {
    getPublicKey() {
      return new Ed25519PublicKey(fromB64(account.publicKey))
    },

    async signTransactionBlock(input: { transactionBlock: Transaction | Uint8Array }) {
      let bytes: Uint8Array

      if (input.transactionBlock instanceof Transaction) {
        const tmpClient = new SuiClient({ url: typeof rpcUrl === "string" ? rpcUrl : (rpcUrl?.toString() ?? "") })
        bytes = await input.transactionBlock.build({ client: tmpClient })
      } else {
        bytes = input.transactionBlock
      }

      const signFeature = wallet.features["sui:signTransactionBlock"]
      if (!signFeature) throw new Error("Wallet does not support sui:signTransactionBlock")

      const { signature, bytes: signedBytes } = await signFeature.signTransactionBlock({
        account,
        transactionBlock: bytes,
        chain: chainTag,
      })

      return {
        signature,
        bytes: signedBytes ?? bytes,
      }
    },
  } as unknown 
}


class SuiStore {
  address = $state<string | undefined>(undefined)
  connectedWallet = $state<SuiWalletId | undefined>(undefined)
  connectionStatus = $state<"disconnected" | "connecting" | "connected">("disconnected")
  private _signer: ReturnType<typeof makeWalletStandardSigner> | undefined
  private _account: any | undefined
  private _rpcUrl: string | undefined

  constructor() {
    this.loadFromStorage()

    if (this.connectedWallet && this.connectionStatus === "connected") {
      setTimeout(() => this.reconnect(this.connectedWallet!), 500)
    }
  }

  loadFromStorage() {
    try {
      const raw = sessionStorage.getItem("sui-store")
      if (!raw) return
      const parsed = JSON.parse(raw)
      this.address = parsed.address
      this.connectedWallet = parsed.connectedWallet
      this.connectionStatus = parsed.connectionStatus ?? "disconnected"
      if (this.address && this.connectionStatus === "connected") {
        wallets.suiAddress = S.decodeOption(Ucs05.SuiDisplay)({
          _tag: "SuiDisplay",
          address: this.address as `0x${string}`,
        })
      }
    } catch (e) {
      console.error("Failed to load Sui store:", e)
    }
  }

  saveToStorage() {
    try {
      sessionStorage.setItem(
        "sui-store",
        JSON.stringify({
          address: this.address,
          connectedWallet: this.connectedWallet,
          connectionStatus: this.connectionStatus,
        }),
      )
    } catch (e) {
      console.error("Failed to save Sui store:", e)
    }
  }

  async connect(walletId: SuiWalletId = "slush", rpcUrl?: string) {
    this.connectionStatus = "connecting"
    this.saveToStorage()

    const wallet = pickSuiWallet(walletId)
    if (!wallet) {
      this.connectionStatus = "disconnected"
      this.saveToStorage()
      window.open("https://slushwallet.xyz", "_blank", "noopener,noreferrer")
      return
    }

    try {
      const connectFeature = wallet.features["standard:connect"]
      if (!connectFeature) throw new Error("Wallet does not support standard:connect")

        await connectFeature.connect()
      const account = wallet.accounts[0]
      if (!account) throw new Error("No Sui account returned by wallet")

      this._account = account
      this._rpcUrl = rpcUrl
      this._signer = makeWalletStandardSigner(wallet, account, rpcUrl)

      this.address = account.address
      this.connectedWallet = "slush"
      this.connectionStatus = "connected"


      wallets.suiAddress = S.decodeOption(Ucs05.SuiDisplay)({
        _tag: "SuiDisplay",
        address: account.address as `0x${string}`,
      })

      this.saveToStorage()

      Effect.log("wallet.connect (sui)").pipe(runSync)
    } catch (e) {
      console.error("Sui connect failed:", e)
      this.connectionStatus = "disconnected"
      this.connectedWallet = undefined
      this.address = undefined
      wallets.suiAddress = Option.none()
      this.saveToStorage()
    }
  }

  async reconnect(walletId: SuiWalletId = "slush") {
    return this.connect(walletId, this._rpcUrl)
  }


  disconnect = async () => {
    try {
      console.log("Sui disconnecting...")
      this.connectedWallet = undefined
      this.connectionStatus = "disconnected"
      this.address = undefined
      this._signer = undefined
      this._account = undefined
      wallets.suiAddress = Option.none()
      sessionStorage.removeItem("sui-store")
    } catch (e) {
      console.error("Sui disconnect failed:", e)
    }
  }

  
  getSuiSigner() {
    if (!this._signer || !this.address) {
      return Option.none<typeof this._signer>()
    }
    return Option.some(this._signer)
  }
}

export const suiStore = new SuiStore()
