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
