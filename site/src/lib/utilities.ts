export const arraySizeN = (n: number) => Array.from(Array(n).keys())

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

export function downloadObjectAsFile({
  filename,
  data
}: { filename: string; data: Record<string, any> }) {
  const blob = new Blob([JSON.stringify(data)])
  const jsonObjectURL = window.URL.createObjectURL(blob)
  const temporaryAnchor = document.createElement("a")
  temporaryAnchor.setAttribute("download", filename)
  temporaryAnchor.setAttribute("href", jsonObjectURL)
  temporaryAnchor.click()
  URL.revokeObjectURL(jsonObjectURL)
  temporaryAnchor.remove()
}
