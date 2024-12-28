export function initTestCaseHideShow() {
  document.querySelectorAll(".test-case").forEach((testCaseElement) => {
    const hidden = testCaseElement.classList.contains("default-hidden");
    testCaseElement.classList.toggle("test-case-hidden", hidden);

    const header = testCaseElement.querySelector(
      ".test-case-header"
    ) as HTMLDivElement;
    header.style.cursor = "pointer";
    header.addEventListener("click", () => {
      testCaseElement.classList.toggle("test-case-hidden");
    });
  });
}
