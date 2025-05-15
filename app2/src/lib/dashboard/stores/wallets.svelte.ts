import { Effect, Option, Fiber, Duration, pipe } from "effect";
import { getWalletsByUserId, insertWalletData, removeUserWallet } from "../queries/private";
import type { Wallet } from "../queries/private";
import { getChains } from "../queries/public";
import type { Entity } from "../client";
import { extractErrorDetails } from "@unionlabs/sdk/utils";
import { WalletError } from "../errors";

const POLL_INTERVAL = 5 * 60_000;

function getWalletCategory(chainId: string): 'evm' | 'cosmos' | 'other' {
  if (!chainId) return 'other';
  const lowerChainId = chainId.toLowerCase();
  if (lowerChainId === 'evm') return 'evm';
  if (lowerChainId.startsWith('cosmos:')) return 'cosmos';
  return 'other'; 
}

export type Chain = Entity<"chains">;

export type EnhancedWallet = Wallet & {
  hasGrouping: boolean;
  createdAt: Date;
};

export class WalletStore {
  /** User\'s wallets */
  wallets = $state<Option.Option<Array<Wallet>>>(Option.none());

  /** Available chains */
  chains = $state<Option.Option<Array<Chain>>>(Option.none());

  /** Polling fiber */
  private pollFiber: Fiber.Fiber<never, Error> | null = null;

  /**
   * Enhanced wallet data with additional computed properties
   * @example
   * \`\`\`ts
   * dashboard.wallets.enhanced // Get all wallets with computed properties
   * \`\`\`
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
   * \`\`\`ts
   * dashboard.wallets.byChain // Get wallets grouped by chain
   * \`\`\`
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
   * \`\`\`ts
   * dashboard.wallets.grouped // Get wallets that have a grouping
   * \`\`\`
   */
  grouped = $derived(
    this.enhanced.filter((w) => w.hasGrouping)
  );

  /**
   * Wallets without grouping
   * @example
   * \`\`\`ts
   * dashboard.wallets.ungrouped // Get wallets without grouping
   * \`\`\`
   */
  ungrouped = $derived(
    this.enhanced.filter((w) => !w.hasGrouping)
  );

  /**
   * Wallet statistics
   * @example
   * \`\`\`ts
   * dashboard.wallets.stats // Get wallet statistics
   * \`\`\`
   */
  stats = $derived({
    total: this.enhanced.length,
    evmCount: this.enhanced.filter(w => getWalletCategory(w.chain_id) === 'evm').length,
    cosmosCount: this.enhanced.filter(w => getWalletCategory(w.chain_id) === 'cosmos').length,
    otherCount: this.enhanced.filter(w => getWalletCategory(w.chain_id) === 'other').length,
  });

  constructor(private readonly userId: string) {
    console.log("[wallet] Initializing WalletStore for user:", userId);
    this.loadWallets(userId);
    this.loadChains();
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
   * Loads chains data
   * @private
   */
  private loadChains() {
    Effect.runPromise(
      pipe(
        getChains(),
        Effect.tap((result) => {
          console.log("[wallet] Chains loaded:", result);
          this.chains = result;
          return Effect.void;
        }),
        Effect.catchAll((error) => 
          Effect.fail(new WalletError({ 
            cause: extractErrorDetails(error), 
            operation: "loadChains" 
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
    this.loadChains();
  }

  /**
   * Cleans up resources when the store is no longer needed
   */
  cleanup() {
    this.stopPolling();
    this.wallets = Option.none();
    this.chains = Option.none();
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
          const currentWallets = Option.getOrElse(this.wallets, () => [] as Array<Wallet>);
          this.wallets = Option.some(currentWallets.filter(wallet => wallet.address !== address));
          
          console.log(`[wallet] Optimistically removed ${address}. Forcing store refresh.`);
          this.refresh();

          return Effect.succeed(true);
        }
        return Effect.succeed(false);
      }),
      Effect.catchAll((error) => 
        Effect.fail(new WalletError({ 
          cause: extractErrorDetails(error), 
          operation: "remove" 
        }))
      )
    );
  }

  // Method to remove an entire wallet group by its groupingId
  removeWalletGroup(groupingId: string) {
    if (!groupingId) return Effect.succeed(false); // Or Effect.fail if this is an error state

    const walletsInGroup = this.enhanced.filter(w => w.grouping === groupingId);
    if (walletsInGroup.length === 0) return Effect.succeed(true); // No wallets in group, consider it success

    const removalEffects = walletsInGroup.map(wallet => 
      removeUserWallet(this.userId, wallet.address)
    );

    return pipe(
      Effect.all(removalEffects, { concurrency: "inherit", discard: false }), // discard:false to get all results
      Effect.flatMap((results) => {
        // Check if all removals were successful (or handle partial success)
        const allSucceeded = results.every(res => res); // Assuming removeUserWallet returns boolean Effect
        if (allSucceeded) {
          const currentWallets = Option.getOrElse(this.wallets, () => [] as Array<Wallet>);
          this.wallets = Option.some(
            currentWallets.filter(wallet => wallet.grouping !== groupingId)
          );
          console.log(`[wallet] Optimistically removed group ${groupingId}. Forcing store refresh.`);
          this.refresh(); 
          return Effect.succeed(true);
        }
        // Handle partial failure if necessary, for now, let's say if not all succeed, it's a failure
        console.error(`[wallet] Failed to remove some wallets in group ${groupingId}`);
        // Optionally, you could try to refresh to get the actual state from DB
        this.refresh(); 
        return Effect.succeed(false); // Or Effect.fail
      }),
      Effect.catchAll((error) => {
        console.error(`[wallet] Error removing wallet group ${groupingId}:`, error);
        this.refresh(); // Refresh to sync with DB state after error
        return Effect.fail(new WalletError({ 
          cause: extractErrorDetails(error),
          operation: "remove"
        }));
      })
    );
  }
}
