import { runFork, runPromise } from "$lib/runtime"
import { Duration, Effect, Fiber, Option, pipe } from "effect"
import type { Entity } from "../client"
import { WalletError } from "../errors"
import { getWalletsByUserId, removeUserWallet } from "../queries/private"
import type { Wallet } from "../queries/private"
import { getChains } from "../queries/public"
import { errorStore } from "../stores/errors.svelte"

const POLL_INTERVAL = 5 * 60_000

function getWalletCategory(chainId: string): "evm" | "cosmos" | "other" {
  if (!chainId) {
    return "other"
  }
  const lowerChainId = chainId.toLowerCase()
  if (lowerChainId.startsWith("evm")) {
    return "evm"
  }
  if (lowerChainId.startsWith("cosmos:")) {
    return "cosmos"
  }
  return "other"
}

export type Chain = Entity<"chains">

export type EnhancedWallet = Wallet & {
  hasGrouping: boolean
  createdAt: Date
}

export class WalletStore {
  /** User\'s wallets */
  wallets = $state<Option.Option<Array<Wallet>>>(Option.none())

  /** Available chains */
  chains = $state<Option.Option<Array<Chain>>>(Option.none())

  /** Polling fiber */
  private pollFiber: Fiber.Fiber<never, Error> | null = null

  /**
   * Enhanced wallet data with additional computed properties
   */
  enhanced = $derived(
    Option.flatMap(this.wallets, (wallets) => {
      return Option.some(
        wallets.map((wallet) => ({
          ...wallet,
          hasGrouping: wallet.grouping !== null,
          createdAt: new Date(wallet.created_at),
        } as EnhancedWallet)),
      )
    }).pipe(
      Option.getOrElse(() => {
        return [] as Array<EnhancedWallet>
      }),
    ),
  )

  /**
   * Wallets grouped by chain
   */
  byChain = $derived(
    this.enhanced.reduce<Record<string, Array<EnhancedWallet>>>((acc, wallet) => {
      const chain = wallet.chain_id
      if (!acc[chain]) {
        acc[chain] = []
      }
      acc[chain].push(wallet)
      return acc
    }, {}),
  )

  /**
   * Wallets with grouping
   */
  grouped = $derived(
    this.enhanced.filter((w) => w.hasGrouping),
  )

  /**
   * Wallets without grouping
   */
  ungrouped = $derived(
    this.enhanced.filter((w) => !w.hasGrouping),
  )

  /**
   * Wallet statistics
   */
  stats = $derived({
    total: this.enhanced.length,
    evmCount: this.enhanced.filter(w => getWalletCategory(w.chain_id) === "evm").length,
    cosmosCount: this.enhanced.filter(w => getWalletCategory(w.chain_id) === "cosmos").length,
    otherCount: this.enhanced.filter(w => getWalletCategory(w.chain_id) === "other").length,
  })

  constructor(private readonly userId: string) {
    this.loadWallets(userId)
    this.loadChains()
    this.startPolling(userId)
  }

  /**
   * Loads user wallets data
   * @param userId - The ID of the user to load wallets for
   * @private
   */
  private loadWallets(userId: string) {
    runPromise(
      pipe(
        getWalletsByUserId(userId),
        Effect.tap((result) => {
          this.wallets = result
          return Effect.void
        }),
        Effect.catchAll((error) => {
          errorStore.showError(new WalletError({ cause: error, operation: "load" }))
          return Effect.succeed(Option.none())
        }),
      ),
    )
  }

  /**
   * Loads chains data
   * @private
   */
  private loadChains() {
    runPromise(
      pipe(
        getChains(),
        Effect.tap((result) => {
          this.chains = result
          return Effect.void
        }),
        Effect.catchAll((error) => {
          errorStore.showError(new WalletError({ cause: error, operation: "loadChains" }))
          return Effect.succeed(Option.none())
        }),
      ),
    )
  }

  /**
   * Starts polling for wallet updates
   * @param userId - The user ID to poll wallets for
   * @private
   */
  private startPolling(userId: string) {
    this.stopPolling()

    const self = this
    this.pollFiber = runFork(
      Effect.forever(
        pipe(
          getWalletsByUserId(userId),
          Effect.tap((result) => {
            self.wallets = result
            return Effect.void
          }),
          Effect.catchAll((error) => {
            errorStore.showError(new WalletError({ cause: error, operation: "load" }))
            return Effect.succeed(Option.none())
          }),
          Effect.delay(Duration.millis(POLL_INTERVAL)),
        ),
      ),
    )
  }

  /**
   * Stops polling for wallet updates
   * @private
   */
  private stopPolling() {
    if (this.pollFiber) {
      runPromise(Fiber.interrupt(this.pollFiber))
      this.pollFiber = null
    }
  }

  /**
   * Refreshes all wallet data for the current user
   */
  refresh() {
    this.loadWallets(this.userId)
    this.loadChains()
  }

  /**
   * Cleans up resources when the store is no longer needed
   */
  cleanup() {
    this.stopPolling()
    this.wallets = Option.none()
    this.chains = Option.none()
  }

  /**
   * Removes a wallet from the current user
   * @param address - The wallet address to remove
   * @returns An Effect that resolves to true if the wallet was removed successfully
   */
  removeWallet(address: string) {
    return pipe(
      removeUserWallet(this.userId, address),
      Effect.flatMap((dbSuccess) => {
        if (dbSuccess) {
          const currentWallets = Option.getOrElse(this.wallets, () => [] as Array<Wallet>)
          this.wallets = Option.some(currentWallets.filter(wallet => wallet.address !== address))
          this.refresh()
          return Effect.succeed(true)
        }
        return Effect.succeed(false)
      }),
      Effect.catchAll((error) => {
        errorStore.showError(new WalletError({ cause: error, operation: "remove" }))
        return Effect.succeed(false)
      }),
    )
  }

  // Method to remove an entire wallet group by its groupingId
  removeWalletGroup(groupingId: string) {
    if (!groupingId) {
      return Effect.succeed(false)
    }

    const walletsInGroup = this.enhanced.filter(w => w.grouping === groupingId)
    if (walletsInGroup.length === 0) {
      return Effect.succeed(true)
    }

    const removalEffects = walletsInGroup.map(wallet =>
      removeUserWallet(this.userId, wallet.address)
    )

    return pipe(
      Effect.all(removalEffects, { concurrency: "inherit", discard: false }),
      Effect.flatMap((results) => {
        const allSucceeded = results.every(res => res)
        if (allSucceeded) {
          const currentWallets = Option.getOrElse(this.wallets, () => [] as Array<Wallet>)
          this.wallets = Option.some(
            currentWallets.filter(wallet => wallet.grouping !== groupingId),
          )
          this.refresh()
          return Effect.succeed(true)
        }
        this.refresh()
        return Effect.succeed(false)
      }),
      Effect.catchAll((error) => {
        this.refresh()
        errorStore.showError(new WalletError({ cause: error, operation: "remove" }))
        return Effect.succeed(false)
      }),
    )
  }
}
