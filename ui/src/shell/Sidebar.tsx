import type { ReactNode } from "react";
import styles from "./Sidebar.module.css";

interface SidebarProps {
  children: ReactNode;
}

export function Sidebar({ children }: SidebarProps) {
  return (
    <aside className={styles.sidebar} aria-label="Filters and sort">
      {children}
    </aside>
  );
}

interface SectionProps {
  title: string;
  action?: { label: string; onClick: () => void };
  children: ReactNode;
}

export function SidebarSection({ title, action, children }: SectionProps) {
  return (
    <div className={styles.sideSection}>
      <div className={styles.sideTitle}>
        <span>{title}</span>
        {action ? (
          <button type="button" className={styles.sideTitleAction} onClick={action.onClick}>
            {action.label}
          </button>
        ) : null}
      </div>
      {children}
    </div>
  );
}
