import { Outlet } from "react-router"
import { RootProvider } from "./providers/root"

function App() {
  return (
    <RootProvider>
      <Outlet />
    </RootProvider>
  )
}

export { App as Component }
