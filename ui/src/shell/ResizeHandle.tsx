import { useEffect, useRef } from "react";

interface ResizeHandleProps {
  /** Direction the drag deltas should be applied. */
  onDrag: (deltaPx: number) => void;
}

/** Thin draggable column divider. Tracks horizontal mouse delta and
 *  forwards it to the parent so it can adjust its column width. */
export function ResizeHandle({ onDrag }: ResizeHandleProps) {
  const draggingRef = useRef<{ lastX: number } | null>(null);

  useEffect(() => {
    function onMove(e: MouseEvent) {
      const drag = draggingRef.current;
      if (!drag) return;
      const dx = e.clientX - drag.lastX;
      drag.lastX = e.clientX;
      if (dx !== 0) onDrag(dx);
    }
    function onUp() {
      if (!draggingRef.current) return;
      draggingRef.current = null;
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    }
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
    return () => {
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    };
  }, [onDrag]);

  function nudge(dx: number) {
    if (dx !== 0) onDrag(dx);
  }

  return (
    <div
      className="resize-handle"
      role="separator"
      aria-orientation="vertical"
      aria-label="Resize column"
      // tabIndex makes the separator reachable via keyboard, satisfying
      // useFocusableInteractive. Arrow Left/Right shifts the column by
      // 8px steps so non-mouse users can still adjust the layout.
      tabIndex={0}
      onMouseDown={(e) => {
        e.preventDefault();
        draggingRef.current = { lastX: e.clientX };
        document.body.style.cursor = "col-resize";
        document.body.style.userSelect = "none";
      }}
      onKeyDown={(e) => {
        if (e.key === "ArrowLeft") {
          e.preventDefault();
          nudge(-8);
        } else if (e.key === "ArrowRight") {
          e.preventDefault();
          nudge(8);
        }
      }}
    />
  );
}
