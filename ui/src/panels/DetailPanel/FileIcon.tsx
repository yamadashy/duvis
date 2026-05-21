// All callers display this icon alongside the entry's name text, so it
// adds no accessibility info on its own — mark it presentational with
// aria-hidden to keep screen readers from announcing it twice.
export function FileIcon({ isDir }: { isDir: boolean }) {
  return isDir ? (
    <svg viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.5" aria-hidden="true">
      <path d="M1.5 3.5a1 1 0 0 1 1-1h2l1 1h4a1 1 0 0 1 1 1V9a1 1 0 0 1-1 1h-7a1 1 0 0 1-1-1z" />
    </svg>
  ) : (
    <svg viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.5" aria-hidden="true">
      <path d="M3 1.5h4l3 3V10a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V2.5a1 1 0 0 1 1-1z" />
    </svg>
  );
}
