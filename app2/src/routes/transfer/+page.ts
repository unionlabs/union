import { ENV } from "$lib/constants"
import { DISABLED_CHAIN_IDS } from "$lib/constants/disabled-chains"
import type { Load } from "@sveltejs/kit"
import { redirect } from "@sveltejs/kit"
import { Array as A, Predicate } from "effect"
import { pipe } from "effect/Function"

const PROTECTED_PATHS: Predicate.Predicate<URL>[] = [
  Predicate.and(
    (url) => url.pathname.startsWith("/transfer"),
    (url) =>
      pipe(
        A.union(
          url.searchParams.getAll("source"),
          url.searchParams.getAll("destination"),
        ),
        A.intersection(DISABLED_CHAIN_IDS),
        A.isNonEmptyArray,
      ),
  ),
]

export const load: Load = ({ url }) => {
  if (ENV() === "PRODUCTION" && Predicate.some(PROTECTED_PATHS)(url)) {
    throw redirect(307, "/transfer")
  }
}
