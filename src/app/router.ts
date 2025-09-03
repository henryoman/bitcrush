export type RouteId = "pixelizer" | "filters";

const DEFAULT_ROUTE: RouteId = "pixelizer";

function getRouteFromHash(): RouteId {
  const match = location.hash.match(/^#\/(\w+)/);
  return (match?.[1] as RouteId) || DEFAULT_ROUTE;
}

export function navigate(route: RouteId) {
  const target = `#/${route}`;
  if (location.hash !== target) location.hash = target;
}

export function mountRoutes() {
  const routeSections = Array.from(
    document.querySelectorAll<HTMLElement>('[data-route]')
  );

  const update = () => {
    const current = getRouteFromHash();
    for (const el of routeSections) {
      const id = el.dataset.route as RouteId | undefined;
      el.style.display = id === current ? "" : "none";
    }
  };

  window.addEventListener("hashchange", update);
  update();
}


