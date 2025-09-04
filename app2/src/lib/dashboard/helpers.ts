import type { User } from "@supabase/supabase-js"
import { extractErrorDetails } from "@unionlabs/sdk/Utils"
import { Effect, Option, pipe } from "effect"
import { AuthenticationError, SupabaseError } from "./errors"
import type { AuthProvider } from "./stores/user.svelte"

export const hasProviderLinked = (user: User, provider: AuthProvider) =>
  user.identities?.some(identity => identity.provider === provider) ?? false

export const getProviderId = (user: User, provider: AuthProvider) =>
  pipe(
    user.identities,
    Option.fromNullable,
    Option.flatMap(identities =>
      Option.fromNullable(
        identities.find(id => id.provider.toLowerCase() === provider.toLowerCase()),
      )
    ),
    Option.map(identity => identity.id),
  )

export const isProviderConnected = (user: User, provider: AuthProvider) =>
  pipe(
    user.identities,
    Option.fromNullable,
    Option.map(identities =>
      identities.some(id => id.provider.toLowerCase() === provider.toLowerCase())
    ),
    Option.getOrElse(() => false),
  )

export const requireAuthenticatedUserId = (user: { session?: { user?: { id?: string } } }) =>
  pipe(
    Option.fromNullable(user?.session?.user?.id),
    Option.match({
      onNone: () =>
        Effect.fail(
          new AuthenticationError({
            cause: "User is not authenticated",
            operation: "requireAuth",
          }),
        ),
      onSome: (userId) => Effect.succeed(userId),
    }),
  )

export const generateDeviceFingerprint = (): Effect.Effect<string, never, never> => {
  const getBasicInfo = Effect.sync(() => [
    `ua:${navigator.userAgent}`,
    `lang:${navigator.language}`,
    `platform:${
      (navigator as unknown as { userAgentData?: { platform?: string } }).userAgentData?.platform
      || (navigator as unknown as { platform?: string }).platform || "unknown"
    }`,
    `cookieEnabled:${navigator.cookieEnabled}`,
    `colorDepth:${screen.colorDepth}`,
    `timezone:${Intl.DateTimeFormat().resolvedOptions().timeZone}`,
    `cores:${navigator.hardwareConcurrency || "unknown"}`,
    `memory:${(navigator as unknown as Record<string, unknown>).deviceMemory || "unknown"}`,
    `touchSupport:${navigator.maxTouchPoints || 0}`,
  ])

  const getCanvasFingerprint = Effect.sync(() => {
    const canvas = document.createElement("canvas")
    const ctx = canvas.getContext("2d")
    if (ctx) {
      ctx.textBaseline = "top"
      ctx.font = "14px Arial"
      ctx.fillText("Device fingerprint test 🎯", 2, 2)
      return `canvas:${canvas.toDataURL().slice(-50)}`
    }
    return "canvas:unavailable"
  }).pipe(
    Effect.catchAll(() => Effect.succeed("canvas:error")),
  )

  const getWebGLFingerprint = Effect.sync(() => {
    const gl = document.createElement("canvas").getContext("webgl")
    if (gl) {
      const debugInfo = gl.getExtension("WEBGL_debug_renderer_info")
      if (debugInfo) {
        return `webgl:${gl.getParameter(debugInfo.UNMASKED_RENDERER_WEBGL)}`
      }
    }
    return "webgl:unavailable"
  }).pipe(
    Effect.catchAll(() => Effect.succeed("webgl:error")),
  )

  const getAudioFingerprint = Effect.sync(() => {
    const audioContext = new (window.AudioContext
      || (window as unknown as Record<string, unknown>)
        .webkitAudioContext as typeof AudioContext)()

    const sampleRate = audioContext.sampleRate
    const baseLatency = audioContext.baseLatency || 0
    const outputLatency = audioContext.outputLatency || 0
    const state = audioContext.state

    audioContext.close()

    return `audio:${sampleRate}-${baseLatency}-${outputLatency}-${state}`
  }).pipe(
    Effect.catchAll(() => Effect.succeed("audio:error")),
  )

  const getFontFingerprint = Effect.sync(() => {
    const testFonts = ["Arial", "Times New Roman", "Courier New", "Georgia", "Verdana"]
    const availableFonts = testFonts.filter(font => {
      const canvas = document.createElement("canvas")
      const ctx = canvas.getContext("2d")
      if (!ctx) {
        return false
      }

      ctx.font = `12px ${font}`
      const width1 = ctx.measureText("mmmmmmmmmmlli").width

      ctx.font = "12px monospace"
      const width2 = ctx.measureText("mmmmmmmmmmlli").width

      return width1 !== width2
    })
    return `fonts:${availableFonts.join(",")}`
  }).pipe(
    Effect.catchAll(() => Effect.succeed("fonts:error")),
  )

  const createHash = (components: string[]) =>
    Effect.sync(() => {
      const fingerprint = components.join("|")
      let hash = 0
      for (let i = 0; i < fingerprint.length; i++) {
        const char = fingerprint.charCodeAt(i)
        hash = ((hash << 5) - hash) + char
        hash = hash & hash // Convert to 32-bit integer
      }
      return Math.abs(hash).toString(16)
    })

  return pipe(
    Effect.all([
      getBasicInfo,
      getCanvasFingerprint,
      getWebGLFingerprint,
      getAudioFingerprint,
      getFontFingerprint,
    ]),
    Effect.map(([basicInfo, canvas, webgl, audio, fonts]) => {
      const components = [
        ...basicInfo,
        canvas,
        webgl,
        audio,
        fonts,
      ]
      console.log("🔧 Fingerprint components:", components)
      return components
    }),
    Effect.flatMap(createHash),
    Effect.tap((fingerprint) =>
      Effect.sync(() => console.log("🔍 Generated fingerprint:", fingerprint))
    ),
  )
}

export const getUserIPAddress = () =>
  Effect.tryPromise({
    try: async () => {
      const response = await fetch("https://api.ipify.org?format=json")
      const data = await response.json()
      return data.ip
    },
    catch: (error) =>
      new SupabaseError({
        operation: "getUserIPAddress",
        cause: extractErrorDetails(error as Error),
      }),
  })
