import { useEffect } from "react";

/** Drill out one level when the user presses Esc or Backspace, unless
 *  focus is in an `<input>` (search box typing must not steal the key).
 *  No-op when `rootPath` is already empty. */
export function useDrillOutKey(rootPath: readonly string[], drillOut: () => void): void {
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      // Skip when focus is on an editable element so the user typing in
      // the search box or any future textarea / contentEditable region
      // doesn't have Backspace yanked out from under them.
      const target = e.target as HTMLElement | null;
      if (
        target?.tagName === "INPUT" ||
        target?.tagName === "TEXTAREA" ||
        target?.isContentEditable
      ) {
        return;
      }
      if ((e.key === "Escape" || e.key === "Backspace") && rootPath.length > 0) {
        // preventDefault is mainly here for Backspace — older browsers
        // mapped it to history.back() and some embedded WebViews still
        // do. Escape is harmless to swallow.
        e.preventDefault();
        drillOut();
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [rootPath, drillOut]);
}
