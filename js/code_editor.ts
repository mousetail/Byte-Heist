import { basicSetup, EditorView, minimalSetup } from "codemirror";
import { keymap } from "@codemirror/view";
import { indentWithTab } from "@codemirror/commands";
import { Compartment } from "@codemirror/state";
import { javascript } from "@codemirror/lang-javascript";
import { indentUnit } from "@codemirror/language";
import { autocompletion } from "@codemirror/autocomplete";
import { WorkerShape } from "@valtown/codemirror-ts/worker";
import * as Comlink from "comlink";
import { StateEffect } from "@codemirror/state";
import { oneDark } from "@codemirror/theme-one-dark";
import { basicLight } from "@fsegurai/codemirror-theme-basic-light";

let typeScriptEnvironment: WorkerShape | undefined = undefined;

async function initTypescriptForCodebox(): Promise<typeof minimalSetup> {
  const {
    tsFacetWorker,
    tsSyncWorker,
    tsLinterWorker,
    tsAutocompleteWorker,
    tsHoverWorker,
  } = await import("@valtown/codemirror-ts");

  if (typeScriptEnvironment === undefined) {
    const innerWorker = new Worker(
      new URL("./typescript_worker.ts", import.meta.url),
      {
        type: "module",
      }
    );
    const worker = Comlink.wrap(innerWorker) as WorkerShape;
    await worker.initialize();
    typeScriptEnvironment = worker;
  }
  const path = "/src/index.ts";

  return [
    javascript({
      typescript: true,
      jsx: true,
    }),
    tsFacetWorker.of({ worker: typeScriptEnvironment, path }),
    tsSyncWorker(),
    tsLinterWorker(),
    autocompletion({
      override: [tsAutocompleteWorker()],
    }),
    tsHoverWorker(),
  ];
}

function getTheme() {
  if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
    return oneDark;
  } else {
    return basicLight;
  }
}

function editorFromTextArea(
  textarea: HTMLTextAreaElement,
  extensions: typeof minimalSetup,
  swapOnSubmit: boolean
): EditorView {
  let view = new EditorView({ doc: textarea.value, extensions });
  textarea.parentNode?.insertBefore(view.dom, textarea);
  if (swapOnSubmit) {
    textarea.style.display = "none";
  } else {
    textarea.parentElement.removeChild(textarea);
  }

  if (swapOnSubmit && textarea.form) {
    textarea.form.addEventListener("submit", () => {
      textarea.value = view.state.doc.toString();
    });
  }

  return view;
}

export function createCodemirrorFromTextAreas(): EditorView | undefined {
  let mainTextArea: EditorView | undefined = undefined;
  const editorTheme = new Compartment();

  for (const textarea of document.querySelectorAll<HTMLTextAreaElement>(
    "textarea.codemirror"
  )) {
    let plugins: typeof basicSetup = [
      basicSetup,
      keymap.of([indentWithTab]),
      indentUnit.of("\t"),
      editorTheme.of(getTheme()),
      EditorView.lineWrapping,
    ];
    console.log("Replacing textarea with codemirror");
    let view = editorFromTextArea(
      textarea,
      plugins,
      textarea.id !== "main-code"
    );

    if (textarea.classList.contains("lang-typescript")) {
      initTypescriptForCodebox().then((plugin) => {
        view.dispatch({
          effects: StateEffect.appendConfig.of(plugin),
        });
      });
    }
    if (textarea.id === "main-code") {
      mainTextArea = view;
    }
  }

  return mainTextArea;
}

const textEncoder = new TextEncoder();
export const lengthInBytes = (s: string): number =>
  textEncoder.encode(s).length;

export function onByteCountChange(
  mainTextArea: EditorView,
  e: (byte_count: number) => void
) {
  mainTextArea.dispatch({
    effects: StateEffect.appendConfig.of([
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          e(lengthInBytes(mainTextArea.state.doc.toString()));
        }
      }),
    ]),
  });
}
