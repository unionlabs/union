import rss from "@astrojs/rss"
import type { APIContext } from "astro"
import { env } from "#/lib/constants/env.ts"
import { graphqlUrl, rssItemsQuery, type RssItemsResponse } from "#/lib/contentful/queries.ts"

export async function GET(context: APIContext) {
  const site = context.site
  if (!site) throw new Error("Missing site metadata")

  const response = await fetch(graphqlUrl, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${env.CONTENTFUL_DELIVERY_TOKEN}`
    },
    body: JSON.stringify({ query: rssItemsQuery })
  })
  const json = (await response.json()) as RssItemsResponse
  if (!("data" in json)) throw new Error("Invalid response from Contentful")
  return rss({
    site,
    title: "The Union Blog",
    description:
      "Union is a hyper-efficient, zero-knowledge interoperability layer that connects Appchains, Layer 1, and Layer 2 networks.",
    items: json.data.blogCollection.items.map(post => ({
      title: post.title,
      author: post.author,
      link: `/blog/${post.slug}/`,
      pubDate: new Date(post.date),
      description: post.description
    })),
    // (optional) inject custom xml
    customData: `<language>en-us</language>`
  })
}
