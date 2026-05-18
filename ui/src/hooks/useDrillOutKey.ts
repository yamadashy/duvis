import { useEffect } from "react";

/** Drill out one level when the user presses Esc or Backspace, unless
 *  focus is in an `<input>` (search box typing must not steal the key).
 *  No-op when `rootPath` is already empty. */
export function useDrillOutKey(rootPath: readonly string[], drillOut: () => void): void {
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if ((e.target as HTMLElement | null)?.tagName === "INPUT") return;
      if ((e.key === "Escape" || e.key === "Backspace") && rootPath.length > 0) {
        drillOut();
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [rootPath, drillOut]);
}
