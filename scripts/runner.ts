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
};

const { code, judge }: Input = JSON.parse(Deno.args[0]);

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
    await Deno.stdout.write(
      textEncoder.encode(JSON.stringify({ code: program, input }) + "\n")
    );

    const result = await lines.next();
    if (result.done) {
      throw new Error(`Pipe closed after running lang`);
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
