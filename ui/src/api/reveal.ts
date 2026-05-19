/** Ask the server to open the OS file manager at `segments` (resolved
 *  relative to the scan root). Throws if the server returns non-2xx —
 *  the caller decides how to surface the failure (toast, label flash). */
export async function revealInFolder(segments: readonly string[]): Promise<void> {
  const res = await fetch("/reveal", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ segments }),
  });
  if (!res.ok) {
    const detail = await res.text().catch(() => "");
    throw new Error(`/reveal returned ${res.status}: ${detail}`);
  }
}
