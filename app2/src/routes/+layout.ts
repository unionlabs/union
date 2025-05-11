import type { Edition } from "$lib/themes"
import { Match, String as Str } from "effect"
import type { LayoutLoad } from "./$types"

export const ssr = false
export const prerender = true
export const trailingSlash = "ignore"

export const load: LayoutLoad = ({ url }) => {
  const hostname = url.hostname

  const edition: Edition = Match.value(hostname).pipe(
    Match.whenOr(
      Str.startsWith("btc."),
      Str.startsWith("staging.btc."),
      () => "btc" as const,
    ),
    Match.whenOr(
      Str.startsWith("app."),
      Str.startsWith("staging.app."),
      () => "app" as const,
    ),
    Match.orElse(() => "app" as const),
  )

  return { edition }
}
