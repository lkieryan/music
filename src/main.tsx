import "./wdyr"
import "@radix-ui/themes/styles.css"
import "./styles/main.css"
// Ensure global CSS load order is deterministic

import * as React from "react"
import ReactDOM from "react-dom/client"
import { RouterProvider } from "react-router/dom"

import { router } from "./router"
import { initializeApp } from "./initialize"

initializeApp()

const $container = document.querySelector("#root") as HTMLElement

ReactDOM.createRoot($container).render(
  <React.StrictMode>
    <RouterProvider router={router} />
  </React.StrictMode>,
)