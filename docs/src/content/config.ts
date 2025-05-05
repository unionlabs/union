import { docsSchema, i18nSchema } from "@astrojs/starlight/schema"
import { defineCollection } from "astro:content"

export const collections = {
  docs: defineCollection({ schema: docsSchema() }),
  i18n: defineCollection({ type: "data", schema: i18nSchema() }),
}
