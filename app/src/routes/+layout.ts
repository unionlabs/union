export const ssr = false
export const prerender = true
export const trailingSlash = "ignore"

import { fetchFeatures } from "$lib/queries/features"

export const load = async () => {
  const environment = import.meta.env.ENVIRONMENT.toUpperCase()
  const features = await fetchFeatures(environment)

  return {
    features
  }
}
