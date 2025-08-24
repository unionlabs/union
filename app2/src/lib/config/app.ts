import type { Edition } from "$lib/themes"
import base64Icon from "./base64.txt?raw"

const projectIds: Record<Edition, string> = {
  app: "f544d5ee6eb61962408fd456c114e9ed",
  btc: "49fe74ca5ded7142adefc69a7788d14a",
}

export function getAppInfo(edition: Edition) {
  const config = {
    base64Icon,
    name: "Union",
    baseUrl: `https://${edition}.union.build`,
    docs: "https://docs.union.build",
    iconUrl: "https://app.union.build/images/logo.png",
    projectId: projectIds[edition],
  }

  console.log(`[getAppInfo] Loaded config for edition "${edition}":`, config)

  return config
}
