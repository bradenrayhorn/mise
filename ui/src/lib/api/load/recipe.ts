import type { DetailedRecipeWithHash, ListedRecipe, RecipePage } from '$lib/types/recipe';
import type { APICall } from '../fetch';
import type { Tag } from '$lib/types/tag';
import { handleAPIError } from '../handle-error';

export const getRecipes = async ({
  fetch: _fetch,
  url,
  cursor,
  search,
  tags,
}: APICall & { cursor: string; search: string; tags: string }): Promise<RecipePage> => {
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
    return await handleAPIError(res, url);
  }

  return await res.json().then((json: Response) => ({
    data: json.data,
    next: json.next ?? undefined,
  }));
};

export const getRecipe = async ({
  fetch: _fetch,
  url,
  id,
}: APICall & { id: string }): Promise<DetailedRecipeWithHash> => {
  type Response = {
    data: DetailedRecipeResponse;
  };

  type DetailedRecipeResponse = {
    id: string;
    hash: string;
    title: string;
    image_id: string | null;
    notes: string | null;
    rich_notes: string | null;
    tags: Array<Tag>;
    ingredient_blocks: Array<IngredientBlockResponse>;
    instruction_blocks: Array<InstructionBlockResponse>;
  };

  type IngredientBlockResponse = {
    title: string | null;
    ingredients: Array<string>;
  };
  type InstructionBlockResponse = {
    title: string | null;
    instructions: Array<string>;
    rich_instructions: Array<string>;
  };

  const res = await _fetch(`/api/v1/recipes/${id}`);

  if (!res.ok) {
    return await handleAPIError(res, url);
  }

  return await res.json().then((json: Response) => ({
    hash: json.data.hash,
    recipe: {
      id: json.data.id,
      hash: json.data.hash,
      title: json.data.title,
      image_id: json.data.image_id ?? undefined,
      notes: json.data.notes ?? undefined,
      rich_notes: json.data.rich_notes ?? undefined,
      tags: json.data.tags,
      ingredient_blocks: json.data.ingredient_blocks.map((block) => ({
        title: block.title ?? undefined,
        ingredients: block.ingredients,
      })),
      instruction_blocks: json.data.instruction_blocks.map((block) => ({
        title: block.title ?? undefined,
        instructions: block.instructions,
        rich_instructions: block.rich_instructions,
      })),
    },
  }));
};
