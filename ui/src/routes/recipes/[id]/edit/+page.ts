import { getRecipe } from '$lib/api/requests/recipe';
import { superValidate } from 'sveltekit-superforms';
import type { PageLoad } from './$types';
import { zod } from 'sveltekit-superforms/adapters';
import { schema } from '$lib/components/recipes/form/schema';
import { getTags } from '$lib/api/requests/tag';

export const load: PageLoad = async ({ fetch, params }) => {
  const { recipe, hash } = await getRecipe({ fetch, id: params.id });

  const form = await superValidate(zod(schema), {
    defaults: {
      title: recipe.title,
      notes: recipe.notes ?? '',
      image: await getImageBlob(fetch, recipe.image_id, hash),
      tags: recipe.tags,
      ingredient_blocks: recipe.ingredient_blocks.map((block) => ({
        title: block.title,
        ingredients: [...block.ingredients, ''],
      })),
    },
  });

  return { form, recipe, hash, id: params.id, tags: getTags({ fetch }) };
};

async function getImageBlob(_fetch: typeof fetch, imageID: string | undefined, hash: string) {
  return imageID
    ? new File(
        [await _fetch(`/api/v1/images/${imageID}`).then((response) => response.blob())],
        hash,
      )
    : undefined;
}
