import { EditorView } from "codemirror";
import {
  createDefaultEditor,
  defaultPlugins,
} from "./code_editing/code_editor";
import { ResultDisplay, renderResultDisplay } from "./test_cases/test_case";

const setupVotingButton = (button: HTMLButtonElement) => {
  const is_upvote = button.value === "true";
  const comment_id = +button.parentElement.querySelector<HTMLInputElement>(
    "input[name=comment_id]"
  ).value;
  const other_button = button.parentElement.querySelector<HTMLButtonElement>(
    `button[name=is_upvote][value=${!is_upvote}]`
  );

  button.addEventListener("click", async (e) => {
    e.preventDefault();
    button.disabled = true;

    let other_button_was_disabled = false;
    if (other_button.disabled) {
      other_button.disabled = false;
      other_button.textContent = other_button.textContent.replace(
        /\d+/,
        (e) => `${+e - 1}`
      );

      other_button_was_disabled = true;
    }

    button.textContent = button.textContent.replace(/\d+/, (e) => `${+e + 1}`);

    const res = await fetch(button.form.action, {
      method: "POST",
      headers: {
        Accept: "Application/Json",
        "Content-Type": "Application/Json",
      },
      redirect: "manual",
      body: JSON.stringify({ is_upvote: is_upvote, comment_id: comment_id }),
    });

    if (res.status >= 400 && res.status != 418) {
      console.error(`Failed to submit vote: ${res.status}`);
      button.textContent = button.textContent.replace(
        /\d+/,
        (e) => `${+e - 1}`
      );
      button.disabled = false;

      other_button.disabled = other_button_was_disabled;
    }
  });
};

const setupComments = () => {
  const votingButtons = document.querySelectorAll("button[name=is_upvote]");
  votingButtons.forEach(setupVotingButton);
};

const setupEditButtons = () => {
  document
    .querySelectorAll<HTMLButtonElement>("button[data-for]")
    .forEach((button) => {
      let editor: EditorView | undefined;
      const oldComponent = document.getElementById(button.dataset["for"]);
      let errorBox: HTMLDivElement | undefined;
      let testCasesBox: HTMLDivElement | undefined;

      function setErrorMessage(message: string) {
        if (errorBox === undefined) {
          errorBox = document.createElement("div");
          errorBox.classList.add("validation-error");
          button.insertAdjacentElement("beforebegin", errorBox);
        }

        errorBox.textContent = message;
      }

      function renderTestCases(data: ResultDisplay) {
        if (testCasesBox === undefined) {
          testCasesBox = document.createElement("div");
          button.parentElement.insertAdjacentElement("afterend", testCasesBox);
        }

        renderResultDisplay(data, testCasesBox);
        button.disabled = false;
      }

      button.addEventListener("click", () => {
        if (button.dataset.active != "true") {
          oldComponent.style.display = "none";

          editor = createDefaultEditor(button.dataset.value, defaultPlugins);
          oldComponent.insertAdjacentElement("afterend", editor.dom);

          button.textContent = "submit";
          button.dataset.active = "true";
        } else {
          button.disabled = true;
          fetch("", {
            headers: {
              "Content-Type": "Application/Json",
              Accept: "Application/Json",
            },
            body: JSON.stringify({
              diff: {
                field: button.dataset.name,
                replacement_value: editor.state.doc.toString(),
              },
              message: `Suggestion to edit ${button.dataset.name}`,
            }),
            method: "POST",
          })
            .then((res: Response) => {
              // HTTP 409 CONFLICT indicates a diff is pending
              if (res.status == 409) {
                throw new Error(
                  "A change suggestion for this field is already pending"
                );
              } else if (res.redirected) {
                window.location.reload();
              } else if (res.status == 400) {
                res.json().then((i) => renderTestCases(i));
              } else if (!res.ok) {
                throw new Error(res.statusText);
              } else {
                return res.json();
              }
            })
            .catch((e: Error) => {
              setErrorMessage(e.message);
              button.disabled = false;
            });
        }
      });
    });
};

setupComments();
setupEditButtons();
