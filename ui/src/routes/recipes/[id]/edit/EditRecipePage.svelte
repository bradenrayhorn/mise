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
  import CloseIconButton from '$lib/components/CloseIconButton.svelte';
  import Button from '$lib/components/Button.svelte';
  import type { DetailedRecipe } from '$lib/types/recipe';
  import { useQueryClient } from '@tanstack/svelte-query';
  import { queryKeys } from '$lib/api/query-keys';
  import { useAuth } from '$lib/auth-context';

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
</script>

<div class="absolute top-1 left-1 z-10 flex items-center">
  <CloseIconButton href={backURL} />
</div>

<form method="POST" use:enhance class="pb-8">
  <div class="flex justify-between mb-8 px-4 md:px-8 lg:px-12 pt-12">
    <h1 class="font-bold text-3xl font-serif">Edit Recipe</h1>

    <Button type="submit" class="btn-solid btn-primary" isLoading={$submitting}>Save</Button>
  </div>

  <div class="flex flex-col md:flex-row gap-8 px-4 md:px-8 lg:px-12">
    <div class="flex-1 flex flex-col gap-6">
      <TitleField {superform} />

      <ImageField {superform} />

      <NotesField {superform} />

      <TagsField {superform} />
    </div>

    <div class="flex-1">
      <h2 class="text-xl font-bold font-serif mb-4 md:mb-6">Ingredients</h2>

      <IngredientsField {superform} />
    </div>

    <div class="flex-1">
      <h2 class="text-xl font-bold font-serif mb-4 md:mb-6">Instructions</h2>

      <InstructionsField {superform} />
    </div>
  </div>
</form>
