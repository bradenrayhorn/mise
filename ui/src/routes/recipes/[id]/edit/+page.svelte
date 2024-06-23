<script lang="ts">
  import { superForm } from 'sveltekit-superforms';
  import type { PageData } from './$types';
  import { zodClient } from 'sveltekit-superforms/adapters';
  import TitleField from '$lib/components/recipes/form/TitleField.svelte';
  import NotesField from '$lib/components/recipes/form/NotesField.svelte';
  import IngredientsField from '$lib/components/recipes/form/IngredientsField.svelte';
  import { schema } from '$lib/components/recipes/form/schema';
  import ImageField from '$lib/components/recipes/form/ImageField.svelte';
  import TagsField from '$lib/components/recipes/form/TagsField.svelte';
  import { uploadImage } from '$lib/api/requests/image';

  export let data: PageData;
  const superform = superForm(data.form, {
    SPA: true,
    dataType: 'json',
    validators: zodClient(schema),
    resetForm: false,
    onUpdate: async function ({ form }) {
      console.log({ form }, form.data.image);
      if (form.valid) {
        let image_id = data.recipe.image_id;
        if (form.data.image) {
          // only upload new image if the image has changed
          if (form.data.image.name !== data.hash) {
            image_id = await uploadImage(form.data.image);
          }
        } else {
          image_id = undefined;
        }

        await fetch(`/api/v1/recipes/${data.id}`, {
          method: 'PUT',
          headers: {
            'content-type': 'application/json',
          },
          body: JSON.stringify({
            previous_hash: data.hash,
            title: form.data.title,
            notes: form.data.notes.trim().length > 0 ? form.data.notes.trim() : null,
            image_id,
            ingredients: form.data.ingredient_blocks
              .map((b) => ({ ...b, ingredients: b.ingredients.filter((i) => i.trim().length > 0) }))
              .filter((b) => b.ingredients.length > 0),
            instructions: [],
            tag_ids: form.data.tags.map((t) => t.id),
          }),
        });
      }
    },
  });

  const { form, errors, enhance } = superform;
</script>

<form method="POST" use:enhance>
  <div class="flex justify-between mb-16">
    <h1 class="font-bold text-3xl font-serif">edit recipe</h1>

    <button type="submit">Save</button>
  </div>

  <div class="flex flex-col md:flex-row gap-8 px-2">
    <div class="flex-1 flex flex-col">
      <h2>Attributes</h2>

      <ImageField {superform} />

      <TitleField bind:value={$form.title} errors={$errors.title} />

      <NotesField bind:value={$form.notes} errors={$errors.notes} />

      <TagsField {superform} promisedTags={data.tags} />
    </div>

    <div class="flex-1">
      <h2>Ingredients</h2>

      <IngredientsField {superform} />
    </div>

    <div class="flex-1">
      <h2>Instructions</h2>
    </div>
  </div>
</form>
