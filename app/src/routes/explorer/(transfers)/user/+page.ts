import type { PageLoad } from "./$types.ts"

export const load = (loadEvent => ({
  address: loadEvent.url.searchParams.get("address")
})) satisfies PageLoad
