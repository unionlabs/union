import { browser } from "$app/environment";

export const port = 4919;
export const host = `http://localhost:${port}`;

type Fetch = (input: RequestInfo | URL, init?: RequestInit | undefined) => Promise<Response>;
export type Params = Record<string, string | number | boolean | undefined>;

export const get = async <T>(resource: string, params: Params, credentials = false, _fetch: Fetch = fetch): Promise<T | undefined> => {
  try {
    const url = new URL(`${host}/${resource}`);
    Object.entries(params).forEach(([key, value]) => value !== undefined && url.searchParams.set(key, `${value}`));
    const res = await _fetch(url, browser && credentials ? { credentials: 'include' } : {});
    if (!res.ok) throw res.status;
    const data: T = await res.json();
    return data ?? undefined;
  } catch (error) {
    console.error("Error during get req:", error);
    return undefined;
  }
};

export const post = async <T>(resource: string, params: Params, body: object, credentials = false, _fetch = fetch): Promise<T | undefined> => {
  try {
    const url = new URL(`${host}/${resource}`);
    Object.entries(params).forEach(([key, value]) => value !== undefined && url.searchParams.set(key, `${value}`));

    console.log('POST URL:', url.href);  // Log the full URL for debugging
    console.log('POST Body:', body);  // Log the body for debugging

    const res = await _fetch(url, {
      ...(browser && credentials ? { credentials: 'include' } : {}),
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',  // Ensure JSON is sent
      },
      body: JSON.stringify(body),  // Send the body as JSON
    });

    console.log('Response Status:', res.status);  // Log the response status

    if (!res.ok) {
      const errorText = await res.text();  // Get the response text if it's an error
      console.error('Response Error Text:', errorText);
      throw new Error(`Server responded with status ${res.status}: ${errorText}`);
    }

    // Check for empty response or status 202 (Accepted) where no body might be returned
    if (res.status === 202 || res.status === 204 || !res.headers.get('content-length')) {
      return undefined;  // No content to return
    }

    // Parse JSON response if available
    const data: T = await res.json();
    console.log('Response Data:', data);
    return data ?? undefined;

  } catch (error) {
    console.error("Error during post req:", error);
    return undefined;
  }
};
