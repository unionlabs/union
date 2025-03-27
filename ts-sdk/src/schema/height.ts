import { Schema } from "effect"

export const Height = Schema.transform(Schema.Union(Schema.Int, Schema.BigInt), Schema.BigInt, {
  strict: true,
  decode: output => `${output}`,
  encode: input => BigInt(input)
}).pipe(Schema.brand("Height"))

export type Height = typeof Height.Type
