/* @refresh reload */
import { render } from "solid-js/web";
import { Route, Router } from "@solidjs/router";
import { HomeRoute } from "./routes/home";
import { AppLayout } from "./components/layouts/app-layout";

const wrapper = document.getElementById("root");

if (!wrapper) {
  throw new Error("Wrapper div not found");
}

render(
  () => (
    <Router>
      <Route path="/" component={AppLayout}>
        <Route path="/" component={HomeRoute} />
      </Route>
    </Router>
  ),
  wrapper,
);
