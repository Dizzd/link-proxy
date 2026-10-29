import { NavLink, Outlet } from "react-router-dom";
const links = [
  ["/", "Overview"],
  ["/mappings", "Mappings"],
  ["/upstreams", "Upstreams"],
  ["/domains", "Domains"],
  ["/imports", "Bulk import"],
  ["/exports", "Bulk export"],
  ["/revisions", "Revisions"],
  ["/monitoring", "Monitoring"],
  ["/settings", "Settings"],
];
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
          {links.map(([path, label]) => (
            <NavLink key={path} end={path === "/"} to={path}>
              {label}
            </NavLink>
          ))}
        </nav>
        <div className="sidebar-foot">
          <span className="badge">Memory snapshots</span>
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
