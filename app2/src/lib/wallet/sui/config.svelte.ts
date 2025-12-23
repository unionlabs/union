import { runSync } from "$lib/runtime"
import { wallets } from "$lib/stores/wallets.svelte"
import { Ucs05 } from "@unionlabs/sdk"
import { Effect, Option } from "effect"
import * as S from "effect/Schema"

import { getWallets } from "@mysten/wallet-standard"
import type { SuiWalletFeatures, WalletWithFeatures } from "@mysten/wallet-standard"

import { Ed25519PublicKey } from "@mysten/sui/keypairs/ed25519"
import { Transaction } from "@mysten/sui/transactions"

type AnyWallet = WalletWithFeatures<SuiWalletFeatures & Record<string, unknown>>

export type SuiWalletId = string

function getDetectedSuiWallets() {
  return getWallets()
    .get()
    .filter(w => Object.keys(w.features || {}).some(f => f.startsWith("sui:")))
    .map(w => ({
      id: w.name.toLowerCase().replace(/\s+/g, "-"),
      name: w.name,
      icon: w.icon ?? "/images/icons/injected.svg",
    }))
}

export const suiWalletsInformation = $state<ReturnType<typeof getDetectedSuiWallets>>([])

if (typeof window !== "undefined") {
  const walletsApi = getWallets()

  const refreshWallets = () => {
    suiWalletsInformation.length = 0
    suiWalletsInformation.push(...getDetectedSuiWallets())
  }

  refreshWallets()
  walletsApi.on("register", refreshWallets)
  walletsApi.on("unregister", refreshWallets)
}

function pickSuiWallet(targetId: SuiWalletId | undefined) {
  if (!targetId) {
    return undefined
  }

  const suiWallets = getWallets()
    .get()
    .filter(w => Object.keys(w.features || {}).some(f => f.startsWith("sui:")))

  const searchTerm = targetId.replace(/-/g, " ")

  return suiWallets.find(
    w =>
      w.name.toLowerCase().includes(searchTerm.toLowerCase())
      || searchTerm.toLowerCase().includes(w.name.toLowerCase().replace(/\s+/g, " ")),
  )
}

function createSigner(
  wallet: AnyWallet,
  account: AnyWallet["accounts"][number],
) {
  async function signTransaction(
    input: { transaction: Transaction; chain?: `${string}:${string}` },
  ) {
    const { transaction, chain = "sui:mainnet" } = input

    const signAndExecute = wallet.features["sui:signAndExecuteTransaction"]
    if (signAndExecute) {
      const resp = await signAndExecute.signAndExecuteTransaction({
        account,
        transaction,
        chain,
      })
      return { kind: "executed" as const, executeResult: resp }
    }

    const signOnly = wallet.features["sui:signTransaction"]
    if (signOnly) {
      const { signature, bytes } = await signOnly.signTransaction({
        account,
        transaction,
        chain,
      })
      return { kind: "signed" as const, signature, bytes }
    }

    return { kind: "error" as const, error: "Wallet does not support transaction signing" }
  }

  return {
    getPublicKey: () => new Ed25519PublicKey(new Uint8Array(account.publicKey)),
    toSuiAddress: () => account.address,
    signTransaction,
  }
}

function getConnectionErrorMessage(error: unknown): string | undefined {
  const message = error instanceof Error ? error.message : String(error)
  if (message.includes("rejected") || message.includes("cancelled")) {
    return undefined
  }
  return message
}

class SuiStore {
  address = $state<string | undefined>(undefined)
  connectedWallet = $state<SuiWalletId | undefined>(undefined)
  connectionStatus = $state<"disconnected" | "connecting" | "connected">("disconnected")
  connectionError = $state<string | undefined>(undefined)
  errorWalletId = $state<SuiWalletId | undefined>(undefined)

  private _signer: ReturnType<typeof createSigner> | undefined
  private _account: AnyWallet["accounts"][number] | undefined

  constructor() {
    this.loadFromStorage()
    if (this.connectedWallet && this.connectionStatus === "connected") {
      setTimeout(() => this.reconnect(this.connectedWallet!), 500)
    }
  }

  private loadFromStorage() {
    try {
      const raw = sessionStorage.getItem("sui-store")
      if (!raw) {
        return
      }

      const data = JSON.parse(raw)
      this.address = data.address
      this.connectedWallet = data.connectedWallet
      this.connectionStatus = data.connectionStatus ?? "disconnected"

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

  private saveToStorage() {
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

  connect = async (walletId: SuiWalletId) => {
    const annotate = Effect.annotateLogs({ wallet: walletId })

    this.connectionStatus = "connecting"
    this.connectedWallet = walletId
    this.connectionError = undefined
    this.errorWalletId = undefined
    this.saveToStorage()

    const wallet = pickSuiWallet(walletId)

    if (!wallet) {
      this.connectionStatus = "disconnected"
      this.connectedWallet = undefined
      this.connectionError = "Wallet not found"
      this.errorWalletId = walletId
      this.saveToStorage()
      return
    }

    try {
      const suiWallet = wallet as WalletWithFeatures<SuiWalletFeatures>
      const connectFeature = suiWallet.features["standard:connect"] as {
        connect: (input?: { silent?: boolean }) => Promise<{ accounts: readonly unknown[] }>
      }

      if (!connectFeature) {
        throw new Error("Wallet does not support standard:connect")
      }

      const res = await connectFeature.connect({ silent: false })
      const accounts = res?.accounts ?? wallet.accounts
      const account = accounts?.[0] as AnyWallet["accounts"][number] | undefined

      if (!account) {
        throw new Error("No Sui account returned by wallet")
      }

      this._account = account
      this._signer = createSigner(suiWallet, account)
      this.address = account.address
      this.connectedWallet = walletId
      this.connectionStatus = "connected"

      wallets.suiAddress = S.decodeOption(Ucs05.SuiDisplay)({
        _tag: "SuiDisplay",
        address: account.address as `0x${string}`,
      })

      this.saveToStorage()
      Effect.log("wallet.connect").pipe(annotate, runSync)
    } catch (e) {
      Effect.logError("wallet.connect", e).pipe(annotate, runSync)

      this.connectionStatus = "disconnected"
      this.connectedWallet = undefined
      this.address = undefined
      this._signer = undefined
      this._account = undefined
      wallets.suiAddress = Option.none()

      this.connectionError = getConnectionErrorMessage(e)
      this.errorWalletId = this.connectionError ? walletId : undefined

      this.saveToStorage()
    }
  }

  reconnect = async (walletId: SuiWalletId) => {
    return this.connect(walletId)
  }

  disconnect = async () => {
    try {
      const wallet = this.connectedWallet ? pickSuiWallet(this.connectedWallet) : undefined
      const disconnectFeature = wallet?.features?.["standard:disconnect"] as
        | { disconnect: () => Promise<void> }
        | undefined

      if (disconnectFeature) {
        await disconnectFeature.disconnect().catch(() => {})
      }
    } finally {
      this.connectedWallet = undefined
      this.connectionStatus = "disconnected"
      this.address = undefined
      this._signer = undefined
      this._account = undefined
      wallets.suiAddress = Option.none()
      sessionStorage.removeItem("sui-store")
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
