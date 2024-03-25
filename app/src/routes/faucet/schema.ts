import * as v from "valibot"

export const unionAddressRegex = /^union[a-z0-9]{39}$/

export const faucetFormSchema = v.object({
  address: v.string([v.regex(unionAddressRegex, "Invalid Union address")])
})

export type FaucetSchema = typeof faucetFormSchema
export type FaucetForm = v.Input<typeof faucetFormSchema>

export type Message = { status: "error" | "success" | "warning"; text: string }
