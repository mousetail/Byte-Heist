import { EditorView } from "codemirror";
import { renderResultDisplay, ResultDisplay } from "./test_cases/test_case";
import { initTestCaseHideShow } from "./test_cases/test_case_show_hide";
import {
  createCodemirrorFromTextAreas,
  lengthInBytes,
  onByteCountChange,
} from "./code_editing/code_editor";

/// Only works from the solutions page
async function submitNewSolution(
  mainTextArea: EditorView,
  submitButton: HTMLButtonElement,
  setOriginalText: (e: string) => void
) {
  submitButton.disabled = true;
  try {
    const content = mainTextArea.state.doc.toString();

    const response = await fetch(window.location.href, {
      method: "POST",
      headers: {
        accept: "application/json",
        "content-type": "application/json",
      },
      body: JSON.stringify({
        code: content,
      }),
    });

    const errorDiv = document.querySelector(".solution-submit-error");
    if (![200, 201, 400].includes(response.status)) {
      errorDiv.textContent = await response.text();
      errorDiv.classList.remove("hidden");
      return;
    }
    errorDiv.classList.add("hidden");

    const { tests, leaderboard } = (await response.json()) as {
      tests: ResultDisplay;
      leaderboard: LeaderboardEntry[];
    };
    updateLeaderbaord(leaderboard);

    if (tests.passed && response.status === 201) {
      setOriginalText(content);
    }
    const testsContainer = document.querySelector(
      "div.result-display-wrapper"
    ) as HTMLDivElement;
    renderResultDisplay(tests, testsContainer);
  } finally {
    submitButton.disabled = false;
  }
}

function setupJsSubmitOnForm(
  mainTextArea: EditorView,
  setOriginalText: (e: string) => void
) {
  const form = document.querySelector("form.challenge-submission-form");
  const submitButton = form.querySelector(
    "button[type='submit']"
  ) as HTMLButtonElement;

  form.addEventListener("submit", (ev) => {
    ev.preventDefault();

    submitNewSolution(mainTextArea, submitButton, setOriginalText);
  });
}

function setupLeaderboardForm(form: HTMLFormElement) {
  form.addEventListener("submit", (ev) => {
    ev.preventDefault();
  });

  form.querySelectorAll("button").forEach((button) => {
    const languageName = window.location.pathname.split("/").pop();

    button.addEventListener("click", async () => {
      changeActiveLeaderboardTab(button.value);

      const response = await fetch(
        `../leaderboard/${languageName}?ranking=${encodeURIComponent(
          button.value
        )}`,
        {
          headers: {
            accept: "application/json",
            "content-type": "application/json",
          },
        }
      );

      if (!response.ok) {
        console.error(await response.json());
      }

      updateLeaderbaord(await response.json());
    });
  });
}

type LeaderboardEntry = {
  rank: number;
  author_avatar: string;
  author_name: string;
  author_id: number;
  score: number;
};

function updateLeaderbaord(ranking: LeaderboardEntry[]) {
  const leaderboard = document.querySelector(".leaderboard table tbody");

  leaderboard.replaceChildren(
    ...ranking.map((entry: LeaderboardEntry) => {
      const row = document.createElement("tr");

      const rankCell = document.createElement("td");
      rankCell.textContent = `#${entry.rank}`;
      row.appendChild(rankCell);

      const avatarCell = document.createElement("td");
      const pfp = document.createElement("img");
      pfp.src = entry.author_avatar;
      avatarCell.appendChild(pfp);
      row.appendChild(avatarCell);

      const authorNameCell = document.createElement("td");
      const link = document.createElement("a");
      link.href = `/user/${entry.author_id}`;
      link.textContent = entry.author_name;
      authorNameCell.appendChild(link);
      row.appendChild(authorNameCell);

      const scoreCell = document.createElement("td");
      scoreCell.textContent = `${entry.score}`;
      row.appendChild(scoreCell);

      return row;
    })
  );
}

function changeActiveLeaderboardTab(tab: string) {
  document.querySelector(
    `.leaderboard-tabs-form button[value=${tab}]`
  ).ariaSelected = "true";
  document.querySelector(
    `.leaderboard-tabs-form button:not([value=${tab}])`
  ).ariaSelected = "false";
}

window.addEventListener("load", async () => {
  let leaderboardForm: HTMLFormElement | undefined;
  if ((leaderboardForm = document.querySelector(".leaderboard-tabs-form"))) {
    setupLeaderboardForm(leaderboardForm);
  }

  const mainTextArea = createCodemirrorFromTextAreas()["main-code"];
  initTestCaseHideShow();

  let editorControls = document.getElementById("editor-controls");
  if (editorControls !== null) {
    setupEditorControls(editorControls, mainTextArea!);
  }
});

const setupEditorControls = (
  editorControls: HTMLElement,
  mainTextArea: EditorView
) => {
  editorControls.classList.remove("hidden");

  const byteCountElement = editorControls.querySelector("#byte-counter")!;
  const resetButton = editorControls.querySelector<HTMLButtonElement>(
    "#restore-solution-button"
  )!;
  let originalText = mainTextArea.state.doc.toString();

  byteCountElement.textContent = lengthInBytes(originalText).toString();

  onByteCountChange(mainTextArea, (byteCount) => {
    byteCountElement.textContent = byteCount.toString();
  });

  if (originalText === "") {
    resetButton.style.display = "none";
  }

  resetButton.addEventListener("click", () => {
    mainTextArea.dispatch({
      changes: {
        from: 0,
        to: mainTextArea.state.doc.length,
        insert: originalText,
      },
    });
  });

  setupJsSubmitOnForm(mainTextArea!, (e) => {
    resetButton.style.display = "block";
    originalText = e;
  });
};
