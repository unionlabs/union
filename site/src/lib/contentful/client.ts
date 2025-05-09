import { env } from "#/lib/constants/env.ts"
import { createClient } from "contentful"

export const contentfulClient = createClient({
  space: env.CONTENTFUL_SPACE_ID,
  environment: env.CONTENTFUL_ENVIRONMENT,
  host: import.meta.env.DEV ? "preview.contentful.com" : "cdn.contentful.com",
  accessToken: import.meta.env.DEV ? env.CONTENTFUL_PREVIEW_TOKEN : env.CONTENTFUL_DELIVERY_TOKEN,
})
