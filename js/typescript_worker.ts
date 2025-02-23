import {
  createDefaultMapFromCDN,
  createSystem,
  createVirtualTypeScriptEnvironment,
} from "@typescript/vfs";
import ts, { CompilerOptions } from "typescript";
import * as Comlink from "comlink";
import { createWorker } from "@valtown/codemirror-ts/worker";

const fetchDeclarations = async (): Promise<string> => {
  const response = await fetch(
    new URL("/ts/runner-lib.d.ts", globalThis.origin)
  );
  if (!response.ok) {
    throw new Error(`Failed to fetch type declarations: ${response.status}`);
  }

  return (await response.text()).replaceAll("export", "");
};

export default Comlink.expose(
  createWorker(async function () {
    const compilerOpts: CompilerOptions = {
      typeRoots: ["/src/types"],
      lib: ["es2021"],
      target: 99,
    };
    const [declarations, fsMap] = await Promise.all([
      fetchDeclarations(),
      createDefaultMapFromCDN(compilerOpts, "5.6.3", false, ts),
    ]);

    // fsMap.set('/lib.d.ts',
    //     fsMap.get('/lib.d.ts') + '\n' + `/// <reference lib="global" />` +
    //     declarations
    // )
    // console.log(fsMap.get('/lib.d.ts'))
    fsMap.set("/src/types/global.d.ts", declarations);
    const system = createSystem(fsMap);
    return createVirtualTypeScriptEnvironment(
      system,
      ["/src/types/global.d.ts"],
      ts,
      compilerOpts
    );
  })
);
