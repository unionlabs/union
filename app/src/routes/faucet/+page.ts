import { superValidate } from "sveltekit-superforms"
import { valibot } from "sveltekit-superforms/adapters"
import { faucetFormSchema, type FaucetForm } from "./schema.ts"

export const load = async () => ({
  form: await superValidate<FaucetForm, Message>(valibot(faucetFormSchema))
})

export type Message = { status: "error" | "success" | "warning"; text: string }
