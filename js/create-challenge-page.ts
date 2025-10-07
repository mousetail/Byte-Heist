import { EditorView } from "codemirror";
import { createCodemirrorFromTextAreas } from "./code_editing/code_editor";
import { TestCase } from "../scripts/runner-lib";
import { ResultDisplay, renderResultDisplay } from "./test_cases/test_case";

window.addEventListener("DOMContentLoaded", () => {
  const textAreas = createCodemirrorFromTextAreas();

  const submit_challenge_form = document.querySelector<HTMLFormElement>(
    "form#submit-challenge-form"
  );
  setup_form(submit_challenge_form, textAreas);
});

const setup_form = (
  form: HTMLFormElement,
  textAreas: { [name: string]: EditorView }
) => {
  const { "challenge-judge": judgeTextArea, "example-code": exampleTextArea } =
    textAreas;

  const submitButtons = form.querySelectorAll<HTMLButtonElement>(
    'button[type="submit"]'
  );

  form.addEventListener("submit", async (ev) => {
    ev.preventDefault();

    const formData = new FormData(form, ev.submitter);

    const jsonData = {};

    for (let [key, value] of formData.entries()) {
      jsonData[key] = value;
    }

    jsonData["example_code"] = exampleTextArea.state.doc.toString();
    jsonData["judge"] = judgeTextArea.state.doc.toString();

    submitButtons.forEach((i) => (i.disabled = true));

    let response: Response;
    try {
      response = await fetch(form.action ?? window.location.href, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Accept: "application/json",
        },
        body: JSON.stringify(jsonData),
      });
    } catch (e) {
      const errorDiv = document.querySelector("#validation-error-status");
      errorDiv.classList.remove("display-none");
      errorDiv.textContent =
        "Failed to submit due to a network error, see console for details";
      console.error(e);

      submitButtons.forEach((i) => (i.disabled = false));
      return;
    }

    if (response.redirected) {
      console.log("redirected");
      window.history.replaceState("", null, response.url);
    }

    const data = await response.json();

    handleResponse(data, form);

    submitButtons.forEach((i) => (i.disabled = false));
  });
};

type submitChallengeResponse = {
  validation?: { [formField: string]: string } | undefined;
  tests?: ResultDisplay;
  description: string;
  judge: string;
  name: string;
  example_code: string;
  catgory: string;
  status: string;
  author_name?: string | undefined;
  author_avatar?: string | undefined;
  author_id?: number | undefined;
  is_post_mortem?: number | undefined;
  post_mortem_date: number[] | undefined;
  id?: number | undefined;
};

const handleResponse = (
  data: submitChallengeResponse,
  form: HTMLFormElement
) => {
  const validation = data.validation;
  console.log(validation);
  const validationFields =
    form.querySelectorAll<HTMLDivElement>(".validation-error");
  console.log(validationFields);

  validationFields.forEach((element) => {
    if (validation && Object.hasOwn(validation, element.dataset.fieldName)) {
      element.classList.remove("display-none");
      element.textContent = validation[element.dataset.fieldName];
    } else {
      element.classList.add("display-none");
    }
  });

  const container = document.querySelector<HTMLDivElement>(
    "#test-case-container"
  );
  if (Object.hasOwn(data, "tests") && data.tests) {
    renderResultDisplay(data.tests, container);
  } else {
    container.replaceChildren();
  }
};
