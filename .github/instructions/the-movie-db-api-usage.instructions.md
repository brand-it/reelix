---
applyTo: "src-tauri/src/the_movie_db/**"
---

# TMDB API usage instructions (v3 reference)

Use these rules whenever writing code that calls The Movie Database (TMDB) API.

Full API reference: https://developer.themoviedb.org/reference/getting-started

## Base URLs
- TMDB API (v3): `https://api.themoviedb.org/3`
- TMDB images: `https://image.tmdb.org/t/p/` (final URL includes a size segment like `w500` and the returned `file_path`). :contentReference[oaicite:1]{index=1}

## Authentication
Prefer **Bearer token** auth:
- Send header: `Authorization: Bearer <TMDB_API_READ_ACCESS_TOKEN>` :contentReference[oaicite:2]{index=2}
- Alternative (legacy): `api_key=<TMDB_API_KEY>` query param is also supported for v3. :contentReference[oaicite:3]{index=3}
- Never hardcode secrets. Read from env vars, e.g. `TMDB_ACCESS_TOKEN` or `TMDB_API_KEY`.

## Request defaults
- Use JSON.
- Build a small reusable HTTP client wrapper:
  - base URL + auth injection
  - query param helper
  - consistent error handling
  - retry/backoff only for transient failures (429/5xx)

## Pagination rules
- Many list/search endpoints return paginated results.
- Pages start at **1** and max at **500**. Validate page inputs before calling. :contentReference[oaicite:4]{index=4}

## Efficient fetching: append_to_response
For detail endpoints (movie, TV, person, etc.), use `append_to_response` to reduce multiple round trips:
- Example: `/movie/{id}?append_to_response=videos,images` :contentReference[oaicite:5]{index=5}
- Max appended remote calls is **20**; don’t exceed it. :contentReference[oaicite:6]{index=6}
- Remember appended sections still respect their own query params; be careful with image language filtering. :contentReference[oaicite:7]{index=7}

## Images
To build a full image URL you need:
- `base_url` and supported `file_size` values from `/configuration`
- the `file_path` returned on objects (e.g. `poster_path`) :contentReference[oaicite:8]{index=8}

Rules:
- Cache `/configuration` aggressively (it rarely changes) and reuse sizes.
- Construct URL: `{base_url}{size}{file_path}` (example size: `w500`). :contentReference[oaicite:9]{index=9}

## Rate limiting and throttling
- Legacy limit (40 requests / 10 seconds) is disabled, but TMDB still enforces upper limits (roughly around ~40 req/sec range) and may change it. Respect `429`. :contentReference[oaicite:10]{index=10}
- Implement:
  - short exponential backoff + jitter on 429
  - a client-side limiter for burst control if doing batch workloads

## Error handling (common)
Map TMDB error responses to actionable outcomes using their error codes list. :contentReference[oaicite:11]{index=11}

Practical handling:
- 401 (auth): surface “invalid/expired token” clearly; don’t retry blindly.
- 404 (not found): return null/empty result; don’t retry.
- 422/400 (bad params): treat as developer error; include details.
- 429 (rate limited): retry with backoff; also reduce concurrency.
- 5xx/503/504: retry with backoff up to a small max; then fail.

## Implementation style preferences (for this repo)
- Keep a single `tmdbClient` module that exposes typed methods (or clearly structured functions).
- Prefer small functions per endpoint group (movies, tv, search, people).
- Add unit tests for URL building (especially images and query params).
- Log safely: never print tokens, API keys, or full auth headers.
