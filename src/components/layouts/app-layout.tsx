import "./app.css";
import { ParentComponent } from "solid-js";

export const AppLayout: ParentComponent = (props) => {
  return <>{props?.children}</>;
};
