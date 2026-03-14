import { $icon } from "../util/icons";

export function Header() {
  return (
    <header className="mb-8 pb-4">
      <div className="flex items-center gap-3 mb-2">
        <img
          src={$icon("/images/termsurf-11-transparent-192.png")}
          alt="TermSurf logo"
          className="w-10 h-10"
        />
        <h1 className="text-2xl font-bold text-primary">TermSurf</h1>
        <span className="text-foreground-dark text-sm">Terminal + Browser</span>
      </div>
      <nav className="text-sm">
        <a
          href="https://github.com/termsurf/termsurf"
          target="_blank"
          rel="noopener noreferrer"
          className="text-accent hover:text-primary"
        >
          [GitHub]
        </a>
      </nav>
      <div className="mt-4 text-muted text-xs">
        ────────────────────────────────────────────────────────────────────
      </div>
    </header>
  );
}
