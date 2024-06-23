import { redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ url }) => {
  return redirect(307, `/auth/init?${url.searchParams.toString()}`);
};
