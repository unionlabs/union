export type Edition = "btc" | "app"

export interface Theme {
  label: string
  accent: string
  primary: string
  background: string
  videoUrl: string
  staticImage: string
}

export const themes: Record<Edition, Theme> = {
  btc: {
    label: "BTC",
    accent: "var(--color-babylon-orange)",
    primary: "var(--color-babylon-orange)",
    background: "var(--color-zinc-950)",
    videoUrl: "https://videos.cdn.union.build/btc-union-background-3.webm",
    staticImage: "/btc-static-video.png",
  },
  app: {
    label: "V4",
    accent: "var(--color-union)",
    primary: "var(--color-union)",
    background: "var(--color-zinc-950)",
    videoUrl: "https://videos.cdn.union.build/app-union-background-11.webm",
    staticImage: "/app-static-video.png",
  },
}
