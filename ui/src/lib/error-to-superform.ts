import { goto } from '$app/navigation';
import { setMessage, type SuperValidated } from 'sveltekit-superforms';

export async function handleSuperformError<T extends Record<string, unknown>>(
  form: SuperValidated<T>,
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  error: any,
  navigate: (path: string) => void,
) {
  console.error('form error: ', error);

  if (error?.status && error?.status >= 300 && error?.status <= 399 && error?.location) {
    navigate(error?.location);
    return {};
  }

  setMessage(form, error?.message ?? 'Unknown error.');
}

export function superformGoto(path: string) {
  goto(path);
}
