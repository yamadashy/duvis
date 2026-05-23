import { Fragment } from "react";
import styles from "./DetailPanel.module.css";

interface BreadcrumbsProps {
  segments: readonly string[];
  /** Called with the new in-view path (excluding the root segment) when the
   *  user clicks any non-last crumb. */
  onNavigateTo: (path: string[]) => void;
}

export function Breadcrumbs({ segments, onNavigateTo }: BreadcrumbsProps) {
  return (
    <nav className={styles.detailCrumbs} aria-label="Path">
      <svg
        className={styles.detailCrumbsIcon}
        viewBox="0 0 12 12"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.5"
        aria-hidden="true"
      >
        <path d="M1.5 3.5a1 1 0 0 1 1-1h2l1 1h4a1 1 0 0 1 1 1V9a1 1 0 0 1-1 1h-7a1 1 0 0 1-1-1z" />
      </svg>
      {segments.map((name, i) => {
        const isLast = i === segments.length - 1;
        return (
          // biome-ignore lint/suspicious/noArrayIndexKey: path segments can repeat (e.g. src/src/foo), index disambiguates
          <Fragment key={`${i}-${name}`}>
            {i > 0 ? (
              <span className={styles.detailCrumbSep} aria-hidden="true">
                /
              </span>
            ) : null}
            {isLast ? (
              <span className={`${styles.detailCrumb} ${styles.last}`}>{name}</span>
            ) : (
              <button
                type="button"
                className={styles.detailCrumb}
                onClick={() => onNavigateTo(segments.slice(1, i + 1))}
              >
                {name}
              </button>
            )}
          </Fragment>
        );
      })}
    </nav>
  );
}
