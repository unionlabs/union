export type EvmAddress = `0x${string}`
export type UnionAddress = `union${string}`

export type Optional<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>
export type ExtractParameters<T> = T extends new (...args: infer P) => any ? P[0] : never

export type Coin = { denom: string; amount: string }
