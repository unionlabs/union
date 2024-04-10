import { z } from "zod"
import type { APIRoute } from "astro"
import endpoints from "#/content/openapi/endpoints.json" with { type: "json" }

export const getStaticPaths = () => [{ params: { slug: "rpc" } }, { params: { slug: "rest" } }]

const paramSchema = z.union([z.literal("rest"), z.literal("rpc")])

export const GET: APIRoute = async context => {
  const parsedSlug = paramSchema.safeParse(context.params.slug)
  if (!parsedSlug.success) return new Response("Not Found", { status: 404 })
  const { data: slug } = parsedSlug

  const openApiSchemaImport = import.meta.glob(`../../content/openapi/**/*`, {
    import: "default"
  })
  const [, openApiSchemaFn] = Object.entries(openApiSchemaImport).find(([path]) =>
    path.endsWith(`${slug}/openapi.json`)
  ) as [string, () => Promise<Record<string, unknown>>]

  const openApiSchema = await openApiSchemaFn()
  Object.assign(openApiSchema, {
    servers: endpoints[slug].map(endpoint => ({ url: endpoint }))
  })

  return Response.json(openApiSchema)
}
