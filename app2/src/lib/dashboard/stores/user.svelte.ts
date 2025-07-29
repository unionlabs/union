import { browser } from "$app/environment"
import { goto } from "$app/navigation"
import { runFork, runPromise } from "$lib/runtime"
import type { AuthChangeEvent, Session, User } from "@supabase/supabase-js"
import { extractErrorDetails } from "@unionlabs/sdk/utils"
import { Duration, Effect, Fiber, Option, pipe } from "effect"
import { SupabaseClient } from "../client"
import { AuthenticationError, SupabaseError } from "../errors"
import { hasProviderLinked } from "../helpers"
import { invokeTick } from "../queries/private"
import { errorStore } from "../stores/errors.svelte"
import { AchievementsStore } from "./achievements.svelte"
import { CheckStore } from "./check.svelte"
import { ExperienceStore } from "./experience.svelte"
import { LeaderboardStore } from "./leaderboard.svelte"
import { MissionsStore } from "./missions.svelte"
import { RewardsStore } from "./rewards.svelte"
import { WalletStore } from "./wallets.svelte"

export type AuthProvider = "github" | "twitter" | "discord"
export type Providers = AuthProvider | "default" | "name"

const PROTECTED_PATHS = ["/dashboard"]

export class Dashboard {
  session = $state<Option.Option<Session>>(Option.none())
  user = $derived(Option.flatMap(this.session, (s) => Option.fromNullable(s.user)))
  userId = $derived(Option.flatMap(this.user, (u) => Option.fromNullable(u.id)))

  /** Achievements store instance */
  achievements = $state<Option.Option<AchievementsStore>>(Option.none())

  /** Experience store instance */
  experience = $state<Option.Option<ExperienceStore>>(Option.none())

  /** Missions store instance */
  missions = $state<Option.Option<MissionsStore>>(Option.none())

  /** Rewards store instance */
  rewards = $state<Option.Option<RewardsStore>>(Option.none())

  /** Leaderboard store instance */
  leaderboard = $state<Option.Option<LeaderboardStore>>(Option.none())

  /** Wallet store instance */
  wallets = $state<Option.Option<WalletStore>>(Option.none())

  /** Check store instance */
  check = $state<Option.Option<CheckStore>>(Option.none())

  /**
   * Usernames from all connected providers
   * @derived Mapping of provider to username
   */
  usernames = $derived({
    twitter: Option.flatMap(
      this.findIdentity("twitter"),
      i => Option.fromNullable(i.identity_data?.user_name),
    ),
    github: Option.flatMap(
      this.findIdentity("github"),
      i => Option.fromNullable(i.identity_data?.user_name),
    ),
    discord: Option.flatMap(
      this.findIdentity("discord"),
      i => Option.fromNullable(i.identity_data?.full_name),
    ),
    name: Option.flatMap(this.session, s => Option.fromNullable(s.user?.user_metadata?.name)),
  })

  /**
   * Avatar URLs from all connected providers
   * @derived Mapping of provider to avatar URL
   */
  userAvatars = $derived({
    twitter: Option.flatMap(
      this.findIdentity("twitter"),
      i => Option.fromNullable(i.identity_data?.picture?.replace("_normal", "")),
    ),
    github: Option.flatMap(
      this.findIdentity("github"),
      i => Option.fromNullable(i.identity_data?.avatar_url),
    ),
    discord: Option.flatMap(
      this.findIdentity("discord"),
      i => Option.fromNullable(i.identity_data?.avatar_url + "?size=1024"),
    ),
    default: Option.flatMap(this.session, s =>
      Option.fromNullable(
        s.user?.user_metadata?.avatar_url
          || s.user?.user_metadata?.picture,
      )),
  })

  /**
   * First connected social provider
   * @derived First provider from priority list that user has connected
   */
  firstConnectedProvider = $derived(
    Option.flatMap(this.session, s =>
      Option.fromNullable(
        ["twitter", "discord", "github"].find(p => s.user?.identities?.some(i => i.provider === p)),
      )),
  )

  /**
   * Combined user identity information
   * @derived Best available username and avatar from connected providers
   */
  identity = $derived({
    username: Option.flatMap(
      this.firstConnectedProvider,
      (provider) => this.usernames[provider as keyof typeof this.usernames],
    ).pipe(
      Option.orElse(() => this.usernames.name),
    ),
    avatar: Option.flatMap(
      this.firstConnectedProvider,
      (provider) => this.userAvatars[provider as keyof typeof this.userAvatars],
    ).pipe(
      Option.orElse(() => this.userAvatars.default),
    ),
  })

