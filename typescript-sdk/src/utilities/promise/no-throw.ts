export const noThrow = async <T>(x: Promise<T>): Promise<T | undefined> => x.catch(() => undefined)

export const noThrowSync = <T>(callback: T): T | undefined => {
  try {
    return callback
  } catch {
    return undefined
  }
}
