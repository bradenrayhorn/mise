<script lang="ts">
  import { goto } from '$app/navigation';
  import { useNavigating } from '$lib/navigating-context';

  export let href: string;
  export let prefetch: () => Promise<void>;

  const navigating = useNavigating();

  let isLoading = false;

  async function onClick(e: MouseEvent) {
    e.preventDefault();

    isLoading = true;
    $navigating.to = href;

    try {
      await prefetch();
    } catch (error: unknown) {
      // ignore any prefetch errors and trigger primary page loading state
      console.error('prefetch error: ', error);
    }

    // only navigate if another navigation hasn't triggered
    if ($navigating.to === href) {
      await goto(href);
      $navigating.to = undefined;
    }

    isLoading = false;
  }

  // the navigation is not going to proceed if another navigation has started
  $: if ($navigating.to !== undefined && $navigating.to !== href) {
    isLoading = false;
  }
</script>

<a {href} on:click={onClick} data-loading={isLoading ? true : undefined} {...$$restProps}
  ><slot {isLoading} /></a
>
