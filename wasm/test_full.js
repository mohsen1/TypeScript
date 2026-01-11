const { ThinParser } = require('./pkg/wasm.js');

const code = `
import { MyPromise } from "missing";

declare var p: Promise<number>;
declare var mp: MyPromise<number>;

async function f0() { }
async function f1(): Promise<void> { }
async function f3(): MyPromise<void> { }

let f4 = async function() { }
let f5 = async function(): Promise<void> { }
let f6 = async function(): MyPromise<void> { }

let f7 = async () => { };
let f8 = async (): Promise<void> => { };
let f9 = async (): MyPromise<void> => { };
`;

const parser = new ThinParser('test.ts', code);
parser.parseSourceFile();
const result = JSON.parse(parser.checkSourceFile());

const ts2355 = (result.diagnostics || []).filter(d => d.code === 2355);
console.log(`Found ${ts2355.length} TS2355 errors:`);
for (const diag of ts2355) {
  console.log(`  - ${diag.message_text}`);
}
