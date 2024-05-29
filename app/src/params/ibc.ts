import type { ParamMatcher } from "@sveltejs/kit"

export const match = (param => {
  return ["connections", "channels", "packets", "clients"].includes(param)
}) satisfies ParamMatcher
