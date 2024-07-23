import { defineStackbitConfig } from "@stackbit/types"
import { ContentfulContentSource } from "@stackbit/cms-contentful"

/**
 * https://docs.netlify.com/visual-editor/content-sources/contentful/#prerequisites
 */

export default defineStackbitConfig({
  ssgName: "astro",
  nodeVersion: "22.5.1",
  stackbitVersion: "~0.6.0",
  presetReferenceBehavior: "duplicateContents",
  contentSources: [
    new ContentfulContentSource({
      spaceId: process.env.CONTENTFUL_SPACE_ID,
      previewToken: process.env.CONTENTFUL_PREVIEW_TOKEN,
      accessToken: process.env.CONTENTFUL_MANAGEMENT_TOKEN,
      environment: process.env.CONTENTFUL_ENVIRONMENT || "master",
      useWebhookForContentUpdates: true // default is false
    })
  ],
  import: {
    uploadAssets: true,
    type: "contentful",
    assetsDirectory: "contentful",
    contentFile: "contentful/export.json",
    spaceIdEnvVar: "CONTENTFUL_SPACE_ID",
    previewTokenEnvVar: "CONTENTFUL_PREVIEW_TOKEN",
    deliveryTokenEnvVar: "CONTENTFUL_DELIVERY_TOKEN"
  }
})
