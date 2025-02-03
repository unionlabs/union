export const ssr = false
export const prerender = true
export const trailingSlash = "ignore"

import { fetchFeatures } from "$lib/queries/features"

export const load = async ({ url }) => {
  let environment = "development"
  if (url.host.startsWith("staging")) {
    environment = "staging"
  } else if (url.host.startsWith("app")) {
    environment = "production"
  }

  const features = await fetchFeatures(environment.toUpperCase())
  return { features }
}
