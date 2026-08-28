import { Chessground } from 'https://esm.sh/@lichess-org/chessground@10.1.1';
import { Chess } from 'https://esm.sh/chess.js@1.4.0';

// Elements
const boardEl = document.getElementById('board');
const statusDot = document.querySelector('.dot');
const statusText = document.getElementById('status-text');
const evalScoreEl = document.getElementById('eval-score');
const evalBarEl = document.getElementById('eval-bar');
const depthEl = document.getElementById('metric-depth');
const npsEl = document.getElementById('metric-nps');
const nodesEl = document.getElementById('metric-nodes');
const pvEl = document.getElementById('metric-pv');
const depthSlider = document.getElementById('search-depth');
const depthValEl = document.getElementById('depth-val');
const playerColorSelect = document.getElementById('player-color');
const moveHistoryEl = document.getElementById('move-history');
const uciLogEl = document.getElementById('uci-log');

const btnNewGame = document.getElementById('btn-new-game');
const btnFlip = document.getElementById('btn-flip');
const btnUndo = document.getElementById('btn-undo');
const btnEngineMove = document.getElementById('btn-engine-move');

// Game State
const chess = new Chess();
let ground;
let worker;
let engineIsSearching = false;
let moveHistory = [];

// Initialize Web Worker
function initWorker() {
  worker = new Worker('worker.js', { type: 'module' });

  worker.onmessage = (event) => {
    const data = event.data;

    switch (data.type) {
      case 'ready':
        statusDot.className = 'dot ready';
        statusText.textContent = 'Engine Ready (WASM64)';
        checkEngineTurn();
        break;

      case 'error':
        statusDot.className = 'dot error';
        statusText.textContent = `Error: ${data.message}`;
        break;

      case 'uci_line':
        appendLog(data.line);
        break;

      case 'info':
        updateMetrics(data);
        break;

      case 'bestmove':
        handleBestMove(data.move);
        break;
    }
  };

  worker.postMessage({ type: 'init', wasmPath: 'lucky_chess.wasm' });
}

function calculateDests() {
  const dests = new Map();
  const moves = chess.moves({ verbose: true });

  for (const m of moves) {
    if (!dests.has(m.from)) {
      dests.set(m.from, []);
    }
    dests.get(m.from).push(m.to);
  }
  return dests;
}

function initChessground() {
  ground = Chessground(boardEl, {
    orientation: 'white',
    fen: chess.fen(),
    turnColor: chess.turn() === 'w' ? 'white' : 'black',
    movable: {
      free: false,
      color: 'white',
      dests: calculateDests(),
      events: {
        after: onPlayerMove,
      },
    },
    premovable: {
      enabled: true,
    },
    animation: {
      enabled: true,
      duration: 200,
    },
    highlight: {
      lastMove: true,
      check: true,
    },
  });
}

function onPlayerMove(orig, dest) {
  // Check promotion
  const piece = chess.get(orig);
  const isPawn = piece && piece.type === 'p';
  const isPromotion = isPawn && ((dest[1] === '8' && piece.color === 'w') || (dest[1] === '1' && piece.color === 'b'));

  let move = null;
  try {
    move = chess.move({
      from: orig,
      to: dest,
      promotion: isPromotion ? 'q' : undefined,
    });
  } catch {
    move = null;
  }

  if (move) {
    moveHistory.push(move.lan || `${orig}${dest}`);
    updateUI();
    checkEngineTurn();
  } else {
    ground.set({ fen: chess.fen() });
  }
}

function checkEngineTurn() {
  const turn = chess.turn(); // 'w' | 'b'
  const mode = playerColorSelect.value;

  if (chess.isGameOver()) {
    handleGameOver();
    return;
  }

  const isEngineTurn =
    (mode === 'white' && turn === 'b') ||
    (mode === 'black' && turn === 'w');

  if (isEngineTurn) {
    triggerEngineMove();
  }
}

function triggerEngineMove() {
  if (engineIsSearching || chess.isGameOver()) return;

  engineIsSearching = true;
  statusDot.className = 'dot loading';
  statusText.textContent = 'Engine thinking...';

  // Send position with move list
  const movesStr = moveHistory.join(' ');
  const posCmd = movesStr.length > 0 ? `position startpos moves ${movesStr}` : 'position startpos';
  worker.postMessage({ type: 'cmd', cmd: posCmd });

  const depth = depthSlider.value;
  worker.postMessage({ type: 'cmd', cmd: `go depth ${depth}` });
}

function handleBestMove(bestMoveStr) {
  engineIsSearching = false;
  statusDot.className = 'dot ready';
  statusText.textContent = 'Engine Ready';

  if (!bestMoveStr || bestMoveStr === '(none)' || bestMoveStr === '0000') {
    return;
  }

  const from = bestMoveStr.substring(0, 2);
  const to = bestMoveStr.substring(2, 4);
  const promo = bestMoveStr.length > 4 ? bestMoveStr[4] : undefined;

  let move = null;
  try {
    move = chess.move({
      from,
      to,
      promotion: promo,
    });
  } catch {
    move = null;
  }

  if (move) {
    moveHistory.push(bestMoveStr);
    ground.set({
      fen: chess.fen(),
      lastMove: [from, to],
    });
    updateUI();
  }
}

