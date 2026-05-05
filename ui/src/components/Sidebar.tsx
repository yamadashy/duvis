import type { ReactNode } from "react";
import "./Sidebar.css";

interface SidebarProps {
  children: ReactNode;
}

export function Sidebar({ children }: SidebarProps) {
  return (
    <aside className="sidebar" aria-label="Filters and sort">
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
    <div className="side-section">
      <div className="side-title">
        <span>{title}</span>
        {action ? (
          <button type="button" className="side-title-action" onClick={action.onClick}>
            {action.label}
          </button>
        ) : null}
      </div>
      {children}
    </div>
  );
}
