import { xionTestnet } from "./chains"
import { KeplrChainInfoFromInternal, LeapChainInfoFromInternal } from "./transform"
import { cosmosWalletsInformation, type CosmosWalletId } from "./wallets"
import { Effect, Schema as S, Context, Layer, Ref, Option, Data } from "effect"

export type ConnectionStatus = "disconnected" | "connecting" | "connected"

export class WalletNotFoundError extends Data.TaggedError("WalletNotFoundError")<{
  readonly walletId: CosmosWalletId
}> {}

export class WalletConnectionError extends Data.TaggedError("WalletConnectionError")<{
  readonly walletId: CosmosWalletId
  readonly cause: unknown
}> {}

export interface CosmosWalletState {
  readonly chain: string
  readonly address: Option.Option<string>
  readonly rawAddress: Option.Option<Uint8Array>
  readonly connectedWallet: Option.Option<CosmosWalletId>
  readonly connectionStatus: ConnectionStatus
}

export class CosmosWallet extends Context.Tag("CosmosWallet")<
  CosmosWallet,
  {
    readonly connect: (walletId: CosmosWalletId) => Effect.Effect<
      void,
      WalletNotFoundError | WalletConnectionError
    >
    readonly disconnect: () => Effect.Effect<void>
    readonly getState: () => Effect.Effect<CosmosWalletState>
    readonly getAddress: () => Effect.Effect<Option.Option<string>>
    readonly getConnectionStatus: () => Effect.Effect<ConnectionStatus>
  }
>() {}

// Helper functions
const getChainInfo = (walletId: CosmosWalletId) => {
  const result = walletId === "keplr" 
    ? S.decodeSync(KeplrChainInfoFromInternal)(xionTestnet)
    : S.decodeSync(LeapChainInfoFromInternal)(xionTestnet)
  
  // Transform functions return [transformedChainInfo, originalInfo] - we want the first element
  return result[0]
}

const openWalletDownload = (walletId: CosmosWalletId) => {
  const walletInfo = cosmosWalletsInformation.find(w => w.id === walletId)
  if (walletInfo) {
    const { deepLink, download } = walletInfo
    window.open(deepLink || download, "_blank", "noopener noreferrer")
  }
}

