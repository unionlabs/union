export const unionAddressRegex = /^union[a-z0-9]{39}$/

export type Message = { status: "error" | "success" | "warning"; text: string }
