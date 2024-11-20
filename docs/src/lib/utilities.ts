// split array into n parts
export const splitArray = <T>({ array, n }: { array: Array<T>; n: number }): Array<Array<T>> =>
  array.reduce(
    (accumulator, current, index) => {
      const chunkIndex = Math.floor(index / n)
      if (!accumulator[chunkIndex]) accumulator[chunkIndex] = []
      accumulator[chunkIndex].push(current)
      return accumulator
    },
    [] as Array<Array<T>>
  )

// remove duplicates from an array of objects by a key
export const removeArrayDuplicates = <T>(array: Array<T>, key: keyof T): Array<T> =>
  array.reduce(
    (accumulator, current) => {
      if (!accumulator.find(item => item[key] === current[key])) {
        accumulator.push(current)
      }
      return accumulator
    },
    [] as Array<T>
  )

export function stringIsJSON(str: string) {
  try {
    let _json = JSON.parse(str)
    return typeof _json === "object" && _json !== null
  } catch {
    return false
  }
}

export function fragmentFromString(stringifiedHTML: string) {
  return document.createRange().createContextualFragment(stringifiedHTML.trim())
}

export const toISODate = (date?: string | Date) => (date ? new Date(date).toISOString() : "")

export const saneDateTime = (date?: string | Date) =>
  new Date(date ?? "").toLocaleDateString("fr-CA", {
    year: "numeric",
    month: "numeric",
    day: "numeric"
  })

export const arraySizeN = (n: number) => Array.from(new Array(n).keys())

export const sleep = (ms: number): Promise<void> => new Promise(resolve => setTimeout(resolve, ms))

export const generateRandomNumber = (min: number, max: number) => Math.random() * (max - min) + min

export const roundNumber = (_number: number, decimalPlaces: number) =>
  Math.round(_number * 10 ** decimalPlaces) / 10 ** decimalPlaces

export function raise(error: unknown): never {
  throw typeof error === "string" ? new Error(error) : error
}

export function isKeyOf<T extends object>(obj: T, key: keyof any): key is keyof T {
  if (!key) return false
  return key in obj
}
