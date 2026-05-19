import { type ReactNode, useRef, useState } from "react";
import { createPortal } from "react-dom";

/** Hover/focus tooltip rendered through a portal at document.body so it
 *  can't be clipped by the detail panel's overflow, and positioned via
 *  fixed coords from the wrapper's bounding rect so it lifts above any
 *  treemap stacking context. (`title` doesn't render reliably on
 *  disabled buttons in Chrome/Safari, which is why we don't use it.)
 *  Shared by TrashButton (explains why it's disabled) and CopyJsonButton
 *  (previews the field list before clicking). */
export function HintWrap({ children, tip }: { children: ReactNode; tip: ReactNode }) {
  const wrapRef = useRef<HTMLSpanElement>(null);
  const [anchor, setAnchor] = useState<{ cx: number; top: number } | null>(null);

  function show() {
    const el = wrapRef.current;
    if (!el) return;
    const r = el.getBoundingClientRect();
    setAnchor({ cx: r.left + r.width / 2, top: r.top });
  }
  function hide() {
    setAnchor(null);
  }

  return (
    <span
      ref={wrapRef}
      className="hint-wrap"
      onMouseEnter={show}
      onMouseLeave={hide}
      onFocus={show}
      onBlur={hide}
    >
      {children}
      {anchor
        ? createPortal(
            <div
              className="hint-tip"
              role="tooltip"
              style={{ left: anchor.cx, top: anchor.top - 8 }}
            >
              {tip}
            </div>,
            document.body,
          )
        : null}
    </span>
  );
}
