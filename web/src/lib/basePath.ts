// Runtime base path injected by the Rust gateway into index.html.
// Allows the SPA to work under a reverse-proxy path prefix.

export const BASE_PATH_GLOBAL = '__R.A.I.N._BASE__';

declare global {
  interface Window {
    [BASE_PATH_GLOBAL]?: string;
  }
}

/** Gateway path prefix (e.g. "/R.A.I.N."), or empty string when served at root. */
export const basePath: string = (window[BASE_PATH_GLOBAL] ?? '').replace(/\/+$/, '');
