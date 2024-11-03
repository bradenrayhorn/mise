<script lang="ts">
  import { defaults, superForm } from 'sveltekit-superforms';
  import { zod } from 'sveltekit-superforms/adapters';
  import TitleField from '$lib/components/recipes/form/TitleField.svelte';
  import NotesField from '$lib/components/recipes/form/NotesField.svelte';
  import IngredientsField from '$lib/components/recipes/form/IngredientsField.svelte';
  import { schema, type RecipeFormType } from '$lib/components/recipes/form/schema';
  import ImageField from '$lib/components/recipes/form/ImageField.svelte';
  import TagsField from '$lib/components/recipes/form/TagsField.svelte';
  import { updateRecipe } from '$lib/api/action/recipe';
  import { handleSuperformError } from '$lib/error-to-superform';
  import InstructionsField from '$lib/components/recipes/form/InstructionsField.svelte';
  import { goto } from '$app/navigation';
  import Button from '$lib/components/Button.svelte';
  import type { DetailedRecipe } from '$lib/types/recipe';
  import { useQueryClient } from '@tanstack/svelte-query';
  import { queryKeys } from '$lib/api/query-keys';
  import { useAuth } from '$lib/auth-context';
  import { uid } from 'uid';
  import { sticky } from '$lib/actions/sticky';

  export let backURL: string;
  export let id: string;
  export let recipe: DetailedRecipe;
  export let hash: string;
  export let initialData: RecipeFormType;

  const client = useQueryClient();
  const auth = useAuth();

  const superform = superForm(defaults(initialData, zod(schema)), {
    SPA: true,
    dataType: 'json',
    validators: zod(schema),
    resetForm: false,
    onUpdate: async function ({ form }) {
      if (!form.valid) {
        return;
      }

      try {
        await updateRecipe({
          fetch,
          id: id,
          hash: hash,
          currentRecipe: recipe,
          recipe: {
            title: form.data.title,
            image: form.data.image,
            notes: form.data.notes,
            ingredient_blocks: form.data.ingredient_blocks,
            instruction_blocks: form.data.instruction_blocks,
            tags: form.data.tags,
          },
        });

        await client.invalidateQueries({ queryKey: queryKeys.recipe.get(id) });
        await client.invalidateQueries({ queryKey: [queryKeys.recipe.list] });
        await goto(backURL);
      } catch (error) {
        await handleSuperformError(form, error, auth);
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
    class="flex justify-between items-baseline sticky bg-base-500 top-0 z-10 px-4 md:px-8 lg:px-12 pb-4 ptsafe-4 md:ptsafe-8 transition-all stuck:shadow-md stuck:bg-base-600"
  >
    <h1 class="text-xl md:text-3xl font-serif font-bold">Edit Recipe</h1>

    <div class="flex gap-2">
      <a href={backURL} class="btn-solid btn-gray">Cancel</a>
      <Button type="submit" class="btn-solid btn-primary" isLoading={$submitting}>Save</Button>
    </div>
  </div>

  <div class="flex flex-col md:flex-row gap-8 px-4 md:px-8 lg:px-12">
    <div class="flex-1 flex flex-col gap-6">
      <TitleField {superform} />

      <ImageField {superform} />

      <NotesField {superform} />

      <TagsField {superform} />
    </div>

    <section class="flex-1" aria-labelledby={ingredientsSectionID}>
      <h2 class="text-xl font-bold mb-4 md:mb-6" id={ingredientsSectionID}>Ingredients</h2>

      <IngredientsField {superform} />
    </section>

    <section class="flex-1" aria-labelledby={instructionsSectionID}>
      <h2 class="text-xl font-bold mb-4 md:mb-6" id={instructionsSectionID}>Instructions</h2>

      <InstructionsField {superform} />
    </section>
  </div>
</form>
