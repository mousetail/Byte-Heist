import { stat } from "node:fs";
import { addShowHideListenersToTextCase } from "./test_case_show_hide";

export type ResultDisplay = {
  judgeError: null | string;
  passed: boolean;
  tests: Test[];
  timedOut: boolean;
};

type Test = {
  columns: Column[];
  status: string;
  title: string | null;
  defaultVisible: boolean;
};

type DiffCell = {
  tag: "delete" | "insert" | "equal";
  content: string;
};

type Column = {
  content: DiffCell[];
  title: string | null;
};

export function renderResultDisplay(
  display: ResultDisplay,
  parent: HTMLDivElement
) {
  const resultPassStateDiv = parent.querySelector(".result-pass-state");
  const timeOutWarningDiv = parent.querySelector(".time-out-warning");
  const judgeErrorsDiv = parent.querySelector(".judge-errors");
  const testCasesDiv = parent.querySelector(".test-cases");

  resultPassStateDiv.textContent = display.passed ? "Pass" : "Fail";

  timeOutWarningDiv.classList.toggle("hidden", !display.timedOut);

  judgeErrorsDiv.classList.toggle("hidden", display.judgeError === null);
  if (display.judgeError !== null) {
    judgeErrorsDiv.querySelector("pre").textContent = display.judgeError;
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

  const columns = document.createElement("div");
  columns.classList.add(
    "test-case-columns",
    `test-case-${testCase.columns.length}-columns`
  );
  for (let column of testCase.columns) {
    columns.appendChild(renderColumn(column));
  }
  body.appendChild(columns);

  addShowHideListenersToTextCase(root, testCase.defaultVisible);

  return root;
}

function renderColumn(column: Column): HTMLDivElement {
  const columnDiv = document.createElement("div");
  columnDiv.classList.add("test-case-column");

  if (column.title) {
    let title = document.createElement("h3");
    title.textContent = column.title;
    columnDiv.appendChild(title);
  }

  const pre = document.createElement("pre");
  pre.classList.add("code-pre");
  pre.replaceChildren(...column.content.map(renderDiffCell));
  columnDiv.appendChild(pre);

  return columnDiv;
}

function renderDiffCell(cell: DiffCell): HTMLSpanElement {
  let span = document.createElement("span");
  span.classList.add(`diff-tag-${cell.tag}`);
  span.textContent = cell.content;
  return span;
}
