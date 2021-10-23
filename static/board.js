'use strict';
const ROTATE = {
  North: 'West',
  West: 'South',
  South: 'East',
  East: 'North',
}
const PORT_FLIPS = {
  'A': 'F', 'B': 'E', 'C': 'H', 'D': 'G', 'E': 'B', 'F': 'A', 'G': 'D', 'H': 'C'
};
const PORT_LEFT_TURNS = {
  'A': 'G', 'B': 'H', 'C': 'A', 'D': 'B', 'E': 'C', 'F': 'D', 'G': 'E', 'H': 'F'
};
const PORT_RIGHT_TURNS = {
  'A': 'C', 'B': 'D', 'C': 'E', 'D': 'F', 'E': 'G', 'F': 'H', 'G': 'A', 'H': 'B'
};
const PLAYER_COLORS = ['red', 'blue', 'green', 'purple', 'magenta'];

let playerPositions = [];
let rotations = [];
let gameId = 0;

function bodyLoaded() {
  gameId = (new URL(window.location)).searchParams.get('id');
  fetchJson(`/board/${gameId}`, (board) => {
    if (renderBoard(board)) {
      fetchJson(`/hand/${gameId}`, renderHand);
    }
  });

  const ws = new WebSocket(`ws://${location.host}/ws/${gameId}`);
  ws.onopen = () => console.log('Opened WS connection.');
  ws.onmessage = (event) => {
    const msg = JSON.parse(event.data);
    console.log('Got WS message:', msg);
    if (msg.action === 'Update') {
      if (renderBoard(msg.board)) {
        fetchJson(`/hand/${gameId}`, renderHand);
      }
    } else if (msg.action === 'GameOver') {
      renderBoard(msg.board);
      document.querySelector('.hand').innerHTML = '';
      if (msg.winner) {
        alert(`Game over: ${msg.winner} is the winner!`);
      } else {
        alert('Game over: everyone lost!');
      }
    } else if (msg.action === 'Error') {
      renderError(msg.message);
    }
  }
  ws.onclose = () => console.log('Closed WS connection.');
}

function renderError(message) {
  document.getElementById('error').innerText = message;
}

function playTile(tileIdx) {
  fetch('/play', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      game_id: +gameId, idx: tileIdx, facing: rotations[tileIdx],
    }),
  });
}

function rotateTile(tileIdx) {
  let new_facing = ROTATE[rotations[tileIdx]];
  rotations[tileIdx] = new_facing;
  let handSvg = document.querySelector(`.hand .choice:nth-child(${tileIdx + 1}) svg`);
  if (handSvg) handSvg.classList.value = new_facing;
  let targetSvg = document.querySelector('.board > .target > svg');
  if (targetSvg) targetSvg.classList.value = new_facing;
}

function renderHand(hand) {
  if (hand === "Game not found.") {
    renderError(hand);
    return;
  }
  // Hack to ensure renderBoard ran first on pageload.
  if (!playerPositions[hand.board_index]) {
    setTimeout(() => renderHand(hand), 10);
    return;
  }
  let subtitle = document.getElementsByClassName('subtitle')[0];
  subtitle.innerText =
    `${hand.username}'s Tiles (${PLAYER_COLORS[hand.board_index]})`;
  if (!document.querySelector('.board > .target')) {
    let [row, col] = playerPositions[hand.board_index];
    document.querySelector(`.board > .r${row}.c${col}`).classList.add('target');
  }
  let handContainer = document.getElementsByClassName('hand')[0];
  const handSize = hand.tiles_in_hand.length;
  for (let idx = 0; idx < handSize; ++idx) {
    let wrap, svg;
    if (idx < handContainer.children.length) {
      wrap = handContainer.children[idx];
      svg = wrap.firstChild.firstChild;
      renderTile(hand.tiles_in_hand[idx], rotations[idx], svg);
    } else {
      rotations.push('North');
      wrap = document.createElement('div');
      wrap.classList.add('choice');
      let elt = document.createElement('div');
      elt.classList.add('tile', `h${idx}`);
      svg = renderTile(hand.tiles_in_hand[idx], rotations[idx]);
      elt.appendChild(svg);
      elt.onclick = () => rotateTile(idx);
      wrap.appendChild(elt);
      let playBtn = document.createElement('button');
      playBtn.innerText = 'Play';
      playBtn.onclick = () => playTile(idx);
      wrap.appendChild(playBtn);
      handContainer.appendChild(wrap);
    }
    wrap.onmouseenter = () => {
      let targetTile = document.querySelector('.board > .target');
      targetTile.innerHTML = '';
      targetTile.appendChild(svg.cloneNode(true));
    };
    wrap.onmouseleave = () => {
      document.querySelector('.board > .target').innerHTML = '';
    };
  }
  while (handContainer.children.length > handSize) {
    handContainer.removeChild(handContainer.lastChild);
  }
}

