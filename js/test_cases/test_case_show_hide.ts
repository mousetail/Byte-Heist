export function initTestCaseHideShow() {
  document
    .querySelectorAll(".test-case")
    .forEach((testCaseElement: HTMLDivElement) => {
      const hidden = testCaseElement.classList.contains("default-hidden");

      addShowHideListenersToTextCase(testCaseElement, !hidden);
    });
}

export function addShowHideListenersToTextCase(
  testCaseElement: HTMLDivElement,
  defaultVisible: boolean
) {
  testCaseElement.classList.toggle("test-case-hidden", !defaultVisible);

  const header = testCaseElement.querySelector(
    ".test-case-header"
  ) as HTMLDivElement;
  header.style.cursor = "pointer";
  header.addEventListener("click", () => {
    testCaseElement.classList.toggle("test-case-hidden");
  });
}
