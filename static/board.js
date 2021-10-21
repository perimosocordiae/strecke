'use strict';
let playerPositions = [];
let gameId = 0;
let ws = null;

function bodyLoaded() {
  gameId = (new URL(window.location)).searchParams.get('id');
  fetchJson(`/board/${gameId}`, renderBoard);
  fetchJson(`/hand/${gameId}`, renderHand);

  ws = new WebSocket(`ws://${location.host}/ws/${gameId}`);
  ws.onopen = () => console.log('Opened WS connection.');
  ws.onmessage = (event) => {
    const msg = JSON.parse(event.data);
    console.log('Got WS message:', msg);
  }
  ws.onclose = () => console.log('Closed WS connection.');
}

function playTile(tileIdx) {
  fetch(`/play/${gameId}/${tileIdx}`, { method: 'POST' }).then(response => {
    if (response.ok) {
      fetchJson(`/board/${gameId}`, (board) => {
        renderBoard(board);
        response.text().then(text => {
          if (text != "OK") {
            document.getElementsByClassName('hand')[0].innerHTML = '';
            alert(text);
          } else {
            fetchJson(`/hand/${gameId}`, renderHand);
          }
        });
      });
    }
  });
}

function rotateTile(tileIdx) {
  fetch(`/rotate/${gameId}/${tileIdx}`, { method: 'POST' }).then(response => {
    if (response.ok) {
      fetchJson(`/hand/${gameId}`, (hand) => {
        renderHand(hand);
        // Trigger target tile update manually.
        let targetTile = document.querySelector('.board > .target');
        if (targetTile.firstChild) {
          targetTile.firstChild.classList.value = hand.tiles_in_hand[tileIdx].facing;
        }
      });
    }
  });
}

function renderHand(hand) {
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
      renderTile(hand.tiles_in_hand[idx], svg);
    } else {
      wrap = document.createElement('div');
      wrap.classList.add('choice');
      let elt = document.createElement('div');
      elt.classList.add('tile', `h${idx}`);
      svg = renderTile(hand.tiles_in_hand[idx]);
      elt.appendChild(svg);
      elt.onclick = () => rotateTile(idx);
      wrap.appendChild(elt);
      let rotBtn = document.createElement('button');
      rotBtn.innerText = 'Rotate';
      rotBtn.onclick = () => rotateTile(idx);
      wrap.appendChild(rotBtn);
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

const PLAYER_COLORS = ['red', 'blue', 'green', 'purple', 'magenta'];

function renderBoard(board) {
  let boardContainer = document.getElementsByClassName('board')[0];
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
        let tile = gridRow[col];
        if (tile) {
          elt.classList.add('played');
          elt.appendChild(renderTile(tile));
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
        let tile = gridRow[col];
        let elt = boardContainer.querySelector(`.tile.r${row}.c${col}`);
        while (elt.firstChild) {
          elt.removeChild(elt.firstChild);
        }
        elt.classList.remove('target');
        if (tile) {
          elt.classList.add('played');
          elt.appendChild(renderTile(tile));
        } else {
          elt.classList.remove('played');
        }
      }
    }
    boardContainer.querySelectorAll('.token').forEach(e => e.remove());
  }
  for (const [idx, player] of board.players.entries()) {
    playerPositions[idx] = nextPosition(player);
    const [x, y] = PORT_LOCATIONS[player.port];
    let tile = boardContainer.querySelector(`.r${player.row}.c${player.col}`);
    let token = document.createElement('div');
    token.classList.add('token');
    token.style.backgroundColor = PLAYER_COLORS[idx];
    token.style.top = `${y}%`;
    token.style.left = `${x}%`;
    tile.appendChild(token);
  }
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

function renderTile(tile, svg) {
  if (!svg) {
    svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    svg.setAttribute('viewBox', '0 0 99 99');
    svg.setAttribute('xmlns', 'http://www.w3.org/2000/svg');
    svg.classList.add(tile.facing);
  } else {
    svg.classList.value = tile.facing;
    svg.removeChild(svg.firstChild);
  }
  let code = '';
  for (const [src, dst] of tile.layout) {
    code += pathCode(src, dst);
  }
  svg.appendChild(makePath(code));
  return svg;
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

const PORT_LOCATIONS = {
  A: [33, 0],
  B: [66, 0],
  C: [99, 33],
  D: [99, 66],
  E: [66, 99],
  F: [33, 99],
  G: [0, 66],
  H: [0, 33],
};

function pathCode(src, dst) {
  const [x0, y0] = PORT_LOCATIONS[src];
  const [x1, y1] = PORT_LOCATIONS[dst];
  const [cx0, cy0] = controlPoint(x0, y0);
  const [cx1, cy1] = controlPoint(x1, y1);
  return `M${x0} ${y0} C${cx0} ${cy0} ${cx1} ${cy1} ${x1} ${y1}`;
}

function controlPoint(x, y) {
  if (x == 0) {
    return [33, y];
  } else if (x == 99) {
    return [66, y];
  } else if (y == 0) {
    return [x, 33];
  } else {
    return [x, 66];
  }
}

function makePath(code) {
  let path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
  path.setAttribute('fill', 'none');
  path.setAttribute('stroke', '#000');
  path.setAttribute('d', code);
  return path;
}

function makeCircle(cx, cy, radius, fillColor) {
  let c = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
  c.setAttribute('fill', fillColor);
  c.setAttribute('cx', cx);
  c.setAttribute('cy', cy);
  c.setAttribute('r', radius);
  return c;
}

function fetchJson(url, cb) {
  fetch(url).then(response => response.json()).then(cb);
}
