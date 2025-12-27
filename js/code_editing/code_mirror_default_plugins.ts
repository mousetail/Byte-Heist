import {
  keymap,
  lineNumbers,
  drawSelection,
  dropCursor,
  rectangularSelection,
  highlightActiveLine,
  highlightActiveLineGutter,
} from "@codemirror/view";
import { highlightSelectionMatches, searchKeymap } from "@codemirror/search";
import { lintKeymap } from "@codemirror/lint";
import { indentWithTab, history } from "@codemirror/commands";
import { EditorView, basicSetup, minimalSetup } from "codemirror";
import {
  carriageReturn,
  insertChar,
  insertCharState,
  showUnprintables,
} from "./code_editor_plugins";
import { indentUnit, bracketMatching } from "@codemirror/language";
import { Compartment, Prec } from "@codemirror/state";
import { oneDark } from "@codemirror/theme-one-dark";
import { basicLight } from "@fsegurai/codemirror-theme-basic-light";

export const editorTheme = new Compartment();

export const defaultPlugins: typeof basicSetup = [
  minimalSetup,
  lineNumbers(),
  drawSelection(),
  dropCursor(),
  history(),
  // multiple selections?
  bracketMatching(),

  rectangularSelection(),
  highlightActiveLine(),
  highlightActiveLineGutter(),

  highlightSelectionMatches(),
  keymap.of(searchKeymap),
  keymap.of(lintKeymap),

  keymap.of([indentWithTab]),
  indentUnit.of("\t"),
  editorTheme.of(getTheme()),
  EditorView.lineWrapping,

  insertChar,
  insertCharState,
  Prec.high(showUnprintables), // Increased precedence to override unprintable char printing in basicSetup
  Prec.high(carriageReturn), // Increased precedence to override shift-enter key binding in basicSetup
];

function getTheme() {
  if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
    return oneDark;
  } else {
    return basicLight;
  }
}
