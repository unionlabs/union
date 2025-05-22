import { redirect } from "@sveltejs/kit";

export const load = async () => {
  // Temporary redirect from sign-in page
  throw redirect(302, "/");
};
