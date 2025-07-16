import { redirect, type Load } from "@sveltejs/kit"

export const load: Load = () => {
    throw redirect(302, "/transfer")
} 