function updateUI() {
  const turnColor = chess.turn() === 'w' ? 'white' : 'black';
  const mode = playerColorSelect.value;

  let movableColor;
  if (mode === 'both') {
    movableColor = turnColor;
  } else if (mode === 'white') {
    movableColor = turnColor === 'white' ? 'white' : undefined;
  } else if (mode === 'black') {
    movableColor = turnColor === 'black' ? 'black' : undefined;
  }

  ground.set({
    turnColor,
    movable: {
      color: movableColor,
      dests: calculateDests(),
    },
    check: chess.inCheck(),
  });

  renderMoveHistory();
}

function renderMoveHistory() {
  let html = '';
  const history = chess.history();
  for (let i = 0; i < history.length; i += 2) {
    const moveNum = Math.floor(i / 2) + 1;
    const whiteMove = history[i] || '';
    const blackMove = history[i + 1] || '';
    html += `<div>${moveNum}.</div><div>${whiteMove}</div><div>${blackMove}</div>`;
  }
  moveHistoryEl.innerHTML = html;
  moveHistoryEl.scrollTop = moveHistoryEl.scrollHeight;
}

function updateMetrics(info) {
  if (info.depth !== undefined) depthEl.textContent = String(info.depth);
  if (info.nps !== undefined) npsEl.textContent = info.nps.toLocaleString();
  if (info.nodes !== undefined) nodesEl.textContent = info.nodes.toLocaleString();
  if (info.pv) pvEl.textContent = info.pv;

  const isBlackTurn = chess.turn() === 'b';

  if (info.scoreCp !== undefined) {
    const whiteScoreCp = isBlackTurn ? -info.scoreCp : info.scoreCp;
    const score = (whiteScoreCp / 100).toFixed(2);
    const sign = whiteScoreCp > 0 ? '+' : '';
    evalScoreEl.textContent = `${sign}${score}`;

    // Map centipawns to bar width percentage (50% is 0.00, clamped between 5% and 95%)
    const clamped = Math.max(-1000, Math.min(1000, whiteScoreCp));
    const percent = 50 + (clamped / 1000) * 45;
    evalBarEl.style.width = `${percent}%`;
  } else if (info.scoreMate !== undefined) {
    const whiteScoreMate = isBlackTurn ? -info.scoreMate : info.scoreMate;
    const sign = whiteScoreMate > 0 ? '+' : '-';
    evalScoreEl.textContent = `${sign}M${Math.abs(whiteScoreMate)}`;
    evalBarEl.style.width = whiteScoreMate > 0 ? '98%' : '2%';
  }

  // Draw arrow on board for best move if available
  if (info.pv) {
    const firstPvMove = info.pv.split(' ')[0];
    if (firstPvMove && firstPvMove.length >= 4) {
      const orig = firstPvMove.substring(0, 2);
      const dest = firstPvMove.substring(2, 4);
      ground.setShapes([
        {
          orig,
          dest,
          brush: 'green',
        },
      ]);
    }
  }
}

function handleGameOver() {
  statusDot.className = 'dot';
  if (chess.isCheckmate()) {
    statusText.textContent = `Checkmate! ${chess.turn() === 'w' ? 'Black' : 'White'} wins.`;
  } else if (chess.isDraw()) {
    statusText.textContent = 'Game drawn.';
  }
}

function appendLog(line) {
  const p = document.createElement('div');
  p.textContent = line;
  uciLogEl.appendChild(p);
  uciLogEl.scrollTop = uciLogEl.scrollHeight;
}

// Event Listeners
btnNewGame.addEventListener('click', () => {
  if (engineIsSearching) {
    worker.postMessage({ type: 'stop' });
  }
  chess.reset();
  moveHistory = [];
  ground.set({
    fen: chess.fen(),
    lastMove: undefined,
  });
  ground.setShapes([]);
  evalScoreEl.textContent = '0.00';
  evalBarEl.style.width = '50%';
  depthEl.textContent = '-';
  npsEl.textContent = '-';
  nodesEl.textContent = '-';
  pvEl.textContent = '-';
  uciLogEl.innerHTML = '';
  worker.postMessage({ type: 'cmd', cmd: 'ucinewgame' });
  updateUI();
  checkEngineTurn();
});

btnFlip.addEventListener('click', () => {
  ground.toggleOrientation();
});

btnUndo.addEventListener('click', () => {
  if (engineIsSearching) {
    worker.postMessage({ type: 'stop' });
  }
  // Undo 2 moves if playing vs engine
  if (playerColorSelect.value !== 'both') {
    chess.undo();
    chess.undo();
    moveHistory.pop();
    moveHistory.pop();
  } else {
    chess.undo();
    moveHistory.pop();
  }
  ground.set({
    fen: chess.fen(),
    lastMove: undefined,
  });
  updateUI();
});

btnEngineMove.addEventListener('click', () => {
  triggerEngineMove();
});

playerColorSelect.addEventListener('change', () => {
  const mode = playerColorSelect.value;
  if (mode === 'black') {
    ground.set({ orientation: 'black' });
  } else {
    ground.set({ orientation: 'white' });
  }
  updateUI();
  checkEngineTurn();
});

depthSlider.addEventListener('input', () => {
  depthValEl.textContent = depthSlider.value;
});

// Bootstrap
initChessground();
initWorker();
