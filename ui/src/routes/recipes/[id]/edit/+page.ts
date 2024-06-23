import { getRecipe } from '$lib/api/load/recipe';
import { superValidate } from 'sveltekit-superforms';
import type { PageLoad } from './$types';
import { zod } from 'sveltekit-superforms/adapters';
import { schema } from '$lib/components/recipes/form/schema';
import { getTags } from '$lib/api/load/tag';
import { streamedPromise } from '$lib/handle-streamed-error';
import { getImageBlob } from '$lib/api/load/image';

export const load: PageLoad = async ({ fetch, url, params }) => {
  const { recipe, hash } = await getRecipe({ fetch, url, id: params.id });

  const imageBlob = await getImageBlob({ fetch, url, imageID: recipe.image_id });

  const form = await superValidate(zod(schema), {
    defaults: {
      title: recipe.title,
      notes: recipe.notes ?? '',
      image: imageBlob ? new File([imageBlob], hash) : undefined,
      tags: recipe.tags,
      ingredient_blocks: recipe.ingredient_blocks.map((block) => ({
        title: block.title,
        ingredients: [...block.ingredients, ''],
      })),
    },
  });

  return {
    form,
    recipe,
    hash,
    id: params.id,
    tags: streamedPromise(getTags({ fetch, url })),
  };
};
