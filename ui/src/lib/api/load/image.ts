import type { APICall } from '../fetch';
import { handleAPIError } from '../handle-error';

export const getImageBlob = async ({
  fetch: _fetch,
  url,
  imageID,
}: APICall & { imageID: string | undefined }): Promise<Blob | undefined> => {
  if (!imageID) {
    return undefined;
  }

  const res = await _fetch(`/api/v1/images/${imageID}`);

  if (!res.ok) {
    await handleAPIError(res, url);
  }

  return await res.blob();
};
