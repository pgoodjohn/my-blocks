import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import {
  QueryClient,
  QueryClientProvider,
} from "@tanstack/react-query"
import { createRootRoute, createRoute, Outlet, useParams } from '@tanstack/react-router'
import { TanStackRouterDevtools } from '@tanstack/router-devtools'

import { RouterProvider, createRouter, Link } from '@tanstack/react-router'

const queryClient = new QueryClient()

const rootRoute = createRootRoute({
  component: () => (
    <>
      <Outlet />
      <TanStackRouterDevtools />
    </>
  ),
})

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: function Index() {
    return (
      <App />
    )
  },
})

const pageRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/page/$id',
  component: function PageRoute() {
    const { id } = useParams({ strict: false })

    console.debug("Page Route", id)

    return (
      <App block_id={id} />
    )
  },
})

const routeTree = rootRoute.addChildren([indexRoute, pageRoute])

const router = createRouter({ routeTree })

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  </React.StrictMode >,
);

