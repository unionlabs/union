import { string, regex, object, type Input } from "valibot"

export const unionAddressRegex = /^union[a-z0-9]{39}$/

export const faucetFormSchema = object({
  address: string([regex(unionAddressRegex, "Invalid Union address")])
})

export type FaucetSchema = typeof faucetFormSchema
export type FaucetForm = Input<typeof faucetFormSchema>

export type Message = { status: "error" | "success" | "warning"; text: string }
