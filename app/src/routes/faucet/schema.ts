import { string, regex, object, pipe, type InferOutput } from "valibot"

export const unionAddressRegex = /^union[a-z0-9]{39}$/

export const faucetFormSchema = object({
  address: pipe(string(), regex(unionAddressRegex, "Invalid Union address"))
})

export type FaucetSchema = typeof faucetFormSchema
export type FaucetForm = InferOutput<typeof faucetFormSchema>

export type Message = { status: "error" | "success" | "warning"; text: string }
