import { getRecipes } from '$lib/api/requests/recipe';
import { getTags } from '$lib/api/requests/tag';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, url: { searchParams } }) => {
  const cursor = searchParams.get('cursor') ?? '';
  const search = searchParams.get('search') ?? '';
  const tags = searchParams.get('tags') ?? '';
  const page = getRecipes({ fetch, cursor, search, tags });

  return { page, tags: getTags({ fetch }) };
};
