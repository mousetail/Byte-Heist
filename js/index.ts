import { EditorView } from "codemirror";
import "./style.css";
import { HeistsFilter } from "./heists-filtering";
import "basecoat-css/all";

(() => {
  const apply = () => {
    const prefersDark = matchMedia("(prefers-color-scheme: dark)").matches;
    document.documentElement.classList.toggle("dark", prefersDark);
  };

  apply();

  matchMedia("(prefers-color-scheme: dark)").addEventListener("change", apply);
})();

// Initialize heists filtering when the DOM is loaded
document.addEventListener("DOMContentLoaded", () => {
  new HeistsFilter();
});