// Service implementation
const makeCosmosWallet = Effect.gen(function* () {
  // State ref
  const stateRef = yield* Ref.make<CosmosWalletState>({
    chain: "cosmos",
    address: Option.none(),
    rawAddress: Option.none(),
    connectedWallet: Option.none(),
    connectionStatus: "disconnected",
  })

  // Helper to update state and save to storage
  const updateAndSave = (updater: (state: CosmosWalletState) => CosmosWalletState) =>
    Effect.gen(function* () {
      yield* Ref.update(stateRef, updater)
      const newState = yield* Ref.get(stateRef)
      
      // Save to storage (localStorage for persistence across sessions)
      yield* Effect.sync(() => {
        try {
          const storeData = {
            chain: newState.chain,
            address: Option.isSome(newState.address) ? newState.address.value : undefined,
            rawAddress: Option.isSome(newState.rawAddress) ? Array.from(newState.rawAddress.value) : undefined,
            connectedWallet: Option.isSome(newState.connectedWallet) ? newState.connectedWallet.value : undefined,
            connectionStatus: newState.connectionStatus,
          }
          localStorage.setItem("cosmos-wallet-connection", JSON.stringify(storeData))
        } catch (e) {
          console.error("Failed to save cosmos store", e)
        }
      })
    })

    // Load initial state from storage (localStorage for persistence)
  yield* Effect.sync(() => {
    try {
      const storedData = localStorage.getItem("cosmos-wallet-connection")
      if (storedData) {
        const parsedData = JSON.parse(storedData)
        const initialState: CosmosWalletState = {
          chain: parsedData.chain || "cosmos",
          address: parsedData.address ? Option.some(parsedData.address) : Option.none(),
          rawAddress: parsedData.rawAddress ? Option.some(new Uint8Array(parsedData.rawAddress)) : Option.none(),
          connectedWallet: parsedData.connectedWallet ? Option.some(parsedData.connectedWallet) : Option.none(),
          connectionStatus: parsedData.connectionStatus || "disconnected",
        }
        Effect.runSync(Ref.set(stateRef, initialState))
      }
    } catch (e) {
      console.error("Failed to load cosmos store", e)
    }
  })

  const connect = (walletId: CosmosWalletId) =>
    Effect.gen(function* () {
      // Update to connecting
      yield* updateAndSave((state) => ({
        ...state,
        connectionStatus: "connecting" as const,
        connectedWallet: Option.some(walletId),
      }))

      const walletApi = window[walletId]
      if (!walletApi) {
        openWalletDownload(walletId)
        yield* Effect.fail(new WalletNotFoundError({ walletId }))
      }

      const chainInfo = getChainInfo(walletId)

      yield* Effect.tryPromise({
        try: () => walletApi!.experimentalSuggestChain(chainInfo),
        catch: (cause) => new WalletConnectionError({ walletId, cause }),
      })

      yield* Effect.tryPromise({
        try: () => walletApi!.enable(["xion-testnet-2"]),
        catch: (cause) => new WalletConnectionError({ walletId, cause }),
      })

      const account = yield* Effect.tryPromise({
        try: () => walletApi!.getKey("xion-testnet-2"),
        catch: (cause) => new WalletConnectionError({ walletId, cause }),
      })

      // Update with connected state
      yield* updateAndSave((state) => ({
        ...state,
        connectionStatus: "connected" as const,
        address: Option.some(account.bech32Address),
        rawAddress: Option.some(account.address),
      }))
    })

  const disconnect = () =>
    Effect.gen(function* () {
      const state = yield* Ref.get(stateRef)
      
      if (Option.isSome(state.connectedWallet)) {
        const walletId = state.connectedWallet.value
        const walletApi = window[walletId]
        
        if (walletApi) {
          if (walletId === "keplr" && "disable" in walletApi) {
            yield* Effect.tryPromise(() => walletApi.disable("xion-testnet-2")).pipe(
              Effect.orElse(() => Effect.void)
            )
          } else if (walletId === "leap" && "disconnect" in walletApi) {
            yield* Effect.tryPromise(() => walletApi.disconnect("xion-testnet-2")).pipe(
              Effect.orElse(() => Effect.void)
            )
          }
        }
      }

      yield* updateAndSave(() => ({
        chain: "cosmos",
        address: Option.none(),
        rawAddress: Option.none(),
        connectedWallet: Option.none(),
        connectionStatus: "disconnected",
      }))

      yield* Effect.sync(() => localStorage.removeItem("cosmos-wallet-connection"))
    })

  return {
    connect,
    disconnect,
    getState: () => Ref.get(stateRef),
    getAddress: () => Effect.map(Ref.get(stateRef), (state) => state.address),
    getConnectionStatus: () => Effect.map(Ref.get(stateRef), (state) => state.connectionStatus),
  }
})

export const CosmosWalletServiceLive = Layer.effect(CosmosWallet, makeCosmosWallet)
const serviceInstance = Effect.provide(makeCosmosWallet, CosmosWalletServiceLive)

export const cosmosStore = {
  connect: (walletId: CosmosWalletId) =>
    Effect.flatMap(serviceInstance, (service) => service.connect(walletId)),
  
  disconnect: () =>
    Effect.flatMap(serviceInstance, (service) => service.disconnect()),
    
  getState: () =>
    Effect.flatMap(serviceInstance, (service) => service.getState()),
    
  getAddress: () =>
    Effect.flatMap(serviceInstance, (service) => service.getAddress()),
    
  getConnectionStatus: () =>
    Effect.flatMap(serviceInstance, (service) => service.getConnectionStatus()),
}
