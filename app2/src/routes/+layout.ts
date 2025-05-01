import type { Edition } from "$lib/themes"

export const ssr = false
export const prerender = true
export const trailingSlash = "ignore"

export const load = ({ url }) => {
  const hostname = url.hostname
  let edition: Edition = "app"

  if (hostname.startsWith("btc.") || hostname.startsWith("staging.btc.")) {
    edition = "btc"
  } else if (hostname.startsWith("app.") || hostname.startsWith("staging.app.")) {
    edition = "app"
  }

  return {
    edition
  }
}
