import type { APIRoute } from "astro"
import endpoints from "#/assets/endpoints.json" with { type: "json" }

export const GET: APIRoute = () => Response.json(endpoints)
