import { Effect, Option, Fiber, Duration, pipe } from "effect";
import { getUserAchievements, getAvailableAchievements } from "../queries/index";
import type { Entity } from "../client";
import { extractErrorDetails } from "@unionlabs/sdk/utils";
import { AchievementError } from "../errors";

export type Achievement = Entity<"achievements"> & {
  category?: {
    title: string
  } | null
  subcategory?: {
    title: string
  } | null
  reward_achievements?: {
    rewards: {
      title: string | null
      cutoff: string | null
    }[]
  }[]
};

export type UserAchievement = Entity<"user_achievements">;

const POLL_INTERVAL = 5 * 60_000;

export class AchievementsStore {
  /** Achievements the current user has earned */
  achieved = $state<Option.Option<Array<UserAchievement>>>(Option.none());
  /** All achievements that can be earned */
  available = $state<Option.Option<Array<Achievement>>>(Option.none());

  /** Polling fiber */
  private pollFiber: Fiber.Fiber<never, Error> | null = null;

  /** Achievements organized by chain */
  achievementByChain = $derived(
    Option.flatMap(this.available, (achievements) => {
      const achievementsMap = new Map(
        achievements.map((achievement) => [achievement.id, achievement])
      );

      // Helper function to find the first achievement in a chain
      const findChainStart = (achievement: Achievement): Achievement => {
        const previous = achievements.find((a) => a.next === achievement.id);
        return previous ? findChainStart(previous) : achievement;
      };

      return Option.some(
        achievements.reduce<Array<Array<Achievement>>>((chains, achievement) => {
          // Skip if achievement is already in any chain
          if (chains.some((chain) => chain.some((a) => a.id === achievement.id))) {
            return chains;
          }

          // Find the actual start of this chain
          const chainStart = findChainStart(achievement);

          // Build chain from the start
          const chain: Array<Achievement> = [];
          let current: Achievement | null = chainStart;

          while (current) {
            chain.push(current);
            current = current.next !== null
              ? achievementsMap.get(current.next) ?? null
              : null;
          }

          chains.push(chain);
          return chains;
        }, [])
      );
    })
  );

  constructor(private readonly userId: string) {
    console.log("[dashboard] - Creating achievements");
    this.loadUserAchievements(userId);
    this.loadAvailableAchievements();
    this.startPolling(userId);
  }

  /**
   * Loads user achievements for a specific user
   * @param userId - The ID of the user to load achievements for
   */
  private loadUserAchievements(userId: string) {
    Effect.runPromise(
      pipe(
        getUserAchievements(userId),
        Effect.tap((result) => {
          this.achieved = result;
          return Effect.void;
        }),
        Effect.catchAll((error) => 
          Effect.fail(new AchievementError({ 
            cause: extractErrorDetails(error), 
            operation: "load" 
          }))
        )
      )
    );
  }

  /**
   * Loads all available achievements
   */
  private loadAvailableAchievements() {
    Effect.runPromise(
      pipe(
        getAvailableAchievements(),
        Effect.tap((result) => {
          this.available = result;
          return Effect.void;
        }),
        Effect.catchAll((error) => 
          Effect.fail(new AchievementError({ 
            cause: extractErrorDetails(error), 
            operation: "loadAvailable" 
          }))
        )
      )
    );
  }

  /**
   * Starts polling for achievement updates
   * @param userId - The user ID to poll achievements for
   * @private
   */
  private startPolling(userId: string) {
    this.stopPolling(); // Make sure to stop any existing poll

    // Start polling fiber
    const self = this;
    this.pollFiber = Effect.runFork(
      Effect.forever(
        pipe(
          getUserAchievements(userId),
          Effect.tap((result) => {
            self.achieved = result;
            return Effect.void;
          }),
          Effect.catchAll((error) => 
            Effect.fail(new AchievementError({ 
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
   * Stops polling for achievement updates
   * @private
   */
  private stopPolling() {
    if (this.pollFiber) {
      Effect.runPromise(Fiber.interrupt(this.pollFiber));
      this.pollFiber = null;
    }
  }

  /**
   * Refreshes all achievements for the current user
   */
  refresh() {
    this.loadUserAchievements(this.userId);
    this.loadAvailableAchievements();
  }

  /**
   * Cleans up resources when the store is no longer needed
   */
  cleanup() {
    this.stopPolling();
    this.achieved = Option.none();
    this.available = Option.none();
  }
}
