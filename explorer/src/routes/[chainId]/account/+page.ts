import type { PageLoad } from "./$types"

export const prerender = false

export const load: PageLoad = async ({ parent }) => {
  const { chain } = await parent()
  return { chain }
}
