import { json } from "@sveltejs/kit"
import { FAUCET_LINKS } from "$lib/constants/faucets.ts"

export const GET = () => json(FAUCET_LINKS)
