import { useEffect, useRef, useState } from "react";
import { copyText } from "./helpers";

/** Drives the transient "idle → ok/error → idle" label on the Copy
 *  buttons. Manages the post-flash revert timer and skips state updates
 *  if the button is unmounted mid-flight (clipboard `await` can outlive
 *  a drill-in that reparents the detail panel). */
export function useCopyButton() {
  const [state, setState] = useState<"idle" | "ok" | "error">("idle");
  // Hold the pending revert-to-idle timer in a ref so we can cancel it
  // both on unmount and on rapid successive clicks (otherwise an earlier
  // timer would race the newer state and snap the label back to "idle"
  // mid-flash).
  const timerRef = useRef<number | null>(null);
  // The clipboard `await` can resolve after the user has navigated away
  // (e.g. drilled into a different node, which unmounts this button).
  // Skip the post-await `setState` in that case to avoid React's
  // "update on unmounted component" warning.
  const mountedRef = useRef(true);
  // Monotonic op counter: each `run()` invocation bumps it and captures
  // the new value. After `await copyText(...)` returns we check the
  // captured value against the current ref — if a newer click bumped it
  // in the meantime, this completion is stale and must not clobber the
  // newer click's "ok"/"error" label. Clipboard reads/writes are normally
  // fast (and we already cancel the revert timer on a new click), so the
  // window is small, but the cost of the guard is trivial.
  const opRef = useRef(0);

  useEffect(
    () => () => {
      mountedRef.current = false;
      if (timerRef.current !== null) {
        window.clearTimeout(timerRef.current);
        timerRef.current = null;
      }
    },
    [],
  );

  async function run(text: string) {
    if (timerRef.current !== null) {
      window.clearTimeout(timerRef.current);
      timerRef.current = null;
    }
    const op = ++opRef.current;
    setState("idle");
    const ok = await copyText(text);
    if (!mountedRef.current || op !== opRef.current) return;
    setState(ok ? "ok" : "error");
    timerRef.current = window.setTimeout(
      () => {
        if (!mountedRef.current || op !== opRef.current) return;
        setState("idle");
        timerRef.current = null;
      },
      ok ? 1200 : 2000,
    );
  }
  return { state, run };
}
