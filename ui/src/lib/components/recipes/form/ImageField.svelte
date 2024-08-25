<script lang="ts">
  import { type SuperForm, type Infer, fileProxy, formFieldProxy } from 'sveltekit-superforms';
  import { type RecipeFormSchema } from './schema';

  export let superform: SuperForm<Infer<RecipeFormSchema>>;

  const file = fileProxy(superform, 'image');
  const { errors } = formFieldProxy(superform, 'image');
</script>

{#if $file.length === 1}
  <img
    src={$file[0].type === 'mise/image_id'
      ? `/api/v1/images/${$file[0].name}`
      : URL.createObjectURL($file[0])}
    alt="Uploaded recipe"
    class="w-full max-w-80 h-56 object-cover rounded shadow-inner"
  />
{/if}

<label>
  <span>Image</span>
  <input
    class="input text-sm block"
    type="file"
    bind:files={$file}
    accept="image/png, image/jpeg"
  />
</label>
{#if $errors}<span class="invalid">{$errors}</span>{/if}
