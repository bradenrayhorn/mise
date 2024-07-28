<script lang="ts">
  import { page } from '$app/stores';
  import IconChef from '~icons/mdi/chef-hat';
</script>

<div class="bg-base-500 fixed top-0 left-0 bottom-0 right-0 flex justify-center items-center">
  <div class="flex flex-col items-center gap-4">
    <IconChef class="text-7xl md:text-8xl text-fg-default" />

    <span>Your session has expired. Please login again.</span>

    <a class="btn-solid btn-primary" href={`/auth/init?redirect_target=${$page.url.pathname}`}
      >Log in</a
    >
  </div>
</div>
