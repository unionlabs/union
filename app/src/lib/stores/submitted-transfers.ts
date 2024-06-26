import { persisted } from "svelte-persisted-store"

export const submittedTransfers = persisted("submittedTransfers", {})
