import type { APIRoute } from "astro"
import { purgeCache } from "@netlify/functions"

export const POST: APIRoute = async ({ params, request, ...context }) => {
  const body = (await request.json()) as { sys: { id: string } }

  const contentfulWebhookSecret = process.env.CONTENTFUL_WEBHOOK_SECRET
  const headerSecret = request.headers.get("X-Contentful-Webhook-Secret")
  if (contentfulWebhookSecret !== headerSecret) return new Response("Unauthorized", { status: 401 })

  await purgeCache({ tags: [body.sys.id, "blog"] }).catch(error =>
    Response.json(
      {
        error: error instanceof Error ? error.message : error,
        message: `Failed to revalidate entry with id ${body.sys.id}`
      },
      { status: 500 }
    )
  )

  return Response.json({ message: `Revalidated entry with id ${body.sys.id}` }, { status: 200 })
}
