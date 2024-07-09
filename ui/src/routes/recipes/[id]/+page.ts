import { getRecipe } from '$lib/api/load/recipe';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, url, params }) => {
  const { recipe } = await getRecipe({ fetch, url, id: params.id });

  const backURL = `/recipes?${localStorage.getItem('last-recipes-query')}`;

  return {
    recipe,
    id: params.id,
    backURL,
  };
};
