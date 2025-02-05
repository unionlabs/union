export const ssr = false
export const prerender = true
export const trailingSlash = "ignore"

import { fetchFeatures } from "$lib/queries/features"

export const load = async ({ url }) => {
  let environment = "DEVELOPMENT"
  if (url.host.startsWith("staging")) {
    environment = "STAGING"
  } else if (url.host.startsWith("app")) {
    environment = "PRODUCTION"
  }

  const features = await fetchFeatures(environment)
  return { features, environment }
}
