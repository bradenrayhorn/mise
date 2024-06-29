import { getRecipes } from '$lib/api/load/recipe';
import { getTags } from '$lib/api/load/tag';
import { streamedPromise } from '$lib/handle-streamed-error';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, url }) => {
  const cursor = url.searchParams.get('cursor') ?? '';
  const search = url.searchParams.get('search') ?? '';
  const tags = url.searchParams.get('tags') ?? '';

  localStorage.setItem('last-recipes-query', url.searchParams.toString());

  return {
    promisedRecipePage: streamedPromise(getRecipes({ fetch, url, cursor, search, tags })),
    promisedTags: streamedPromise(getTags({ fetch, url })),
  };
};
