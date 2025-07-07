/**
 * @category utils
 * @since 2.0.0
 */
export type Tail<T extends readonly any[]> = T extends readonly [any, ...infer U] ? U : []
