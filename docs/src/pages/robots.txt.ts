import type { APIRoute } from "astro"

/**
 * @see https://docs.astro.build/en/guides/integrations-guide/sitemap/#usage
 */

const robotsTxt = `
User-agent: *
Allow: /

Sitemap: ${new URL("sitemap-index.xml", import.meta.env.SITE).href}
`.trim()

export const GET: APIRoute = () => {
  return new Response(robotsTxt, {
    headers: {
      "Content-Type": "text/plain; charset=utf-8"
    }
  })
}
