export const ssr = false
export const prerender = true
export const trailingSlash = "ignore"

import { fetchFeatures } from "$lib/queries/features"

import { PUBLIC_ENVIRONMENT } from "$env/static/public"

export const load = async () => {
  const environment = PUBLIC_ENVIRONMENT.toUpperCase()
  const features = await fetchFeatures(environment)

  const modifiedFeatures = features.map(chain => {
    if (chain.chain_id === "bbn-test-5" || chain.chain_id === "union-testnet-9") {
      return {
        ...chain,
        features: [
          {
            channel_list: true,
            connection_list: true,
            environment: environment,
            index_status: true,
            packet_list: true,
            transfer_list: true,
            transfer_submission: true
          }
        ]
      }
    }
    return chain
  })

  console.log("Modified features:", modifiedFeatures)

  return {
    features: modifiedFeatures
  }
}

// export const load = async () => {
//   const environment = PUBLIC_ENVIRONMENT.toUpperCase()
//   const features = await fetchFeatures(environment)
//
//   console.log(features)
//
//   return {
//     features
//   }
// }
