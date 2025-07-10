import { runPromise } from "$lib/runtime"
import type { Session, User } from "@supabase/supabase-js"
import { Effect, Option, pipe } from "effect"
import { SupabaseError } from "../errors"
import { generateDeviceFingerprint } from "../helpers"
import { createSnagUserDevice, createSnagUserMetadata } from "../queries/private"
import { getUserIPAddress } from "../queries/public"
import type { SnagMetadataPayload } from "../queries/types"
import { errorStore } from "../stores/errors.svelte"

export class CheckStore {
  /** Device registration status */
  deviceRegistered = $state(false)
  /** Metadata creation status */
  metadataCreated = $state(false)
  /** Session data */
  private session = $state<Option.Option<Session>>(Option.none())

  constructor(private readonly userId: string, session?: Session) {
    this.session = Option.fromNullable(session)
    this.registerUserDevice()
    this.registerUserMetadata()
  }

  /**
   * Registers user device for fraud tracking
   * @private
   */
  private registerUserDevice() {
    if (this.deviceRegistered) {
      return
    }

    runPromise(pipe(
      Effect.all([
        getUserIPAddress().pipe(Effect.catchAll(() => Effect.succeed("0.0.0.0"))),
        generateDeviceFingerprint(),
      ]),
      Effect.flatMap(([ipAddress, deviceFingerprint]) => {
        const payload = {
          ipAddress,
          userId: this.userId,
          deviceIdentifier: deviceFingerprint,
        }

        console.log("📱 Snag Device Payload:", payload)
        console.log("🔍 Device Fingerprint:", deviceFingerprint)

        // For now, just simulate success
        return Effect.succeed({ success: true })
      }),
      Effect.tap(() =>
        Effect.sync(() => {
          this.deviceRegistered = true
        })
      ),
      Effect.catchAll((error) => {
        console.error("❌ Device registration error:", error)
        return Effect.void
      }),
    ))
  }

  /**
   * Creates user metadata automatically on session start
   * @private
   */
  private registerUserMetadata() {
    if (this.metadataCreated) {
      return
    }

    const metadata = this.extractSessionMetadata()
    const payload = {
      userId: this.userId,
      ...metadata,
    }

    console.log("👤 Snag Metadata Payload:", payload)

    runPromise(pipe(
      // For now, just simulate success
      Effect.succeed({ success: true }),
      Effect.tap(() =>
        Effect.sync(() => {
          this.metadataCreated = true
        })
      ),
      Effect.catchAll((error) => {
        console.error("❌ Metadata registration error:", error)
        return Effect.void
      }),
    ))
  }

  /**
   * Extracts available metadata from the session
   * @private
   */
  private extractSessionMetadata(): Partial<SnagMetadataPayload> {
    return Option.match(this.session, {
      onNone: () => ({}),
      onSome: (session) => {
        if (!session.user) {
          return {}
        }

        const user = session.user
        const metadata: Partial<SnagMetadataPayload> = {}

        // Basic user info
        if (user.email) {
          metadata.emailAddress = user.email
        }
        if (user.user_metadata?.name) {
          metadata.displayName = user.user_metadata.name
        }

        // Extract from connected identities
        if (user.identities) {
          for (const identity of user.identities) {
            switch (identity.provider) {
              case "twitter":
                if (identity.identity_data?.user_name) {
                  metadata.twitterUser = identity.identity_data.user_name
                }
                if (identity.identity_data?.sub) {
                  metadata.twitterUserId = identity.identity_data.sub
                }
                break

              case "discord":
                if (identity.identity_data?.full_name) {
                  metadata.discordUser = identity.identity_data.full_name
                }
                if (identity.identity_data?.sub) {
                  metadata.discordUserId = identity.identity_data.sub
                }
                if (identity.identity_data?.email && !metadata.emailAddress) {
                  metadata.emailAddress = identity.identity_data.email
                }
                break

              case "github":
                if (identity.identity_data?.user_name && !metadata.displayName) {
                  metadata.displayName = identity.identity_data.user_name
                }
                if (identity.identity_data?.email && !metadata.emailAddress) {
                  metadata.emailAddress = identity.identity_data.email
                }
                break
            }
          }
        }

        // Fallback email extraction with priority
        if (!metadata.emailAddress) {
          if (user.user_metadata?.email) {
            metadata.emailAddress = user.user_metadata.email
          }
        }

        return metadata
      },
    })
  }

  cleanup() {
    this.deviceRegistered = false
    this.metadataCreated = false
  }
}