function renderBoard(board) {
  let boardContainer = document.getElementsByClassName('board')[0];
  if (board === "Game not found.") {
    boardContainer.innerHTML = '';
    renderError(board);
    return false;
  }
  if (boardContainer.children.length == 0) {
    boardContainer.appendChild(document.createElement('div'));
    for (let col = 0; col < 6; ++col) {
      let pad = document.createElement('div');
      pad.classList.add('pad', 'r-1', `c${col}`);
      pad.appendChild(makeBorder('E', 'F'));
      boardContainer.appendChild(pad);
    }
    boardContainer.appendChild(document.createElement('div'));
    for (let row = 0; row < board.grid.length; ++row) {
      let pad = document.createElement('div');
      pad.classList.add('pad', `r${row}`, 'c-1');
      pad.appendChild(makeBorder('C', 'D'));
      boardContainer.appendChild(pad);
      const gridRow = board.grid[row];
      for (let col = 0; col < gridRow.length; ++col) {
        let elt = document.createElement('div');
        elt.classList.add('tile', `r${row}`, `c${col}`);
        if (gridRow[col]) {
          elt.classList.add('played');
          elt.appendChild(renderTile(...gridRow[col]));
        }
        boardContainer.appendChild(elt);
      }
      pad = document.createElement('div');
      pad.classList.add('pad', `r${row}`, 'c6');
      pad.appendChild(makeBorder('G', 'H'));
      boardContainer.appendChild(pad);
    }
    boardContainer.appendChild(document.createElement('div'));
    for (let col = 0; col < 6; ++col) {
      let pad = document.createElement('div');
      pad.classList.add('pad', 'r6', `c${col}`);
      pad.appendChild(makeBorder('A', 'B'));
      boardContainer.appendChild(pad);
    }
    boardContainer.appendChild(document.createElement('div'));
  } else {
    for (let row = 0; row < board.grid.length; ++row) {
      const gridRow = board.grid[row];
      for (let col = 0; col < gridRow.length; ++col) {
        let elt = boardContainer.querySelector(`.tile.r${row}.c${col}`);
        while (elt.firstChild) {
          elt.removeChild(elt.firstChild);
        }
        elt.classList.remove('target');
        if (gridRow[col]) {
          elt.classList.add('played');
          elt.appendChild(renderTile(...gridRow[col]));
        } else {
          elt.classList.remove('played');
        }
      }
    }
    boardContainer.querySelectorAll('.token').forEach(e => e.remove());
  }
  // Update player positions.
  for (const [idx, playerTrail] of board.players.entries()) {
    const color = PLAYER_COLORS[idx];
    let tileDiv;
    for (const pos of playerTrail) {
      tileDiv = boardContainer.querySelector(`.r${pos.row}.c${pos.col}`);
      if (tileDiv.classList.contains('played')) {
        // Add an svg path for the trail.
        let [gridTile, facing] = board.grid[pos.row][pos.col];
        let origPort = unnormalizePort(pos.port, facing);
        let [p0, p1] = gridTile.layout.find(p => p.includes(origPort));
        let trailPath = makePath(pathCode(p0, p1));
        trailPath.setAttribute('stroke', color);
        trailPath.setAttribute('stroke-width', 3);
        tileDiv.firstChild.appendChild(trailPath);
      }
    }
    const pos = playerTrail[playerTrail.length - 1];
    playerPositions[idx] = nextPosition(pos);
    // Mark the current position.
    const [x, y] = PORT_LOCATIONS[pos.port];
    let token = document.createElement('div');
    token.classList.add('token');
    if (!pos.alive) token.classList.add('dead');
    token.style.backgroundColor = color;
    token.style.top = `${y}%`;
    token.style.left = `${x}%`;
    tileDiv.appendChild(token);
  }
  return true;
}

function unnormalizePort(port, facing) {
  if (facing == 'North') return port;
  if (facing == 'South') return PORT_FLIPS[port];
  if (facing == 'East') return PORT_LEFT_TURNS[port];
  if (facing == 'West') return PORT_RIGHT_TURNS[port];
  throw new Error(`Invalid facing: ${facing}`);
}

function nextPosition(player) {
  if ('AB'.includes(player.port)) {
    return [player.row - 1, player.col];
  } else if ('CD'.includes(player.port)) {
    return [player.row, player.col + 1];
  } else if ('EF'.includes(player.port)) {
    return [player.row + 1, player.col];
  }
  return [player.row, player.col - 1];
}

function makeBorder(p0, p1) {
  let svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  svg.setAttribute('viewBox', '0 0 99 99');
  svg.setAttribute('xmlns', 'http://www.w3.org/2000/svg');
  const [x0, y0] = PORT_LOCATIONS[p0];
  const [x1, y1] = PORT_LOCATIONS[p1];

  let code;
  if (x0 == 0) {
    code = `M${x0} ${y0} h10 M${x1} ${y1} h10`;
  } else if (x0 == 99) {
    code = `M${x0} ${y0} h-10 M${x1} ${y1} h-10`;
  } else if (y0 == 0) {
    code = `M${x0} ${y0} v10 M${x1} ${y1} v10`;
  } else {
    code = `M${x0} ${y0} v-10 M${x1} ${y1} v-10`;
  }
  svg.appendChild(makePath(code));
  return svg;
}

function fetchJson(url, cb) {
  fetch(url).then(response => response.json()).then(cb);
}
