import { Link } from "@tanstack/react-router";

export function MainSidebar() {
  return (
    <aside
      className="
        w-60 h-full flex flex-col
        bg-sidebar text-sidebar-foreground
        border-r border-sidebar-border
      "
    >
      <div className="h-14 flex items-center px-4 border-b border-sidebar-border">
        <span className="font-mono tracking-tight text-sm">Glint</span>
      </div>

      <nav className="flex flex-col p-2 text-sm flex-1">
        <Section title="Overview">
          <NavItem to="/" label="Dashboard" />
        </Section>

        <Section title="Observability">
          <NavItem to="/services" label="Services" />
          <NavItem to="/traces" label="Traces" />
          <NavItem to="/metrics" label="Metrics" />
          <NavItem to="/logs" label="Logs" />
        </Section>

        <Section title="System">
          <NavItem to="/settings" label="Settings" />
        </Section>
      </nav>
    </aside>
  );
}

function Section({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div className="mb-4">
      <h4 className="px-2 mb-1 text-xs uppercase tracking-wider text-sidebar-foreground/50">
        {title}
      </h4>
      <div className="flex flex-col">{children}</div>
    </div>
  );
}

function NavItem({ to, label }: { to: string; label: string }) {
  return (
    <Link
      to={to}
      className="
        px-2 py-1 flex items-center
        hover:bg-sidebar-accent hover:text-sidebar-accent-foreground
        transition-colors
      "
      activeProps={{
        className: "bg-sidebar-primary text-sidebar-primary-foreground",
      }}
    >
      {label}
    </Link>
  );
}
