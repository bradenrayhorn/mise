import type { DetailedRecipeWithHash, ListedRecipe, RecipePage } from '$lib/types/recipe';
import { error } from '@sveltejs/kit';
import type { WithFetch } from '../fetch';
import type { Tag } from '$lib/types/tag';

export const getRecipes = async ({
  fetch: _fetch,
  cursor,
  search,
  tags,
}: WithFetch & { cursor: string; search: string; tags: string }): Promise<RecipePage> => {
  type Response = {
    data: Array<ListedRecipe>;
    next: string | null;
  };

  const params = new URLSearchParams();

  if (cursor.trim().length > 0) {
    params.set('next', cursor);
  }
  if (search.trim().length > 0) {
    params.set('title', search);
  }
  if (tags.trim().length > 0) {
    params.set('tag_ids', tags);
  }

  const res = await _fetch('/api/v1/recipes?' + params.toString());

  if (!res.ok) {
    return error(500, 'oh no');
    //return await getError(res);
  }

  return await res.json().then((json: Response) => ({
    data: json.data,
    next: json.next ?? undefined,
  }));
};

export const getRecipe = async ({
  fetch: _fetch,
  id,
}: WithFetch & { id: string }): Promise<DetailedRecipeWithHash> => {
  type Response = {
    data: DetailedRecipeResponse;
  };

  type DetailedRecipeResponse = {
    id: string;
    hash: string;
    title: string;
    image_id: string | null;
    notes: string | null;
    tags: Array<Tag>;
    ingredient_blocks: Array<IngredientBlockResponse>;
  };

  type IngredientBlockResponse = {
    title: string | null;
    ingredients: Array<string>;
  };

  const res = await _fetch(`/api/v1/recipes/${id}`);

  if (!res.ok) {
    return error(500, 'oh no');
    //return await getError(res);
  }

  return await res.json().then((json: Response) => ({
    hash: json.data.hash,
    recipe: {
      ...json.data,
      image_id: json.data.image_id ?? undefined,
      notes: json.data.notes ?? undefined,
      tags: json.data.tags,
      ingredient_blocks: json.data.ingredient_blocks.map((block) => ({
        title: block.title ?? undefined,
        ingredients: block.ingredients,
      })),
    },
  }));
};
