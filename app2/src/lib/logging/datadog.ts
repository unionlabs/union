import { PUBLIC_DATADOG_CLIENT_TOKEN } from "$env/static/public"
import { ENV } from "$lib/constants"
import { datadogLogs } from "@datadog/browser-logs"
import { Logger } from "effect"

export const init = () => {
  if (!PUBLIC_DATADOG_CLIENT_TOKEN) {
    return
  }

  datadogLogs.init({
    clientToken: PUBLIC_DATADOG_CLIENT_TOKEN,
    site: "datadoghq.eu",
    forwardErrorsToLogs: false,
    service: "app2",
    env: ENV().toLowerCase(),
    sessionSampleRate: 100,
    version: "unknown",
    telemetrySampleRate: 0,
  })
}

const logger = Logger.make(({ logLevel, message }) => {
  globalThis.console.log(`[DD] [${logLevel.label}] ${message}`)
})
