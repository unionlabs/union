import type { User } from "@supabase/supabase-js"
import { Effect, Option, pipe } from "effect"
import { AuthenticationError } from "./errors"
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

export const requireAuthenticatedUserId = (
  user: unknown,
): Effect.Effect<string, AuthenticationError, never> => {
  return pipe(
    Option.fromNullable((user as { session?: { user?: { id?: string } } })?.session?.user?.id),
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
}

/**
 * Generates a device fingerprint based on various browser and device characteristics
 * @returns An Effect that resolves to a unique device fingerprint string
 */
export const generateDeviceFingerprint = (): Effect.Effect<string, never, never> => {
  const getBasicInfo = Effect.sync(() => [
    `ua:${navigator.userAgent}`,
    `lang:${navigator.language}`,
    `platform:${
      (navigator as unknown as { userAgentData?: { platform?: string } }).userAgentData?.platform
      || (navigator as unknown as { platform?: string }).platform || "unknown"
    }`,
    `cookieEnabled:${navigator.cookieEnabled}`,
    `doNotTrack:${navigator.doNotTrack}`,
    `screenRes:${screen.width}x${screen.height}`,
    `colorDepth:${screen.colorDepth}`,
    `pixelRatio:${window.devicePixelRatio}`,
    `timezone:${Intl.DateTimeFormat().resolvedOptions().timeZone}`,
    `timezoneOffset:${new Date().getTimezoneOffset()}`,
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
    const oscillator = audioContext.createOscillator()
    const analyser = audioContext.createAnalyser()
    const gainNode = audioContext.createGain()

    oscillator.connect(analyser)
    analyser.connect(gainNode)
    gainNode.connect(audioContext.destination)

    oscillator.frequency.value = 1000
    oscillator.start()

    const frequencyData = new Uint8Array(analyser.frequencyBinCount)
    analyser.getByteFrequencyData(frequencyData)

    const result = `audio:${Array.from(frequencyData.slice(0, 10)).join(",")}`

    oscillator.stop()
    audioContext.close()

    return result
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
    Effect.map(([basicInfo, canvas, webgl, audio, fonts]) => [
      ...basicInfo,
      canvas,
      webgl,
      audio,
      fonts,
    ]),
    Effect.flatMap(createHash),
  )
}
