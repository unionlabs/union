import { Effect, Option, Fiber, Duration, pipe } from "effect";
import { getWalletsByUserId, insertWalletData, removeUserWallet } from "../queries/private";
import type { Wallet } from "../queries/private";
import { extractErrorDetails } from "@unionlabs/sdk/utils";
import { WalletError } from "../errors";

const POLL_INTERVAL = 5 * 60_000;

type EnhancedWallet = Wallet & {
  hasGrouping: boolean;
  createdAt: Date;
};

export class WalletStore {
  /** User's wallets */
  wallets = $state<Option.Option<Array<Wallet>>>(Option.none());

  /** Polling fiber */
  private pollFiber: Fiber.Fiber<never, Error> | null = null;

  /**
   * Enhanced wallet data with additional computed properties
   * @example
   * ```ts
   * dashboard.wallets.enhanced // Get all wallets with computed properties
   * ```
   */
  enhanced = $derived(
    Option.flatMap(this.wallets, (wallets) => {
      return Option.some(
        wallets.map((wallet) => ({
          ...wallet,
          hasGrouping: wallet.grouping !== null,
          createdAt: new Date(wallet.created_at),
        } as EnhancedWallet))
      );
    }).pipe(
      Option.getOrElse(() => {
        console.log("[wallet] No enhanced wallets available");
        return [] as Array<EnhancedWallet>;
      })
    )
  );

  /**
   * Wallets grouped by chain
   * @example
   * ```ts
   * dashboard.wallets.byChain // Get wallets grouped by chain
   * ```
   */
  byChain = $derived(
    this.enhanced.reduce<Record<string, Array<EnhancedWallet>>>((acc, wallet) => {
      const chain = wallet.chain_id;
      if (!acc[chain]) {
        acc[chain] = [];
      }
      acc[chain].push(wallet);
      return acc;
    }, {})
  );

  /**
   * Wallets with grouping
   * @example
   * ```ts
   * dashboard.wallets.grouped // Get wallets that have a grouping
   * ```
   */
  grouped = $derived(
    this.enhanced.filter((w) => w.hasGrouping)
  );

  /**
   * Wallets without grouping
   * @example
   * ```ts
   * dashboard.wallets.ungrouped // Get wallets without grouping
   * ```
   */
  ungrouped = $derived(
    this.enhanced.filter((w) => !w.hasGrouping)
  );

  /**
   * Wallet statistics
   * @example
   * ```ts
   * dashboard.wallets.stats // Get wallet statistics
   * ```
   */
  stats = $derived({
    total: this.enhanced.length,
    grouped: this.grouped.length,
    ungrouped: this.ungrouped.length,
    chains: Object.keys(this.byChain).length,
  });

  constructor(private readonly userId: string) {
    console.log("[wallet] Initializing WalletStore for user:", userId);
    this.loadWallets(userId);
    this.startPolling(userId);
  }

  /**
   * Loads user wallets data
   * @param userId - The ID of the user to load wallets for
   * @private
   */
  private loadWallets(userId: string) {
    Effect.runPromise(
      pipe(
        getWalletsByUserId(userId),
        Effect.tap((result) => {
          console.log("[wallet] User wallets loaded:", result);
          this.wallets = result;
          return Effect.void;
        }),
        Effect.catchAll((error) => 
          Effect.fail(new WalletError({ 
            cause: extractErrorDetails(error), 
            operation: "load" 
          }))
        )
      )
    );
  }

  /**
   * Starts polling for wallet updates
   * @param userId - The user ID to poll wallets for
   * @private
   */
  private startPolling(userId: string) {
    this.stopPolling(); // Make sure to stop any existing poll

    // Start polling fiber
    const self = this;
    this.pollFiber = Effect.runFork(
      Effect.forever(
        pipe(
          getWalletsByUserId(userId),
          Effect.tap((result) => {
            self.wallets = result;
            return Effect.void;
          }),
          Effect.catchAll((error) => 
            Effect.fail(new WalletError({ 
              cause: extractErrorDetails(error), 
              operation: "load" 
            }))
          ),
          Effect.delay(Duration.millis(POLL_INTERVAL))
        )
      )
    );
  }

  /**
   * Stops polling for wallet updates
   * @private
   */
  private stopPolling() {
    if (this.pollFiber) {
      Effect.runPromise(Fiber.interrupt(this.pollFiber));
      this.pollFiber = null;
    }
  }

  /**
   * Refreshes all wallet data for the current user
   */
  refresh() {
    this.loadWallets(this.userId);
  }

  /**
   * Cleans up resources when the store is no longer needed
   */
  cleanup() {
    this.stopPolling();
    this.wallets = Option.none();
  }

  /**
   * Adds a new wallet for the current user
   * @param address - The wallet address to add
   * @param chainId - The chain ID the wallet belongs to
   * @returns An Effect that resolves to the added wallet if successful
   */
  addWallet(address: string, chainId: string) {
    return pipe(
      insertWalletData({
        address,
        chain_id: chainId,
        user_id: this.userId,
      }),
      Effect.tap((result) => 
        Effect.sync(() => {
          if (result) {
            // Update the wallets state with the new wallet
            Option.match(this.wallets, {
              onNone: () => {
                this.wallets = Option.some([result]);
              },
              onSome: (wallets) => {
                this.wallets = Option.some([...wallets, result]);
              }
            });
          }
        })
      ),
      Effect.catchAll((error) => 
        Effect.fail(new WalletError({ 
          cause: extractErrorDetails(error), 
          operation: "add" 
        }))
      )
    );
  }

  /**
   * Removes a wallet from the current user
   * @param address - The wallet address to remove
   * @returns An Effect that resolves to true if the wallet was removed successfully
   */
  removeWallet(address: string) {
    return pipe(
      removeUserWallet(this.userId, address),
      Effect.tap((success) => 
        Effect.sync(() => {
          if (success) {
            // Update the wallets state by removing the wallet
            Option.match(this.wallets, {
              onNone: () => {},
              onSome: (wallets) => {
                this.wallets = Option.some(
                  wallets.filter(wallet => wallet.address !== address)
                );
              }
            });
          }
        })
      ),
      Effect.catchAll((error) => 
        Effect.fail(new WalletError({ 
          cause: extractErrorDetails(error), 
          operation: "remove" 
        }))
      )
    );
  }
}
