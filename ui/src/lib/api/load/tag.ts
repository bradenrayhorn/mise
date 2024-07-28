import type { APICall } from '../fetch';
import type { Tag } from '$lib/types/tag';
import { handleAPIError } from '../handle-error';

export const getTags = async ({ fetch: _fetch }: APICall): Promise<Array<Tag>> => {
  type Response = {
    data: Array<Tag>;
  };

  const res = await _fetch('/api/v1/tags');

  if (!res.ok) {
    await handleAPIError(res);
  }

  return await res.json().then((json: Response) => json.data);
};
