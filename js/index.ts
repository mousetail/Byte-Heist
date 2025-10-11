import { EditorView } from "codemirror";
import "./style.css";
import { HeistsFilter } from "./heists-filtering";
import "basecoat-css/all";

// Initialize heists filtering when the DOM is loaded
document.addEventListener("DOMContentLoaded", () => {
  new HeistsFilter();
});
