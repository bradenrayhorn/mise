import type { DetailedRecipe, IngredientBlock, InstructionBlock } from '$lib/types/recipe';
import type { TagOnRecipe } from '$lib/types/tag';
import type { APICall } from '../fetch';
import { handleAPIError } from '../handle-error';
import { uploadImage } from './image';

type RecipeParams = {
  title: string;
  image?: File;
  notes: string;
  ingredient_blocks: Array<IngredientBlock>;
  instruction_blocks: Array<InstructionBlock>;
  tags: Array<TagOnRecipe>;
};

export async function createRecipe({
  fetch: _fetch,
  recipe: { image, ...recipe },
}: APICall & { recipe: RecipeParams }) {
  const res = await fetch('/api/v1/recipes', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
    },
    body: JSON.stringify({
      ...cleanRecipeParams(recipe),
      image_id: image ? await uploadImage({ fetch: _fetch, image }) : undefined,
    }),
  });

  if (!res.ok) {
    await handleAPIError(res);
  }
}

export async function updateRecipe({
  fetch: _fetch,
  id,
  hash,
  currentRecipe,
  recipe: { image, ...recipe },
}: APICall & { id: string; hash: string; currentRecipe: DetailedRecipe; recipe: RecipeParams }) {
  let image_id = currentRecipe.image_id;
  if (image) {
    // only upload new image if the image has changed
    if (image.type !== 'mise/image_id') {
      image_id = await uploadImage({ fetch: _fetch, image });
    }
  } else {
    image_id = undefined;
  }

  const res = await fetch(`/api/v1/recipes/${id}`, {
    method: 'PUT',
    headers: {
      'content-type': 'application/json',
    },
    body: JSON.stringify({
      ...cleanRecipeParams(recipe),
      previous_hash: hash,
      image_id,
    }),
  });

  if (!res.ok) {
    await handleAPIError(res);
  }
}

function cleanRecipeParams(params: Omit<RecipeParams, 'image'>) {
  return {
    title: params.title,
    notes: params.notes.trim().length > 0 ? params.notes.trim() : null,
    ingredients: params.ingredient_blocks
      .map((b) => ({
        ...b,
        ingredients: b.ingredients.filter((i) => i.trim().length > 0),
      }))
      .filter((b) => b.ingredients.length > 0),
    instructions: params.instruction_blocks
      .map((b) => ({
        ...b,
        instructions: b.instructions.filter((i) => i.trim().length > 0),
      }))
      .filter((b) => b.instructions.length > 0),
    tag_ids: params.tags.map((t) => t.id),
  };
}
