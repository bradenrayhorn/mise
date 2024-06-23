<script lang="ts">
  import { createDialog, melt, type AnyMeltElement } from '@melt-ui/svelte';
  import { defaults, superForm } from 'sveltekit-superforms';
  import { zod } from 'sveltekit-superforms/adapters';
  import { schema } from './tag-schema';

  export let element: AnyMeltElement;

  const {
    elements: { trigger, portalled, overlay, content, title, close },
    states: { open },
  } = createDialog();

  const { form, errors, enhance } = superForm(defaults(zod(schema)), {
    SPA: true,
    validators: zod(schema),
    onUpdate: async function ({ form }) {
      console.log({ form });
      if (form.valid) {
        await fetch(`/api/v1/tags`, {
          method: 'POST',
          headers: {
            'content-type': 'application/json',
          },
          body: JSON.stringify({
            name: form.data.name,
          }),
        });
      }
    },
  });
</script>

<button use:melt={$element} use:melt={$trigger}>Create New Tag</button>

<!-- TODO - when styling for mobile, instead of a modal bring form up as a drawer from bottom -->

{#if $open}
  <div use:melt={$portalled}>
    <div use:melt={$overlay} />
    <div use:melt={$content}>
      <form method="POST" use:enhance>
        <h2 use:melt={$title}>Create new tag</h2>
        <div>
          <label>
            Name
            <input aria-invalid={$errors.name ? 'true' : undefined} bind:value={$form.name} />
          </label>
          {#if $errors.name}<span class="invalid">{$errors.name}</span>{/if}
        </div>
        <button>Save</button>
        <button use:melt={$close}>Cancel</button>
      </form>
    </div>
  </div>
{/if}
