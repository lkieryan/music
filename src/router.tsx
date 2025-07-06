import { buildGlobRoutes } from "~/lib/route-builder"
import { createBrowserRouter } from "react-router"

import { Component as App } from "./App"

const globTree = import.meta.glob("./pages/**/*.tsx")

const tree = buildGlobRoutes(globTree)

const routerCreator = createBrowserRouter

export const router = routerCreator([
  {
    path: "/",
    Component: App,
    children: tree,
  },
  {
    path: "*",
    element: <div>404</div>,
  },
])