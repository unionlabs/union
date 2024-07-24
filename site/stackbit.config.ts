import { defineStackbitConfig } from "@stackbit/types"
import { ContentfulContentSource } from "@stackbit/cms-contentful"

export default defineStackbitConfig({
  stackbitVersion: "~0.6.0",
  nodeVersion: "18",
  ssgName: "astro",
  contentSources: [
    new ContentfulContentSource({
      spaceId: process.env.CONTENTFUL_SPACE_ID,
      previewToken: process.env.CONTENTFUL_PREVIEW_TOKEN,
      accessToken: process.env.CONTENTFUL_MANAGEMENT_TOKEN,
      environment: process.env.CONTENTFUL_ENVIRONMENT || "master"
    })
  ],
  postInstallCommand: "npm i --no-save @stackbit/types @stackbit/cms-contentful"
})
