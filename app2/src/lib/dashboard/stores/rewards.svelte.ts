import { Effect, Option, Fiber, Duration, pipe } from "effect";
import { getUserRewards, getAvailableRewards } from "../queries/index";
import type { Entity } from "../client";

export type Reward = Entity<"rewards">;
export type UserReward = Entity<"user_rewards_with_queue">;

const POLL_INTERVAL = 5 * 60_000;

export class RewardsStore {
  /** User's earned rewards */
  earned = $state<Option.Option<UserReward[]>>(Option.none());

  /** All available rewards in the system */
  availableRewards = $state<Option.Option<Reward[]>>(Option.none());

  /** Polling fiber */
  private pollFiber: Fiber.Fiber<never, Error> | null = null;

  /**
   * Rewards enhanced with user progress and status information
   * @example
   * ```ts
   * dashboard.rewards.enhanced // Get all rewards with progress
   * ```
   */
  enhanced = $derived(
    Option.flatMap(this.availableRewards, (rewards) =>
      Option.flatMap(this.earned, (userRewards) => {
        console.log("[reward] Computing enhanced rewards:", {
          available: rewards,
          earned: userRewards
        });
        return Option.some(
          rewards.map((reward) => {
            const userReward = userRewards.find((ur) =>
              ur.reward_id === reward.id
            );
            const isClaimed = userReward?.created_at != null;
            const isQueued = userReward?.enqueued_at != null;
            const isHandled = userReward?.handled ?? false;
            const requiresHandling = reward.default_requires_handling ?? false;

            console.log("[reward] Processing reward:", {
              id: reward.id,
              title: reward.title,
              isClaimed,
              isQueued,
              isHandled,
              requiresHandling
            });

            return {
              ...reward,
              claimed: isClaimed,
              queued: isQueued,
              handled: isHandled,
              claimed_at: userReward?.created_at,
              queued_at: userReward?.enqueued_at,
              handled_at: userReward?.handled ? userReward?.created_at : null,
              status: isHandled ? "handled" : 
                     isClaimed ? "claimed" : 
                     isQueued ? "queued" : 
                     "available"
            };
          })
        );
      })
    ).pipe(
      Option.getOrElse(() => {
        console.log("[reward] No enhanced rewards available");
        return [];
      })
    )
  );

  /**
   * Rewards that have been claimed by the user
   * @example
   * ```ts
   * dashboard.rewards.claimed // Get claimed rewards
   * ```
   */
  claimed = $derived(
    this.enhanced
      .filter((r) => r.claimed)
      .sort((a, b) => {
        const dateA = new Date(a.claimed_at ?? 0);
        const dateB = new Date(b.claimed_at ?? 0);
        return dateB.getTime() - dateA.getTime();
      })
  );

  /**
   * Rewards that are currently queued for processing
   * @example
   * ```ts
   * dashboard.rewards.queued // Get queued rewards
   * ```
   */
  queued = $derived(
    this.enhanced
      .filter((r) => r.queued && !r.handled)
      .sort((a, b) => {
        const dateA = new Date(a.queued_at ?? 0);
        const dateB = new Date(b.queued_at ?? 0);
        return dateA.getTime() - dateB.getTime();
      })
  );

  /**
   * Rewards that have been fully handled
   * @example
   * ```ts
   * dashboard.rewards.handled // Get handled rewards
   * ```
   */
  handled = $derived(
    this.enhanced
      .filter((r) => r.handled)
      .sort((a, b) => {
        const dateA = new Date(a.handled_at ?? 0);
        const dateB = new Date(b.handled_at ?? 0);
        return dateB.getTime() - dateA.getTime();
      })
  );

  /**
   * Rewards that are available to be claimed
   * @example
   * ```ts
   * dashboard.rewards.available // Get available rewards
   * ```
   */
  available = $derived(
    this.enhanced
      .filter((r) => !r.claimed && !r.queued && !r.handled)
      .sort((a, b) => (b.type ?? 0) - (a.type ?? 0))
  );

  /**
   * Overall reward statistics
   * @example
   * ```ts
   * dashboard.rewards.stats // Get reward statistics
   * ```
   */
  stats = $derived({
    total: this.enhanced.length,
    claimed: this.claimed.length,
    queued: this.queued.length,
    handled: this.handled.length,
    available: this.available.length,
    claimRate: this.enhanced.length > 0
      ? (this.claimed.length / this.enhanced.length) * 100
      : 0,
  });

  constructor(private readonly userId: string) {
    console.log("[reward] Initializing RewardsStore for user:", userId);
    this.loadUserRewards(userId);
    this.loadAvailableRewards();
    this.startPolling(userId);
  }

  /**
   * Loads user rewards data
   * @param userId - The ID of the user to load rewards for
   * @private
   */
  private loadUserRewards(userId: string) {
    Effect.runPromise(
      pipe(
        getUserRewards(userId),
        Effect.tap((result) => {
          console.log("[reward] User rewards loaded:", result);
          this.earned = result;
          return Effect.void;
        })
      )
    );
  }

  /**
   * Loads all available rewards
   * @private
   */
  private loadAvailableRewards() {
    Effect.runPromise(
      pipe(
        getAvailableRewards(),
        Effect.tap((result) => {
          console.log("[reward] Available rewards loaded:", result);
          this.availableRewards = result;
          return Effect.void;
        })
      )
    );
  }

  /**
   * Starts polling for reward updates
   * @param userId - The user ID to poll rewards for
   * @private
   */
  private startPolling(userId: string) {
    this.stopPolling(); // Make sure to stop any existing poll

    // Start polling fiber
    const self = this;
    this.pollFiber = Effect.runFork(
      Effect.forever(
        pipe(
          getUserRewards(userId),
          Effect.tap((result) => {
            self.earned = result;
            return Effect.void;
          }),
          Effect.delay(Duration.millis(POLL_INTERVAL))
        )
      )
    );
  }

  /**
   * Stops polling for reward updates
   * @private
   */
  private stopPolling() {
    if (this.pollFiber) {
      Effect.runPromise(Fiber.interrupt(this.pollFiber));
      this.pollFiber = null;
    }
  }

  /**
   * Refreshes all reward data for the current user
   */
  refresh() {
    this.loadUserRewards(this.userId);
    this.loadAvailableRewards();
  }

  /**
   * Cleans up resources when the store is no longer needed
   */
  cleanup() {
    this.stopPolling();
    this.earned = Option.none();
    this.availableRewards = Option.none();
  }

  /**
   * Updates all reward data for the current user
   * @param force - If true, forces an immediate refresh even if polling is active
   */
  async updateRewards(force = false) {
    // If not forcing and we have a polling fiber, let the polling handle the update
    if (!force && this.pollFiber) {
      return;
    }

    // Stop current polling if it exists
    if (this.pollFiber) {
      this.stopPolling();
    }

    // Update both rewards in parallel
    await Promise.all([
      Effect.runPromise(
        pipe(
          getUserRewards(this.userId),
          Effect.tap((result) => {
            this.earned = result;
            return Effect.void;
          })
        )
      ),
      Effect.runPromise(
        pipe(
          getAvailableRewards(),
          Effect.tap((result) => {
            this.availableRewards = result;
            return Effect.void;
          })
        )
      )
    ]);

    // Restart polling if it was active
    if (this.pollFiber) {
      this.startPolling(this.userId);
    }
  }
}
