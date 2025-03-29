import { homedir } from 'os';
import { resolve } from 'path';
import { spawn, spawnSync } from 'child_process';

// keep track of the `tauri-driver` child process
let tauriDriver;

export const config = {
  specs: ['./tests/specs/**/*.js'],
  maxInstances: 1,
  capabilities: [
    {
       maxInstances: 1,
       'tauri:options': {
        application: './src-tauri/target/release/app.exe',
      },
    },
  ],
  reporters: ['spec'],
  framework: 'mocha',
  mochaOpts: {
    ui: 'bdd',
    timeout: 60000,
  },
  hostname: 'localhost',
  port: 4444,
  path: '/',
  protocol: 'http',

  // ensure the rust project is built since we expect this binary to exist for the webdriver sessions
  onPrepare: () => spawnSync('cargo', ['build', '--release', '--features', '\"ollama, llama_cpp\"']),

  // ensure we are running `tauri-driver` before the session starts so that we can proxy the webdriver requests
  beforeSession: () => {
    const cargoHome = process.env.CARGO_HOME || resolve(homedir(), '.cargo');
    const tauriDriverPath = resolve(cargoHome, 'bin', 'tauri-driver');
    tauriDriver = spawn(tauriDriverPath, [], {
      stdio: [null, process.stdout, process.stderr]
    });
  },

  // clean up the `tauri-driver` process we spawned at the start of the session
  afterSession: () => tauriDriver.kill(),
};