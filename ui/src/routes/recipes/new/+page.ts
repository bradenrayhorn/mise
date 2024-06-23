import { getTags } from '$lib/api/requests/tag';
import { schema } from '$lib/components/recipes/form/schema';
import type { PageLoad } from './$types';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';

export const load: PageLoad = async ({ fetch }) => {
  const form = await superValidate(zod(schema), {
    defaults: { title: '', notes: '', ingredient_blocks: [{ ingredients: [''] }], tags: [] },
  });

  return { form, tags: getTags({ fetch }) };
};
