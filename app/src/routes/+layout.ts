export const ssr = false
export const prerender = true
export const trailingSlash = "ignore"

import { fetchFeatures } from "$lib/queries/features"

export const load = async () => {
  const environment = import.meta.env.VITE_ENVIRONMENT.toUpperCase()
  const features = await fetchFeatures(environment)

  return {
    features
  }
}

// export const load = async () => {
//   const environment = import.meta.env.VITE_ENVIRONMENT.toUpperCase();
//   const features = await fetchFeatures(environment);
//
//   // Map through the features and set all flags to false for development environment on Sepolia chain
//   const mappedFeatures = features.map(chain => {
//     if (chain.chain_id === "11155111") {
//       return {
//         ...chain,
//         features: chain.features.map(feature => {
//           if (feature.environment === "DEVELOPMENT") {
//             return {
//               ...feature,
//               channel_list: false,
//               connection_list: false,
//               index_status: false,
//               packet_list: false,
//               transfer_list: false,
//               transfer_submission: false
//             };
//           }
//           return feature;
//         })
//       };
//     }
//     return chain;
//   });
//
//   return {
//     features: mappedFeatures
//   };
// };
