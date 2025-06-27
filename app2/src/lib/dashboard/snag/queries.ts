import { Effect, pipe } from "effect"
import { mapSnagError } from "./errors"
import { getSnagClient } from "./client"
import {
  CreateUserDeviceParams,
  CreateUserMetadataParams,
  validateCreateUserDeviceParams,
  validateCreateUserMetadataParams,
} from "./schema"


/**
 * Creates a user device in Snag for fraud tracking
 * @param params - The device parameters including IP address
 * @returns An Effect that resolves to the created user device or fails with an error
 */
export const createUserDevice = (params: CreateUserDeviceParams) =>
  pipe(
    validateCreateUserDeviceParams(params),
    Effect.flatMap((validatedParams) =>
      pipe(
        getSnagClient(),
        Effect.flatMap((client) =>
          Effect.tryPromise({
            try: () => client.users.createDevice({ ipAddress: validatedParams.ipAddress }),
            catch: (error) => mapSnagError(error, "createUserDevice"),
          })
        ),
      )
    ),
    Effect.catchAll((error) => {
      console.error("Failed to create user device:", error)
      return Effect.fail(error)
    }),
  )

/**
 * Creates user metadata in Snag
 * @param params - User metadata parameters to validate and send
 * @returns An Effect that resolves to the created user metadata or fails with an error
 */
export const createUserMetadata = (params: CreateUserMetadataParams) =>
  pipe(
    validateCreateUserMetadataParams(params),
    Effect.flatMap((validatedParams) =>
      pipe(
        getSnagClient(),
        Effect.flatMap((client) =>
          Effect.tryPromise({
            try: () => client.users.metadatas.create(validatedParams),
            catch: (error) => mapSnagError(error, "createUserMetadata"),
          })
        ),
      )
    ),
    Effect.catchAll((error) => {
      console.error("Failed to create user metadata:", error)
      return Effect.fail(error)
    }),
  ) 