  /**
   * Connected social providers
   * @derived Mapping of provider to connection status
   */
  connections = $derived({
    github: Option.flatMap(this.session, s => Option.some(hasProviderLinked(s.user, "github"))),
    twitter: Option.flatMap(this.session, s => Option.some(hasProviderLinked(s.user, "twitter"))),
    discord: Option.flatMap(this.session, s => Option.some(hasProviderLinked(s.user, "discord"))),
  })

  /** Tick polling fiber */
  private tickFiber: Fiber.Fiber<never, Error> | null = null

  /**
   * Finds an identity for a specific authentication provider
   * @param provider - The authentication provider to find identity for
   * @returns An Option containing the identity if found, None otherwise
   * @private
   */
  private findIdentity(provider: AuthProvider) {
    return Option.flatMap(this.session, s =>
      Option.fromNullable(
        s.user.identities?.find(i => i.provider === provider),
      ))
  }

  /**
   * Initializes the Dashboard instance and sets up authentication listeners
   */
  constructor() {
    if (!browser) {
      return
    }
    this.listenToAuth()
  }

  /**
   * Sets up authentication state change listeners using Supabase client
   * @private
   */
  private listenToAuth() {
    runPromise(pipe(
      SupabaseClient,
      Effect.andThen((client) =>
        pipe(
          Effect.tryPromise({
            try: async () => {
              const { data, error } = await client.auth.getSession()
              if (error) {
                throw error
              }
              return { data }
            },
            catch: (cause) =>
              new SupabaseError({
                operation: "getSession",
                cause,
              }),
          }),
          Effect.tap(({ data }) =>
            Effect.sync(() => {
              this.session = Option.fromNullable(data.session)
              this.handleAuthChange(data.session)
            })
          ),
          Effect.andThen(() =>
            Effect.sync(() => {
              client.auth.onAuthStateChange((_event: AuthChangeEvent, session: Session | null) => {
                this.session = Option.fromNullable(session)
                this.handleAuthChange(session)
              })
            })
          ),
          Effect.catchAll((error) => {
            errorStore.showError(new AuthenticationError({ cause: error, operation: "auth" }))
            return Effect.void
          }),
        )
      ),
    ))
  }

  /**
   * Starts polling for tick updates
   * @private
   */
  private startTickPolling() {
    this.stopTickPolling()

    this.tickFiber = runFork(
      pipe(
        this.userId,
        Effect.flatMap((id) => Effect.map(invokeTick(id), () => void 0)),
        Effect.catchAll(() => Effect.void),
        Effect.delay(Duration.minutes(4)),
        Effect.forever,
      ),
    )
  }

  /**
   * Stops polling for tick updates
   * @private
   */
  private stopTickPolling() {
    if (this.tickFiber) {
      runPromise(Fiber.interrupt(this.tickFiber))
      this.tickFiber = null
    }
  }

  /**
   * Handles authentication state changes
   * @param session - The new session or null if logged out
   * @private
   */
  private handleAuthChange(session: Session | null) {
    if (session?.user?.id) {
      // Create stores if they don't exist
      if (Option.isNone(this.achievements)) {
        this.achievements = Option.some(new AchievementsStore(session.user.id))
      }
      if (Option.isNone(this.experience)) {
        this.experience = Option.some(new ExperienceStore(session.user.id))
      }
      if (Option.isNone(this.missions)) {
        this.missions = Option.some(new MissionsStore(session.user.id))
      }
      if (Option.isNone(this.rewards)) {
        this.rewards = Option.some(new RewardsStore(session.user.id))
      }
      if (Option.isNone(this.leaderboard)) {
        this.leaderboard = Option.some(new LeaderboardStore())
      }
      if (Option.isNone(this.wallets)) {
        this.wallets = Option.some(new WalletStore(session.user.id))
      }
      if (Option.isNone(this.check)) {
        this.check = Option.some(new CheckStore(session.user.id))
      }
      // Start tick polling when user is authenticated
      this.startTickPolling()
    } else {
      // Clean up stores
      Option.match(this.achievements, {
        onNone: () => {},
        onSome: (store) => store.cleanup(),
      })
      this.achievements = Option.none()

      Option.match(this.experience, {
        onNone: () => {},
        onSome: (store) => store.cleanup(),
      })
      this.experience = Option.none()

      Option.match(this.missions, {
        onNone: () => {},
        onSome: (store) => store.cleanup(),
      })
      this.missions = Option.none()

      Option.match(this.rewards, {
        onNone: () => {},
        onSome: (store) => store.cleanup(),
      })
      this.rewards = Option.none()

      Option.match(this.leaderboard, {
        onNone: () => {},
        onSome: (store) => store.cleanup(),
      })
      this.leaderboard = Option.none()

      Option.match(this.wallets, {
        onNone: () => {},
        onSome: (store) => store.cleanup(),
      })
      this.wallets = Option.none()

      Option.match(this.check, {
        onNone: () => {},
        onSome: (store) => store.cleanup(),
      })
      this.check = Option.none()

      // Stop tick polling when user is logged out
      this.stopTickPolling()
    }
  }

