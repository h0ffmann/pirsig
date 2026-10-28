// ztk's REST API, on the same origin as this page (see desktop/README.md).

export class ApiError extends Error {
  constructor(status, message, data) {
    super(message);
    this.status = status;
    this.data = data;
  }
}

// The words to show for an error answer: validation errors field by field,
// else whatever the framework said.
function describe(data, res) {
  if (data && typeof data === "object") {
    const errors = data.errors ?? data.details?.errors;
    if (errors && typeof errors === "object") {
      return Object.entries(errors)
        .map(([field, list]) =>
          (Array.isArray(list) ? list : [list])
            .map((e) => `${field}: ${e?.message ?? e?.code ?? e}`)
            .join("\n"))
        .join("\n");
    }
    return data.description ?? data.message ?? data.error ?? JSON.stringify(data);
  }
  return data || `${res.status} ${res.statusText}`;
}

function query(params) {
  if (!params) return "";
  const q = new URLSearchParams();
  for (const [k, v] of Object.entries(params)) {
    if (v !== undefined && v !== null && v !== "") q.set(k, v);
  }
  const s = q.toString();
  return s ? `?${s}` : "";
}

async function request(method, path, body) {
  const init = { method, headers: { Accept: "application/json" } };
  if (body !== undefined) {
    init.headers["Content-Type"] = "application/json";
    init.body = JSON.stringify(body);
  }
  const res = await fetch(path, init);
  if (res.status === 204) return null;
  const text = await res.text();
  let data = text;
  try {
    data = text ? JSON.parse(text) : null;
  } catch {
    // not JSON: keep the text
  }
  if (!res.ok) throw new ApiError(res.status, describe(data, res), data);
  return data;
}

export const api = {
  get: (path, params) => request("GET", path + query(params)),
  post: (path, body, params) => request("POST", path + query(params), body ?? {}),
  patch: (path, body) => request("PATCH", path, body),
  del: (path, params) => request("DELETE", path + query(params)),
};

// Every item of a paginated list, page by page.
export async function all(path, params = {}) {
  const items = [];
  for (let page = 1; ; page++) {
    const res = await api.get(path, { ...params, page, page_size: 100 });
    items.push(...res.items);
    if (page >= res.total_pages) return items;
  }
}
