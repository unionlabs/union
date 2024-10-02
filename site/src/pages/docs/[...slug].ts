import type { APIRoute } from "astro"

export const GET = (context => {
  const url = new URL(context.url)
  const pathSegments = url.pathname.split("/")

  const redirectUrl = new URL("https://docs.union.build")
  redirectUrl.pathname = pathSegments.join("/").replace("/docs", "")

  return context.redirect(redirectUrl.toString())
}) satisfies APIRoute
