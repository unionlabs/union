import type { APIRoute } from "astro"

export const GET: APIRoute = async () => {
  const openApiDoc = await import(
    `${import.meta.env["COMETBLS_STORE_PATH"]}/rpc/openapi/openapi.yaml`
  )

  return Response.json(openApiDoc)
}
