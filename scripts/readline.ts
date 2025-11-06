import { ConcatenatedJsonParseStream } from "@std/json/concatenated-json-parse-stream";

export async function* readLines(
  f: ReadableStream
): AsyncGenerator<{}, undefined, undefined> {
  const readable = f
    .pipeThrough(new TextDecoderStream())
    .pipeThrough(new ConcatenatedJsonParseStream());

  for await (const data of readable) {
    yield data;
  }
}
