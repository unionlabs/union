import type { APIRoute } from "astro"
import endpoints from "~root/uniond/docs/static/openapi.json" with { type: "json" }

export const ALL: APIRoute = () => Response.json(endpoints)
