import { getTags } from '$lib/api/load/tag';
import { schema } from '$lib/components/recipes/form/schema';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import type { PageLoad } from './$types';
import { streamedPromise } from '$lib/handle-streamed-error';

export const load: PageLoad = async ({ fetch, url }) => {
  const form = await superValidate(zod(schema), {
    defaults: {
      title: '',
      notes: '',
      ingredient_blocks: [{ ingredients: [''] }],
      instruction_blocks: [{ instructions: [''] }],
      tags: [],
    },
  });

  return {
    form,
    tags: streamedPromise(getTags({ fetch, url })),
  };
};
