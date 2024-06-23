import type { APICall } from '../fetch';
import { handleAPIError } from '../handle-error';

export async function uploadImage({ fetch: _fetch, url, image }: APICall & { image: File }) {
  const formData = new FormData();
  formData.append('file', image);

  const res = await _fetch(`/api/v1/images`, {
    method: 'POST',
    body: formData,
  });

  if (!res.ok) {
    await handleAPIError(res, url);
  }

  return res.json().then((json: { data: string }) => json.data);
}
