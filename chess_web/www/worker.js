// Web Worker running lucky_chess wasm engine

let wasmExports = null;
let ffiStatePtr = 0;
let cmdBufferPtr = 0;

const encoder = new TextEncoder();
const decoder = new TextDecoder();

async function initEngine(wasmPath = 'lucky_chess.wasm') {
  try {
    const response = await fetch(wasmPath);
    if (!response.ok) {
      throw new Error(`Failed to load WASM binary: ${response.statusText}`);
    }
    const bytes = await response.arrayBuffer();
    const imports = {
      env: {
        performance_now: () => performance.now(),
      },
    };
    const { instance } = await WebAssembly.instantiate(bytes, imports);

    wasmExports = instance.exports;
    ffiStatePtr = wasmExports.uci_new();
    cmdBufferPtr = wasmExports.uci_get_cmd_buffer();

    postMessage({ type: 'ready' });

    // Initialize UCI protocol
    sendCommand('uci');
    sendCommand('isready');
  } catch (err) {
    postMessage({ type: 'error', message: err.message || String(err) });
  }
}

function sendCommand(cmd) {
  if (!wasmExports || !ffiStatePtr) return;

  const encoded = encoder.encode(cmd);
  const memoryBuf = new Uint8Array(wasmExports.memory.buffer);
  const offset = cmdBufferPtr;

  if (encoded.length > 4096) {
    console.error('Command too long for buffer');
    return;
  }

  memoryBuf.set(encoded, offset);
  wasmExports.uci_send_cmd(ffiStatePtr, encoded.length);

  drainOutputs();
}

function drainOutputs() {
  if (!wasmExports || !ffiStatePtr) return;

  while (true) {
    const memoryBuf = new Uint8Array(wasmExports.memory.buffer);
    const offset = cmdBufferPtr;
    const n = wasmExports.uci_read_output(ffiStatePtr, cmdBufferPtr, 4096);
    const bytesRead = n;
    if (bytesRead === 0) break;

    const line = decoder.decode(memoryBuf.subarray(offset, offset + bytesRead)).trim();
    if (line.length > 0) {
      handleEngineOutput(line);
    }
  }
}

function handleEngineOutput(line) {
  postMessage({ type: 'uci_line', line });

  if (line.startsWith('bestmove')) {
    const parts = line.split(' ');
    const move = parts[1];
    postMessage({ type: 'bestmove', move });
  } else if (line.startsWith('info')) {
    // Parse info string: depth, score cp/mate, nodes, nps, pv
    const depthMatch = line.match(/\bdepth\s+(\d+)/);
    const scoreCpMatch = line.match(/\bscore\s+cp\s+(-?\d+)/);
    const scoreMateMatch = line.match(/\bscore\s+mate\s+(-?\d+)/);
    const nodesMatch = line.match(/\bnodes\s+(\d+)/);
    const npsMatch = line.match(/\bnps\s+(\d+)/);
    const pvMatch = line.match(/\bpv\s+(.+)$/);

    postMessage({
      type: 'info',
      depth: depthMatch ? parseInt(depthMatch[1], 10) : undefined,
      scoreCp: scoreCpMatch ? parseInt(scoreCpMatch[1], 10) : undefined,
      scoreMate: scoreMateMatch ? parseInt(scoreMateMatch[1], 10) : undefined,
      nodes: nodesMatch ? parseInt(nodesMatch[1], 10) : undefined,
      nps: npsMatch ? parseInt(npsMatch[1], 10) : undefined,
      pv: pvMatch ? pvMatch[1] : undefined,
      raw: line,
    });
  }
}

self.onmessage = (event) => {
  const { type, cmd, wasmPath } = event.data;

  switch (type) {
    case 'init':
      initEngine(wasmPath);
      break;
    case 'cmd':
      sendCommand(cmd);
      break;
    case 'stop':
      if (wasmExports && ffiStatePtr) {
        wasmExports.uci_stop(ffiStatePtr);
        drainOutputs();
      }
      break;
    default:
      console.warn('Unknown worker message type:', type);
  }
};
