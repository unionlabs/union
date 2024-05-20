import type { APIRoute } from "astro"
import contracts from "~root/versions/contracts.json" with { type: "json" }

export const GET: APIRoute = () => Response.json(contracts)

