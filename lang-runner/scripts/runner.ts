import {
  Context,
  FinalVerdict,
  RunCodeResult,
  RunCompiledCodeResult,
  TestCase,
} from "./runner-lib.ts";
import { readLines } from "./readline.ts";

type Lang = {
  name: string;
  compileCommand: string[];
  runCommand: string[];
  env: [string, string][];
  installEnv: [string, string][];
  plugin: string;
  latestVersion: string;
  extension: string;
};

type Input = {
  code: string;
  lang: Lang;
  judge: string;
  max_code_size: number;
  max_input_size: number;
};

const { code, judge, max_code_size, max_input_size }: Input = JSON.parse(
  Deno.args[0]
);

(async () => {
  const lines = readLines(Deno.stdin.readable);
  const textEncoder = new TextEncoder();

  const judge_function = (
    await import(
      "data:text/typescript," +
        encodeURIComponent(
          Deno.readTextFileSync("./scripts/runner-lib.ts") +
            "\nexport default " +
            judge
        )
    )
  ).default as (
    code: Context
  ) => AsyncGenerator<TestCase, FinalVerdict, undefined>;

  const onRunCallback = async (
    program: string,
    input?: string | undefined
  ): Promise<RunCompiledCodeResult> => {
    if (program.length > max_code_size) {
      throw new Error(
        `Expected code to have at most ${max_code_size} characters`
      );
    }

    if (input && input.length > max_input_size) {
      throw new Error(
        `Expected input to be at most ${max_input_size} characters`
      );
    }

    await Deno.stdout.write(
      textEncoder.encode(JSON.stringify({ code: program, input }) + "\n")
    );

    const result = await lines.next();
    if (result.done) {
      throw new Error(`Pipe closed after running lang`);
    }

    if (result.value === "CodeTooLarge") {
      throw new Error(`Expected code to have at most ${max_code_size} bytes`);
    }

    if (result.value === "InputTooLarge") {
      throw new Error(`Expected input to have at most ${max_input_size} bytes`);
    }

    return result.value as RunCompiledCodeResult;
  };

  const generator = judge_function(new Context(code, onRunCallback));

  let value: IteratorResult<TestCase, FinalVerdict>;
  while (!(value = await generator.next()).done) {
    console.log(JSON.stringify(value.value));
  }
  console.log(JSON.stringify(value.value));

  Deno.exit(0);
})();
