import { getRecipe } from '$lib/api/requests/recipe';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, params }) => {
  const { recipe } = await getRecipe({ fetch, id: params.id });

  return { recipe, id: params.id };
};
