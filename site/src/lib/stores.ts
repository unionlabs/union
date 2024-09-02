import { atom } from "nanostores"

export const walletAddress = atom<`0x${string}` | undefined>(undefined)
