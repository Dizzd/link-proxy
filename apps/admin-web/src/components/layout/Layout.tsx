import { NavLink, Outlet, useLocation } from "react-router-dom";
import { useState } from "react";
import { CacheStatusBadge } from "./CacheStatusBadge";
export const navigationGroups = [
  {
    label: "Routing",
    links: [
      ["/mappings", "Mappings"],
      ["/upstreams", "Upstreams"],
      ["/domains", "Domains"],
    ],
  },
  {
    label: "Data management",
    links: [
      ["/imports", "Bulk import"],
      ["/exports", "Bulk export"],
      ["/revisions", "Revisions"],
    ],
  },
  {
    label: "Operations",
    links: [
      ["/monitoring", "Monitoring"],
      ["/settings", "Settings"],
    ],
  },
];
function NavigationGroup({ label, links }: (typeof navigationGroups)[number]) {
  const { pathname } = useLocation();
  const active = links.some(
    ([path]) => pathname === path || pathname.startsWith(`${path}/`),
  );
  const [expanded, setExpanded] = useState(true);
  const id = `nav-${label.toLowerCase().replaceAll(" ", "-")}`;
  return (
    <section className="nav-group" aria-label={label}>
      <button
        className="nav-group-toggle"
        aria-expanded={expanded}
        aria-controls={id}
        onClick={() => setExpanded(!expanded)}
      >
        <span>
          {label}
          {!expanded && active && (
            <span className="nav-current"> · Current</span>
          )}
        </span>
        <span aria-hidden="true">{expanded ? "−" : "+"}</span>
      </button>
      <div id={id} className="nav-group-links" hidden={!expanded}>
        {links.map(([path, title]) => (
          <NavLink key={path} to={path}>
            {title}
          </NavLink>
        ))}
      </div>
    </section>
  );
}
export function Layout({
  logout,
  username,
  signingOut,
}: {
  logout: () => void;
  username: string;
  signingOut: boolean;
}) {
  return (
    <div className="shell">
      <a className="skip" href="#content">
        Skip to content
      </a>
      <aside>
        <div className="brand">
          <span className="mark" aria-hidden="true">
            ↗
          </span>
          <div>
            Dynamic URL<small>PROXY CONTROL PLANE</small>
          </div>
        </div>
        <nav aria-label="Main navigation">
          <NavLink end to="/">
            Overview
          </NavLink>
          {navigationGroups.map((group) => (
            <NavigationGroup key={group.label} {...group} />
          ))}
        </nav>
        <div className="sidebar-foot">
          <CacheStatusBadge />
          <p>Signed in as {username}</p>
          <button className="quiet" disabled={signingOut} onClick={logout}>
            Sign out
          </button>
        </div>
      </aside>
      <div className="workspace">
        <header>
          <span>Routing infrastructure</span>
          <span>Authenticated administration</span>
        </header>
        <main id="content">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
