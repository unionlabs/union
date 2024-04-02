import type { APIRoute } from "astro"
import endpoints from "~root/uniond/docs/static/openapi.json" with { type: "json" }

export const GET: APIRoute = () => Response.json(endpoints)
