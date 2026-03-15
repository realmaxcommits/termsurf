import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/manifesto")({
  head: () => ({ meta: [{ title: "Manifesto — TermSurf" }] }),
  component: AboutPage,
});

function AboutPage() {
  return (
    <section>
      <h2 className="text-sm font-bold text-foreground mb-4">┌─ Manifesto ─┐</h2>
      <div className="text-sm space-y-4 text-foreground-dark">
        <p>
          I surf by day. I hack by night. I sleep somewhere in between.
        </p>
        <p>
          I live in the terminal. It is my cockpit. Total control. Every
          process, every socket, every file — all reachable from a single prompt.
          The terminal does not hide things. It does not decide what I am allowed
          to see. I have root. I am the operator.
        </p>
        <p>
          Then there is the browser.
        </p>
        <p>
          Browsers are designed for the lowest common denominator. They hide the
          network. They hide the DOM. They wrap everything in a GUI built for
          people who do not know what a process is. Chrome has three billion
          users. It is optimized for all of them. It is optimized for none of
          them.
        </p>
        <p>
          I am not the lowest common denominator. I need to inspect every
          request. Override every header. Pipe responses into scripts. Open
          DevTools in a split pane while tailing logs in another. I need the web
          the same way I need the filesystem — raw, fast, and under complete
          control.
        </p>
        <p>
          The browser I need does not exist. Not as a standalone app. The app is
          the wrong container. The right container is the terminal — where
          everything else already lives. The browser should be a pane. It should
          sit next to the shell, next to the editor, next to the logs. It should
          resize with a keystroke. It should speak protobuf over a Unix socket.
          It should be a component in the system, not a system unto itself.
        </p>
        <p>
          So I built TermSurf.
        </p>
        <p>
          TermSurf is a protocol for jacking web browsers into terminal
          emulators. Full Chromium. Full GPU rendering. Zero-copy compositing.
          Type{" "}
          <code className="text-accent">web localhost:3000</code> and the page
          is there — right next to your shell. No alt-tab. No context switch. No
          lowest common denominator.
        </p>
        <p>
          When I am not in the water, I am in the terminal. And now the web is
          there too.
        </p>
        <p className="pt-2">
          — Max Commits
        </p>
        <p className="pt-2">
          <a
            href="https://github.com/termsurf/termsurf"
            target="_blank"
            rel="noopener noreferrer"
            className="text-accent hover:text-primary"
          >
            [fork the source]
          </a>
        </p>
      </div>
    </section>
  );
}
