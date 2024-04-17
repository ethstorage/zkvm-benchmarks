import { readFile } from 'node:fs/promises';
import { WASI } from 'wasi';
import { argv, env } from 'node:process';

(async function () {
  const wasi = new WASI({
    version: 'preview1',
    args: argv,
    env,
    returnOnExit: true
  });
  const wasm = await WebAssembly.compile(
    await readFile(process.argv[2]),
  );

  let instance
  let cur = 0
  const numbers = argv.slice(2).map(Number);
  const hostio = {
    env: {
      wasm_input: (ispulic) => {
				const input = numbers[cur]
                console.log("input:", input, "cur:", cur)
				cur += 1
				return input
      },
      require: (cond) => {
        if (cond == 0) {
          console.log("require is not satisfied, which is a false assertion in the wasm code. Please check the logic of your image or input.");
          process.exit(1);
        }
      },
      wasm_output:(value) => {
        console.log("successfully run:", value)
      }
    }
  }

  instance = await WebAssembly.instantiate(wasm, hostio);
  wasi.start(instance);
})()



