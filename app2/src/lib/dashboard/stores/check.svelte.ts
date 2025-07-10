import { runPromise } from "$lib/runtime"
import { Effect, Option, pipe } from "effect"
import { generateDeviceFingerprint } from "../helpers"
import { createSnagUserDevice, createSnagUserMetadata } from "../queries/private"
import { getUserIPAddress } from "../queries/public"
import type { Session, User } from "@supabase/supabase-js"
import type { SnagMetadataPayload } from "../queries/types"
import { SupabaseError } from "../errors"
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
        generateDeviceFingerprint()
      ]),
      Effect.flatMap(([ipAddress, deviceFingerprint]) =>
        createSnagUserDevice({
          ipAddress,
          userId: this.userId,
          deviceIdentifier: deviceFingerprint,
        })
      ),
      Effect.tap(() => 
        Effect.sync(() => {
          this.deviceRegistered = true
        })
      ),
      Effect.catchAll((error) => {
        errorStore.showError(
          new SupabaseError({ 
            cause: error, 
            operation: "registerUserDevice",
            message: "Failed to register user device for fraud tracking"
          })
        )
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
    
    runPromise(pipe(
      createSnagUserMetadata({
        userId: this.userId,
        ...metadata,
      }),
      Effect.tap(() => 
        Effect.sync(() => {
          this.metadataCreated = true
        })
      ),
      Effect.catchAll((error) => {
        errorStore.showError(
          new SupabaseError({ 
            cause: error, 
            operation: "registerUserMetadata",
            message: "Failed to register user metadata for fraud tracking"
          })
        )
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
        if (!session.user) return {}
        
        const user = session.user
        const metadata: Partial<SnagMetadataPayload> = {}

        // Basic user info
        if (user.email) metadata.emailAddress = user.email
        if (user.user_metadata?.name) metadata.displayName = user.user_metadata.name
        if (user.user_metadata?.avatar_url || user.user_metadata?.picture) {
          metadata.logoUrl = user.user_metadata.avatar_url || user.user_metadata.picture
        }

        // Extract from connected identities
        if (user.identities) {
          for (const identity of user.identities) {
            switch (identity.provider) {
              case 'twitter':
                if (identity.identity_data?.user_name) metadata.twitterUser = identity.identity_data.user_name
                if (identity.identity_data?.sub) metadata.twitterUserId = identity.identity_data.sub
                if (identity.identity_data?.picture && !metadata.logoUrl) {
                  metadata.logoUrl = identity.identity_data.picture.replace('_normal', '') // Get higher res Twitter avatar
                }
                break
                
              case 'discord':
                if (identity.identity_data?.full_name) metadata.discordUser = identity.identity_data.full_name
                if (identity.identity_data?.sub) metadata.discordUserId = identity.identity_data.sub
                if (identity.identity_data?.avatar_url && !metadata.logoUrl) {
                  metadata.logoUrl = identity.identity_data.avatar_url + "?size=1024" // Get higher res Discord avatar
                }
                if (identity.identity_data?.email && !metadata.emailAddress) {
                  metadata.emailAddress = identity.identity_data.email
                }
                break
                
              case 'github':
                if (identity.identity_data?.user_name && !metadata.displayName) {
                  metadata.displayName = identity.identity_data.user_name
                }
                if (identity.identity_data?.avatar_url && !metadata.logoUrl) {
                  metadata.logoUrl = identity.identity_data.avatar_url
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
          if (user.user_metadata?.email) metadata.emailAddress = user.user_metadata.email
        }

        return metadata
      }
    })
  }

  cleanup() {
    this.deviceRegistered = false
    this.metadataCreated = false
  }
} 