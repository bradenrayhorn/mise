import { goto } from '$app/navigation';
import { setMessage, type SuperValidated } from 'sveltekit-superforms';

export async function handleSuperformError(
  form: SuperValidated<any>,
  error: any,
  navigate: (path: string) => void,
) {
  console.error('form error: ', error);

  if (error?.status >= 300 && error?.status <= 399 && error?.location) {
    navigate(error?.location);
    return {};
  }

  setMessage(form, error?.message ?? 'Unknown error.');
}

export function superformGoto(path: string) {
  goto(path);
}
