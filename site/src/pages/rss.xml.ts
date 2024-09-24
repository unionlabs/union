import rss from "@astrojs/rss"
import type { APIContext } from "astro"
import { getCollection } from "astro:content"

export async function GET(context: APIContext) {
  const blog = await getCollection("blog")
  const site = context.site
  if (!site) throw new Error("Missing site metadata")

  return rss({
    site,
    title: "The Union Blog",
    description:
      "Union is a hyper-efficient, zero-knowledge interoperability layer that connects Appchains, Layer 1, and Layer 2 networks.",
    items: blog
      // @ts-expect-error
      .filter(post => !post.data.hidden)
      // @ts-expect-error
      .map(post => ({
        title: post.data.title,
        pubDate: post.data.date,
        link: `/blog/${post.slug}/`,
        description: post.data.description
      })),
    // (optional) inject custom xml
    customData: `<language>en-us</language>`
  })
}
