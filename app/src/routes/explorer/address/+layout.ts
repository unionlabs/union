import type { LayoutLoad } from "./$types.ts"
const addressExplorerPagePath = "/explorer/address"

export const load = (loadEvent => {
  // if (loadEvent.url.pathname.length > addressExplorerPagePath.length) return
  // const addressArray = [
  //   get(sepoliaStore).address?.toLowerCase(),
  //   get(cosmosStore).address?.toLowerCase()
  // ].filter(Boolean)
  // if (addressArray.length > 0) {
  //   throw redirect(302, `/explorer/address/${addressArray.join("-")}`)
  // }
}) satisfies LayoutLoad
