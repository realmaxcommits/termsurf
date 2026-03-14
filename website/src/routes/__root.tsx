import { createRootRoute, Outlet, useRouterState } from "@tanstack/react-router";
import { Header } from "../components/Header";
import { Footer } from "../components/Footer";

export const Route = createRootRoute({
  component: RootComponent,
});

function RootComponent() {
  const { location } = useRouterState();
  const isWelcome = location.pathname === "/welcome";

  if (isWelcome) {
    return <Outlet />;
  }

  return (
    <div className="max-w-3xl mx-auto px-4 py-6">
      <Header />
      <main>
        <Outlet />
      </main>
      <Footer />
    </div>
  );
}
