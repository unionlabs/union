import type { APIRoute } from "astro"
import endpoints from "#/content/openapi/endpoints.json" with { type: "json" }

export const GET: APIRoute = () => Response.json(endpoints)
