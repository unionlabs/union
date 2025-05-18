import { Array as A, Record as R } from "effect"

export const flattenObject = <T extends Record<string, any>>(
  self: T,
  prefix = "",
): Record<string, any> =>
  R.reduce(self, {} as Record<string, any>, (acc, value, key) => {
    const fullKey = prefix ? `${prefix}.${key}` : key
    const combine = (s1: string, s2: string) => s1 + s2
    const shouldRecurse = value !== null && typeof value === "object" && !A.isArray(value)
    if (shouldRecurse) {
      return R.union(acc, flattenObject(value, fullKey), combine)
    } else {
      return R.set(acc, fullKey, value)
    }
  })
