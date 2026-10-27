// The Rust core, loaded from its WebAssembly bytes. Strings cross as UTF-8 in
// linear memory; every operation is painter.call(op, input) -> output (see
// crates/obsidian-painter/src/lib.rs for the list).

type Exports = {
  memory: WebAssembly.Memory;
  painter_alloc(len: number): number;
  painter_free(ptr: number, len: number): void;
  painter_call(op: number, opLen: number, input: number, inputLen: number): bigint;
};

export class Painter {
  private x: Exports;

  constructor(x: Exports) {
    this.x = x;
  }

  static async load(bytes: Uint8Array): Promise<Painter> {
    const module = await WebAssembly.compile(bytes as unknown as BufferSource);
    const instance = await WebAssembly.instantiate(module, {});
    return new Painter(instance.exports as unknown as Exports);
  }

  private put(s: string): [number, number] {
    const bytes = new TextEncoder().encode(s);
    const ptr = this.x.painter_alloc(bytes.length);
    new Uint8Array(this.x.memory.buffer, ptr, bytes.length).set(bytes);
    return [ptr, bytes.length];
  }

  /** Run one operation of the core; throws its error message. */
  call<T = unknown>(op: string, input: unknown): T {
    const [opPtr, opLen] = this.put(op);
    const [inPtr, inLen] = this.put(JSON.stringify(input));
    const packed = this.x.painter_call(opPtr, opLen, inPtr, inLen);
    this.x.painter_free(opPtr, opLen);
    this.x.painter_free(inPtr, inLen);
    const ptr = Number(packed >> 32n);
    const len = Number(packed & 0xffffffffn);
    const text = new TextDecoder().decode(new Uint8Array(this.x.memory.buffer, ptr, len));
    this.x.painter_free(ptr, len);
    const answer = JSON.parse(text) as { ok?: T; error?: string };
    if (answer.error !== undefined) throw new Error(answer.error);
    return answer.ok as T;
  }
}
