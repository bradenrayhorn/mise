<script lang="ts">
  import { formFieldProxy, type Infer, type SuperForm } from 'sveltekit-superforms';
  import type { RecipeFormSchema } from './schema';

  interface Props {
    superform: SuperForm<Infer<RecipeFormSchema>>;
  }

  const { superform }: Props = $props();

  const { value, errors } = formFieldProxy(superform, 'title');
</script>

<label>
  <span>Title</span>
  <input
    class="input"
    type="text"
    bind:value={$value}
    aria-invalid={$errors ? 'true' : undefined}
  />
</label>
{#if $errors}<span class="invalid">{$errors}</span>{/if}
