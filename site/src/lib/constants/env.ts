import { raise } from "#/lib/utilities.ts"

export const env = Object.freeze({
  MODE: getEnvVariable("MODE"),
  SITE_URL: getEnvVariable("SITE_URL", { optional: true }),
  CONTENTFUL_SPACE_ID: getEnvVariable("PUBLIC_CONTENTFUL_SPACE_ID"),
  CONTENTFUL_ENVIRONMENT: getEnvVariable("PUBLIC_CONTENTFUL_ENVIRONMENT"),
  CONTENTFUL_ACCESS_TOKEN: getEnvVariable("PUBLIC_CONTENTFUL_ACCESS_TOKEN"),
  CONTENTFUL_PREVIEW_TOKEN: getEnvVariable("PUBLIC_CONTENTFUL_PREVIEW_TOKEN"),
  CONTENTFUL_DELIVERY_TOKEN: getEnvVariable("PUBLIC_CONTENTFUL_DELIVERY_TOKEN")
})

function getEnvVariable<T extends keyof ImportMetaEnv>(
  name: T,
  { optional = false } = {}
): ImportMetaEnv[T] {
  const value = import.meta.env[name]
  if (optional && value === undefined) {
    console.warn(`Missing environment variable ${name}`)
    return undefined
  }
  return value ?? raise(`Missing environment variable ${name}`)
}
