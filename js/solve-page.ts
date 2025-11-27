import { EditorView } from "codemirror";
import { renderResultDisplay, ResultDisplay } from "./test_cases/test_case";
import { initTestCaseHideShow } from "./test_cases/test_case_show_hide";
import {
  createCodemirrorFromTextAreas,
  lengthInBytes,
  onByteCountChange,
} from "./code_editing/code_editor";

type ScoreInfo = {
  rank: number;
  points: number;
  score: number;
};

type Toast = {
  old_scores: ScoreInfo | undefined;
  new_scores: ScoreInfo;
};

function displayToast(
  toast: Toast | undefined,
  status_code: number,
  account_id: number | undefined
) {
  let category: "success" | "warning" | "info" | "error" = "success";
  let title: string;
  let description: string;
  let action:
    | {
        label: string;
        onclick: string;
      }
    | undefined = undefined;
  if (status_code == 400) {
    category = "error";
    title = "Invalid Solution";
    description = "At least one test failed, see output window for details";
  } else if (!account_id) {
    category = "warning";
    title = "Passed";
    description = "Log in to save your score";
    action = {
      label: "Create Account",
      onclick: 'window.open("/login/github", "_blank")'.replace(/"/g, "&quot;"),
    };
  } else if (status_code == 200 || !toast) {
    category = "info";
    title = "Passed";
    description = "Solution passed but is worse than previous best score";
  } else if (!toast.old_scores) {
    title = `${toast.new_scores.points} bytes (#${toast.new_scores.rank})`;
    description = `Earned ${toast.new_scores.score} score (#${toast.new_scores.rank} rank, ${toast.new_scores.points} points)`;
  } else if (toast.new_scores.rank < toast.old_scores.rank) {
    title = `Saved ${
      toast.old_scores.points - toast.new_scores.points
    } bytes (Rank #${toast.old_scores.rank} -> #${toast.new_scores.rank})`;
    description = `+${
      toast.new_scores.score - toast.old_scores.score
    } score (#${toast.new_scores.rank}, ${toast.new_scores.points} points)`;
  } else if (toast.new_scores.points < toast.old_scores.points) {
    title = `Saved ${toast.old_scores.points - toast.new_scores.points} bytes`;
    description = `+${
      toast.new_scores.score - toast.old_scores.score
    } score (#${toast.new_scores.rank}, ${toast.new_scores.points} points)`;
  } else {
    category = "info";
    title = "Score matched";
    description = "Score equal to your previous best score";
  }

  document.dispatchEvent(
    new CustomEvent("basecoat:toast", {
      detail: {
        config: {
          category: category,
          title: title,
          description: description,
          cancel: {
            label: "Dismiss",
          },
          action,
        },
      },
    })
  );
}

/// Only works from the solutions page
async function submitNewSolution(
  mainTextArea: EditorView,
  submitButton: HTMLButtonElement,
  setOriginalText: (e: string) => void,
  localStorageId: string
) {
  submitButton.disabled = true;
  try {
    const content = mainTextArea.state.doc.toString();

    localStorage.setItem(
      localStorageId,
      JSON.stringify({
        content,
        submittedDate: new Date().valueOf(),
      })
    );

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

    const { tests, leaderboard, toast, account_id } =
      (await response.json()) as {
        tests: ResultDisplay;
        leaderboard: LeaderboardEntry[];
        toast?: Toast | undefined;
        account_id?: number | undefined;
      };
    updateLeaderboard(leaderboard);

    displayToast(toast, response.status, account_id);

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
  localStorageId: string,
  setOriginalText: (e: string) => void
) {
  const form = document.querySelector("form.challenge-submission-form");
  const submitButton = form.querySelector(
    "button[type='submit']"
  ) as HTMLButtonElement;

  form.addEventListener("submit", (ev) => {
    ev.preventDefault();

    submitNewSolution(
      mainTextArea,
      submitButton,
      setOriginalText,
      localStorageId
    );
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

      updateLeaderboard(await response.json());
    });
  });
}

type LeaderboardEntry = {
  rank: number;
  author_avatar: string;
  author_name: string;
  author_id: number;
  points: number;
};

function updateLeaderboard(ranking: LeaderboardEntry[]) {
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

      const pointsCell = document.createElement("td");
      pointsCell.textContent = `${entry.points}`;
      row.appendChild(pointsCell);

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

globalThis.addEventListener("load", async () => {
  const mainCodeTextArea = document.getElementById("main-code");
  const originalCode = decodeURIComponent(
    mainCodeTextArea.dataset.encodedSource
  );
  const originalLocalStorageId = mainCodeTextArea.dataset.localStorageId;

  let leaderboardForm: HTMLFormElement | undefined;
  if ((leaderboardForm = document.querySelector(".leaderboard-tabs-form"))) {
    setupLeaderboardForm(leaderboardForm);
  }

  const mainTextArea = createCodemirrorFromTextAreas()["main-code"];
  initTestCaseHideShow();

  const editorControls = document.getElementById("editor-controls");
  if (editorControls !== null) {
    setupEditorControls(
      editorControls,
      mainTextArea!,
      originalCode,
      originalLocalStorageId
    );
  }
});

const setupEditorControls = (
  editorControls: HTMLElement,
  mainTextArea: EditorView,
  originalCode: string,
  localStorageId: string
) => {
  editorControls.classList.remove("hidden");

  const byteCountElement = editorControls.querySelector("#byte-counter")!;
  const resetButton = editorControls.querySelector<HTMLButtonElement>(
    "#restore-solution-button"
  )!;
  let currentCode = mainTextArea.state.doc.toString();

  byteCountElement.textContent = lengthInBytes(currentCode).toString();

  onByteCountChange(mainTextArea, (byteCount) => {
    byteCountElement.textContent = byteCount.toString();
  });

  if (currentCode === "") {
    resetButton.style.display = "none";
  }

  resetButton.addEventListener("click", () => {
    mainTextArea.dispatch({
      changes: {
        from: 0,
        to: mainTextArea.state.doc.length,
        insert: originalCode,
      },
    });
  });

  setupJsSubmitOnForm(mainTextArea!, localStorageId, (e) => {
    resetButton.style.display = "block";
    currentCode = e;
  });
};
