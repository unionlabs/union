import {
  CONTENTFUL_SPACE_ID,
  CONTENTFUL_ENVIRONMENT,
  CONTENTFUL_PREVIEW_TOKEN,
  CONTENTFUL_DELIVERY_TOKEN
} from "astro:env/client"
import contentful from "contentful"

export const contentfulClient = contentful.createClient({
  space: CONTENTFUL_SPACE_ID,
  environment: CONTENTFUL_ENVIRONMENT,
  host: import.meta.env.DEV ? "preview.contentful.com" : "cdn.contentful.com",
  accessToken: import.meta.env.DEV ? CONTENTFUL_PREVIEW_TOKEN : CONTENTFUL_DELIVERY_TOKEN
})
