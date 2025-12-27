import { addShowHideListenersToTextCase } from "./test_case_show_hide";
import { Challenge } from "../types.ts";

export type ResultDisplay = {
  judgeError: null | string;
  passed: boolean;
  tests: Test[];
  timedOut: boolean;
  points: number | undefined;
};

type Test = {
  columns: Columns;
  status: string;
  title: string | null;
  defaultVisible: boolean;
};

type Columns = {
  height: number;
  fields: Field[];
  column_titles: string[];
};

type Field = {
  column: number;
  span: number;
  row_span: number;
  content: string;
  kind: "identical" | "insert" | "delete" | "meta";
};

function getOrCreateElement(
  parent: HTMLDivElement,
  className: string
): HTMLDivElement {
  const elementsWithClass = parent.querySelector<HTMLDivElement>(
    "." + className
  );
  if (elementsWithClass) {
    return elementsWithClass;
  } else {
    const elem = document.createElement("div");
    parent.appendChild(elem);
    elem.classList.add(className);
    return elem;
  }
}

export function renderResultDisplay(
  display: ResultDisplay,
  parent: HTMLDivElement,
  unit: string
) {
  const resultPassStateDiv = getOrCreateElement(parent, "result-pass-state");
  const timeOutWarningDiv = getOrCreateElement(parent, "time-out-warning");
  const judgeErrorsDiv = getOrCreateElement(parent, "judge-errors");
  if (!judgeErrorsDiv.querySelector("pre")) {
    const pre = document.createElement("pre");
    judgeErrorsDiv.appendChild(pre);
  }
  const testCasesDiv = getOrCreateElement(parent, "test-cases");

  resultPassStateDiv.textContent =
    (display.passed ? "Pass" : "Fail") +
    (display.points ? ` (${display.points} ${unit})` : "");

  timeOutWarningDiv.classList.toggle("hidden", !display.timedOut);

  judgeErrorsDiv.classList.toggle("hidden", display.judgeError === null);
  if (display.judgeError !== null) {
    judgeErrorsDiv.querySelector("pre").textContent = display.judgeError;
  } else {
    judgeErrorsDiv.querySelector("pre").textContent = "";
  }

  testCasesDiv.replaceChildren(...display.tests.map(renderTestCase));
}

function renderTestCase(testCase: Test): HTMLDivElement {
  const root = document.createElement("div");
  root.classList.add("test-case", `test-${testCase.status.toLowerCase()}`);

  const header = document.createElement("div");
  header.classList.add("test-case-header");

  const img = document.createElement("img");
  img.src = "/static/triangle.svg";
  img.width = 32;
  header.appendChild(img);

  const title = document.createElement("h2");
  title.classList.add("test-case-title");
  if (testCase.title) {
    title.textContent = testCase.title;
  }
  header.appendChild(title);

  const status = document.createElement("div");
  status.classList.add("test-case-status");
  status.textContent = testCase.status;
  header.appendChild(status);

  root.appendChild(header);

  const body = document.createElement("div");
  body.classList.add("test-case-content");
  root.appendChild(body);

  body.appendChild(renderHeadings(testCase.columns.column_titles));

  const columns = document.createElement("div");
  columns.classList.add(
    "test-case-columns",
    `test-case-${testCase.columns.column_titles.length}-columns`
  );
  for (let field of testCase.columns.fields) {
    columns.appendChild(renderField(field));
  }
  body.appendChild(columns);

  addShowHideListenersToTextCase(root, testCase.defaultVisible);

  return root;
}

function renderHeadings(headings: string[]) {
  const container = document.createElement("div");
  container.classList.add(
    "test-case-column-headings",
    `test-case-${headings.length}-columns`
  );

  for (const heading of headings) {
    const div = document.createElement("div");
    div.textContent = heading;
    container.appendChild(div);
  }

  return container;
}

function renderField(field: Field): HTMLDivElement {
  const columnDiv = document.createElement("div");
  columnDiv.classList.add("test-case-column", "diff-tag-" + field.kind);
  columnDiv.style.gridColumn = `${field.column + 1} / span ${field.span}`;
  columnDiv.style.gridRow = `span ${field.row_span}`;
  columnDiv.textContent = field.content;

  return columnDiv;
}
