import { getSelf } from '$lib/api/load/user';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, url }) => {
  await getSelf({ fetch, url });

  const backURL = `/recipes?${localStorage.getItem('last-recipes-query')}`;

  return { backURL };
};
