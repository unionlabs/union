import { runPromise } from "$lib/runtime"
import { Effect, pipe } from "effect"
import { generateDeviceFingerprint, getUserIPAddress } from "../helpers"
import { createSnagUserDevice } from "../queries/private"

export class CheckStore {
  deviceRegistered = $state(false)
  userId = $state("")

  constructor(userId: string) {
    this.userId = userId
    this.registerUserDevice()
  }

  registerUserDevice() {
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

        console.log("📱 Device Payload:", payload)
        console.log("🔍 Device Fingerprint:", deviceFingerprint)

        return createSnagUserDevice(payload)
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

  cleanup() {
    this.deviceRegistered = false
  }
}
