export type PassState = "Pass" | "Fail" | "Warning" | "Info";
export type ResultDisplay =
  | {
      Diff: {
        expected: string;
        output: string;
        input?: string | undefined;
        sep?: string | undefined;
      };
    }
  | { Text: string }
  | { Run: { input?: string | undefined; output: string; error: string } };
export type Challenge = AsyncGenerator<TestCase, FinalVerdict, undefined>;

export class TestCase {
  name: string | undefined;
  pass: PassState;
  resultDisplay: ResultDisplay;

  constructor(
    name: string | undefined,
    pass: PassState,
    resultDisplay: ResultDisplay
  ) {
    this.name = name;
    this.pass = pass;
    this.resultDisplay = resultDisplay;
  }

  public setName(name: string): this {
    this.name = name;
    return this;
  }

  public replaceFailState(state: PassState): this {
    if (this.pass === "Fail") {
      this.pass = state;
    }
    return this;
  }
}

export class FinalVerdict {
  pass: boolean;

  constructor(pass: boolean) {
    this.pass = pass;
  }
}

export type RunCodeResult = {
  stdout: string;
  stderr: string;
  exitStatus: number;
};

export interface RunCompiledCodeResult extends RunCodeResult {
  compilationResult: RunCodeResult | undefined;
}

export class StringResult {
  protected context: Context;
  public text: string;
  protected input: string;

  public constructor(
    context: Context,
    text: string,
    input: string | undefined = undefined
  ) {
    this.context = context;
    this.text = text;
    this.input = input;
  }

  public assertEquals(value: string, sep: string = "\n"): TestCase {
    const valid = eqIgnoreTrailingWhitespace(this.text, value);
    const testCase = new TestCase(undefined, valid ? "Pass" : "Fail", {
      Diff: {
        expected: value,
        output: this.text,
        sep,
        input: this.input,
      },
    });
    this.context.testCases.push(testCase);
    return testCase;
  }

  public assert(cb: (k: string) => TestCase): TestCase {
    const vestCase = cb(this.text);
    this.context.testCases.push(vestCase);
    return vestCase;
  }
}

export class RunResult extends StringResult {
  private stderr: string;

  public constructor(
    context: Context,
    result: RunCodeResult,
    input: string | undefined
  ) {
    super(context, result.stdout, input);
    this.stderr = result.stderr;
  }

  public error() {
    return new StringResult(this.context, this.stderr, this.input);
  }
}

function shuffleAndDeal<T>(
  testCases: T[],
  options: { shuffle: boolean; numberOfRuns: number }
): T[][] {
  if (options.shuffle) {
    shuffle(testCases);
  }

  // Ensure the runs are uneven
  // This is mostly to prevent people from hardcoding the length of the input
  const cardsPerHand = testCases.length / (options.numberOfRuns + 1);
  const hands = [testCases.slice(0, Math.ceil(cardsPerHand * 2))];
  for (let i = cardsPerHand * 2; i < testCases.length; i += cardsPerHand) {
    const hand = testCases.slice(Math.ceil(i), Math.ceil(i + cardsPerHand) + 1);
    if (hand.length != 0) {
      hands.push(hand);
    }
  }

  return hands;
}

export type TestCasesOptions = {
  inputSeperator: string;
  outputSeperator: string;
  numberOfRuns: number;
  shuffle: boolean;
};

export type FilterCasesOptions = {
  inputSeperator: string;
  shuffle: boolean;
  numberOfRuns: number;
};

export class Context {
  public code: string;
  private onRunCallback: (
    code: string,
    input: string | undefined
  ) => Promise<RunCompiledCodeResult>;
  public testCases: TestCase[];

  private runs: number = 0;

  constructor(
    code: string,
    onRunCallback: (
      code: string,
      input: string | undefined
    ) => Promise<RunCompiledCodeResult>
  ) {
    this.code = code;
    this.onRunCallback = onRunCallback;
    this.testCases = [];
  }

  /**
   * Helper method for when each input corresponds to a matching output.
   * Automatically shuffles the test cases and divides them over multiple runs.
   * @param testCases A mapping of input -> expected output
   * @param overrideOptions Special options (optional)
   */
  async *runTestCases(
    testCases: [string, string][],
    overrideOptions: Partial<TestCasesOptions> = {}
  ): AsyncIterableIterator<TestCase> {
    const options: TestCasesOptions = {
      inputSeperator: "\n",
      outputSeperator: "\n",
      numberOfRuns: 2,
      shuffle: true,
      ...overrideOptions,
    };

    const hands = shuffleAndDeal(testCases, options);

    // TODO: When running code becomes thread safe, this should run in paralel
    for (const hand of hands) {
      yield (
        await this.run(hand.map((i) => i[0]).join(options.inputSeperator))
      ).assertEquals(hand.map((i) => i[1]).join(options.outputSeperator));
    }
  }

  async *runFilterCases(
    testCases: [string, boolean][],
    overrideOptions: Partial<FilterCasesOptions> = {}
  ): AsyncIterableIterator<TestCase> {
    const options: FilterCasesOptions = {
      inputSeperator: "\n",
      numberOfRuns: 2,
      shuffle: true,
      ...overrideOptions,
    };
    const hands = shuffleAndDeal(testCases, options);

    // TODO: When running code becomes thread safe, this should run in paralel
    for (const hand of hands) {
      yield (
        await this.run(hand.map((i) => i[0]).join(options.inputSeperator))
      ).assertEquals(
        hand
          .filter((i) => i[1])
          .map((i) => i[0])
          .join(options.inputSeperator)
      );
    }
  }

  async run(input?: string | undefined): Promise<RunResult> {
    return this.runCode(this.code, input);
  }

  async runCode(code: string, input?: string | undefined) {
    const result = await this.onRunCallback(code, input);
    let { stderr, stdout, compilationResult } = result;
    const compilationFailed =
      compilationResult && compilationResult.exitStatus !== 0;
    if (compilationFailed) {
      stdout = compilationResult.stdout;
      stderr = compilationResult.stderr;
    }
    const runDisplay: TestCase = new TestCase(
      `Run #${++this.runs}`,
      compilationFailed ? "Warning" : "Info",
      {
        Run: {
          input: input,
          output: stdout,
          error: stderr,
        },
      }
    );
    console.log(JSON.stringify(runDisplay));
    return new RunResult(this, result, input);
  }

  registerTestCase(testCase: TestCase): TestCase {
    this.testCases.push(testCase);
    return testCase;
  }

  noFailures(): FinalVerdict {
    if (this.testCases.every((i) => i.pass !== "Fail")) {
      return new FinalVerdict(true);
    } else {
      return new FinalVerdict(false);
    }
  }
}

export const eqIgnoreTrailingWhitespace = (a: string, b: string): boolean => {
  const [a_stripped, b_stripped] = [a, b].map((text) =>
    text.replace(/\s*(?=\n|$)/gu, "")
  );
  return a_stripped == b_stripped;
};

export function range(a: number, b?: number): number[] {
  return b === undefined
    ? [...Array(a).keys()]
    : range(b - a).map((x) => x + a);
}

export function rand(a: number, b?: number): number {
  return b === undefined ? Math.floor(Math.random() * a) : rand(b - a) + a;
}

export function shuffle(array: unknown[]): void {
  for (let i = array.length - 1; i >= 0; i--) {
    const j = rand(i + 1);
    [array[i], array[j]] = [array[j], array[i]];
  }
}
