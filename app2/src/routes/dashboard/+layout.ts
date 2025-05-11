import { redirect } from '@sveltejs/kit';
import type { LayoutLoad } from './$types';
import { dashboard } from '$lib/dashboard/stores/user.svelte';
import { Option } from 'effect';

export const load: LayoutLoad = async () => {
  if (Option.isNone(dashboard.session)) {
    throw redirect(302, '/');
  }
  
  return {
    session: dashboard.session
  };
};