  /**
   * Requires a valid user ID for operations that need authentication
   * @returns An Effect that succeeds with the user ID or fails with an error if not authenticated
   */
  requireUserId() {
    return pipe(
      this.userId,
      Option.match({
        onNone: () =>
          Effect.fail(
            new SupabaseError({
              operation: "requireUserId",
              cause: "User not authenticated",
            }),
          ),
        onSome: Effect.succeed,
      }),
    )
  }

  /**
   * Initiates the login process with a specific authentication provider
   * @param provider - The authentication provider to use for login
   * @returns An Effect that handles the OAuth login flow
   */
  login(provider: AuthProvider) {
    return pipe(
      SupabaseClient,
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () =>
            client.auth.signInWithOAuth({
              provider,
              options: {
                redirectTo: `${window.location.origin}/dashboard`,
                skipBrowserRedirect: true,
              },
            }),
          catch: (cause) =>
            new SupabaseError({
              operation: "signInWithOAuth",
              cause,
            }),
        })
      ),
      Effect.flatMap(({ data, error: cause }) =>
        cause
          ? Effect.fail(
            new SupabaseError({
              operation: "signInWithOAuth",
              cause,
            }),
          )
          : Effect.succeed(data)
      ),
      Effect.tap(({ url }) =>
        Effect.sync(() => {
          if (url) {
            window.location.href = url
          }
        })
      ),
      Effect.catchAll((error) => {
        errorStore.showError(
          new AuthenticationError({ cause: error, operation: "signInWithOAuth" }),
        )
        return Effect.succeed(null)
      }),
    )
  }

  /**
   * Logs out the current user and redirects from protected paths
   * @returns An Effect that handles the logout process
   */
  logout() {
    return pipe(
      SupabaseClient,
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () => client.auth.signOut(),
          catch: (error) =>
            new SupabaseError({
              operation: "signOut",
              cause: error,
            }),
        })
      ),
      Effect.flatMap(({ error: cause }) =>
        cause
          ? Effect.fail(
            new SupabaseError({
              operation: "signOut",
              cause,
            }),
          )
          : Effect.void
      ),
      Effect.tap(() =>
        Effect.sync(() => {
          if (PROTECTED_PATHS.some(path => window.location.pathname.startsWith(path))) {
            goto("/")
          }
        })
      ),
      Effect.catchAll((error) => {
        errorStore.showError(new AuthenticationError({ cause: error, operation: "signOut" }))
        return Effect.void
      }),
    )
  }

  /**
   * Links a new authentication provider to the current user account
   * @param provider - The authentication provider to link
   * @returns An Effect that handles the identity linking process
   */
  linkIdentity(provider: AuthProvider) {
    return pipe(
      SupabaseClient,
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () =>
            client.auth.linkIdentity({
              provider,
              options: {
                redirectTo: `${window.location.origin}/auth?linking=true&returnTo=${
                  encodeURIComponent(window.location.pathname)
                }`,
                skipBrowserRedirect: true,
              },
            }),
          catch: (cause) =>
            new SupabaseError({
              operation: "linkIdentity",
              cause,
            }),
        })
      ),
      Effect.flatMap(({ data, error: cause }) =>
        cause
          ? Effect.fail(
            new SupabaseError({
              operation: "linkIdentity",
              cause,
            }),
          )
          : Effect.succeed(data)
      ),
      Effect.tap(({ url }) =>
        Effect.sync(() => {
          if (url) {
            window.location.href = url
          }
        })
      ),
      Effect.catchAll((error) => {
        errorStore.showError(new AuthenticationError({ cause: error, operation: "linkIdentity" }))
        return Effect.succeed(null)
      }),
    )
  }

  /**
   * Refreshes the current user session
   * @returns An Effect that handles the session refresh
   * @private
   */
  private refreshSession() {
    return Effect.gen(function*(this: Dashboard) {
      const client = yield* SupabaseClient

      const { data: { session }, error } = yield* Effect.tryPromise({
        try: () => client.auth.refreshSession(),
        catch: (cause) =>
          new SupabaseError({
            operation: "refreshSession",
            cause,
          }),
      })

      if (error) {
        errorStore.showError(new AuthenticationError({ cause: error, operation: "refreshSession" }))
        return yield* Effect.succeed(null)
      }

      yield* Effect.sync(() => {
        this.session = Option.none()
        setTimeout(() => {
          this.session = Option.fromNullable(session)
          this.handleAuthChange(session)
        }, 10)
      })
    }.bind(this))
  }

  /**
   * Removes a linked authentication provider from the current user account
   * @param provider - The authentication provider to unlink
   * @returns An Effect that handles the identity unlinking process
   */
  unlinkIdentity(provider: AuthProvider) {
    return Effect.gen(function*(this: Dashboard) {
      const client = yield* SupabaseClient

      const user = Option.getOrUndefined(this.user) as User | undefined
      if (!user) {
        errorStore.showError(
          new AuthenticationError({
            cause: "No user session",
            operation: "unlinkIdentity",
          }),
        )
        return yield* Effect.succeed(null)
      }

      if (!hasProviderLinked(user, provider)) {
        errorStore.showError(
          new AuthenticationError({
            cause: `Provider ${provider} not linked`,
            operation: "unlinkIdentity",
          }),
        )
        return yield* Effect.succeed(null)
      }

      const confirmed = yield* Effect.sync(() =>
        confirm(`Are you sure you want to unlink your ${provider} account?`)
      )

      if (!confirmed) {
        return yield* Effect.succeed(undefined)
      }

      const identity = user.identities?.find((i) => i.provider === provider)

      if (!identity) {
        errorStore.showError(
          new AuthenticationError({
            cause: `Provider ${provider} not found`,
            operation: "unlinkIdentity",
          }),
        )
        return yield* Effect.succeed(null)
      }

      const { error } = yield* Effect.tryPromise({
        try: () => client.auth.unlinkIdentity(identity),
        catch: (error) =>
          new SupabaseError({
            operation: "unlinkIdentity",
            cause: error,
          }),
      })

      if (error) {
        errorStore.showError(new AuthenticationError({ cause: error, operation: "unlinkIdentity" }))
        return yield* Effect.succeed(null)
      }

      yield* this.refreshSession()
      goto(`/auth?returnTo=${encodeURIComponent(window.location.pathname)}`)
    }.bind(this))
  }

  /**
   * Deletes the current user's account
   * @returns An Effect that handles the account deletion process
   */
  deleteAccount() {
    return pipe(
      SupabaseClient,
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () =>
            client.functions.invoke("delete-account", {
              method: "POST",
            }),
          catch: (error) => {
            const details = extractErrorDetails(error as Error)
            return new SupabaseError({
              operation: "deleteAccount",
              cause: details,
              message: "Unexpected error while deleting account",
            })
          },
        })
      ),
      Effect.flatMap(({ error }) =>
        error
          ? Effect.fail(
            new SupabaseError({
              operation: "deleteAccount",
              cause: extractErrorDetails(error as Error),
              message: "Edge function error: " + error.message,
            }),
          )
          : Effect.void
      ),
      Effect.tap(() =>
        Effect.sync(() => {
          if (PROTECTED_PATHS.some(path => window.location.pathname.startsWith(path))) {
            goto("/")
          }
        })
      ),
      Effect.catchAll((error) => {
        errorStore.showError(
          new AuthenticationError({
            cause: error,
            operation: "deleteAccount",
            message: error.message || "Failed to delete account",
          }),
        )
        return Effect.void
      }),
    )
  }
}

export const dashboard = new Dashboard()
