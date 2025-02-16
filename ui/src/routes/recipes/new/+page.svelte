<script lang="ts">
  import { defaults, superForm } from 'sveltekit-superforms';
  import { zod } from 'sveltekit-superforms/adapters';
  import TitleField from '$lib/components/recipes/form/TitleField.svelte';
  import NotesField from '$lib/components/recipes/form/NotesField.svelte';
  import IngredientsField from '$lib/components/recipes/form/IngredientsField.svelte';
  import { schema } from '$lib/components/recipes/form/schema';
  import ImageField from '$lib/components/recipes/form/ImageField.svelte';
  import { createRecipe } from '$lib/api/action/recipe';
  import TagsField from '$lib/components/recipes/form/TagsField.svelte';
  import { handleSuperformError } from '$lib/error-to-superform';
  import InstructionsField from '$lib/components/recipes/form/InstructionsField.svelte';
  import { goto } from '$app/navigation';
  import Button from '$lib/components/Button.svelte';
  import { queryKeys } from '$lib/api/query-keys';
  import { useQueryClient } from '@tanstack/svelte-query';
  import { uid } from 'uid';
  import { sticky } from '$lib/actions/sticky';

  const backURL = `/recipes?${localStorage.getItem('last-recipes-query')}`;

  const client = useQueryClient();

  const initialData = {
    title: '',
    notes: '',
    ingredient_blocks: [{ ingredients: [''] }],
    instruction_blocks: [{ instructions: [''] }],
    tags: [],
  };

  const superform = superForm(defaults(initialData, zod(schema)), {
    SPA: true,
    dataType: 'json',
    validators: zod(schema),
    onUpdate: async function ({ form }) {
      if (!form.valid) {
        return;
      }

      try {
        await createRecipe({
          fetch,
          recipe: {
            title: form.data.title,
            image: form.data.image,
            notes: form.data.notes,
            ingredient_blocks: form.data.ingredient_blocks,
            instruction_blocks: form.data.instruction_blocks,
            tags: form.data.tags,
          },
        });

        await client.invalidateQueries({ queryKey: [queryKeys.recipe.list] });
        await goto('/recipes');
      } catch (error) {
        await handleSuperformError(form, error);
      }
    },
  });

  const { enhance, submitting } = superform;

  const ingredientsSectionID = uid();
  const instructionsSectionID = uid();
</script>

<form method="POST" use:enhance class="pbsafe-8">
  <div
    use:sticky
    class="flex justify-between items-baseline sticky bg-base-500 top-0 z-10 px-4 md:px-8 xl:px-12 pb-4 ptsafe-4 md:ptsafe-8 transition-all stuck:shadow-md stuck:bg-base-600"
  >
    <h1 class="text-xl md:text-3xl font-serif text-fg-highlight font-bold">Add Recipe</h1>

    <div class="flex gap-2">
      <a href={backURL} class="btn-solid btn-gray">Cancel</a>
      <Button type="submit" class="btn-solid btn-primary" isLoading={$submitting}>Save</Button>
    </div>
  </div>

  <div class="grid grid-cols-1 lg:grid-cols-2 gap-8 px-4 md:px-8 xl:px-12">
    <div class="flex-1 flex flex-col gap-6 lg:col-span-2 max-w-[75ch]">
      <TitleField {superform} />

      <ImageField {superform} />

      <NotesField {superform} />

      <TagsField {superform} />
    </div>

    <section class="flex-1 max-w-[75ch]" aria-labelledby={ingredientsSectionID}>
      <h2 class="text-xl font-bold mb-4 md:mb-6" id={ingredientsSectionID}>Ingredients</h2>

      <IngredientsField {superform} />
    </section>

    <section class="flex-1 max-w-[75ch]" aria-labelledby={instructionsSectionID}>
      <h2 class="text-xl font-bold mb-4 md:mb-6" id={instructionsSectionID}>Directions</h2>

      <InstructionsField {superform} />
    </section>
  </div>
</form>
