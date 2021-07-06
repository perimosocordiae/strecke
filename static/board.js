
function bodyLoaded() {
  fetchJson('/board', renderBoard);
  fetchJson('/hand', renderHand);
}

function playTile(idx) {
  let xhr = new XMLHttpRequest();
  xhr.open('POST', `/play/${idx}`);
  xhr.send();
  xhr.onload = () => {
    if (xhr.status != 200) {
      console.error('Got', xhr.status, xhr.response);
    } else {
      fetchJson('/board', renderBoard);
      if (xhr.responseText != "OK") {
        document.getElementsByClassName('hand')[0].innerHTML = '';
        alert(xhr.responseText);
      } else {
        fetchJson('/hand', renderHand);
      }
    }
  };
  xhr.onerror = () => { console.error('Error', xhr.status, xhr.response); };
}

function rotateTile(playerIdx, tileIdx) {
  let xhr = new XMLHttpRequest();
  xhr.open('POST', `/rotate/${playerIdx}/${tileIdx}`);
  xhr.send();
  xhr.onload = () => {
    if (xhr.status != 200) {
      console.error('Got', xhr.status, xhr.response);
    } else {
      fetchJson('/hand', renderHand);
    }
  };
  xhr.onerror = () => { console.error('Error', xhr.status, xhr.response); };
}

function renderHand(hand) {
  let subtitle = document.getElementsByClassName('subtitle')[0];
  subtitle.innerText = `${hand.username}'s Tiles (${PLAYER_COLORS[hand.board_index]})`;
  let handContainer = document.getElementsByClassName('hand')[0];
  handContainer.innerHTML = '';
  for (let idx = 0; idx < hand.tiles_in_hand.length; ++idx) {
    let wrap = document.createElement('div');
    wrap.classList.add('choice');
    let elt = document.createElement('div');
    elt.classList.add('tile', `h${idx}`);
    elt.appendChild(renderTile(hand.tiles_in_hand[idx]));
    wrap.appendChild(elt);
    let rotBtn = document.createElement('button');
    rotBtn.innerText = 'Rotate';
    rotBtn.onclick = () => rotateTile(hand.board_index, idx);
    wrap.appendChild(rotBtn);
    let playBtn = document.createElement('button');
    playBtn.innerText = 'Play';
    playBtn.onclick = () => playTile(idx);
    wrap.appendChild(playBtn);
    handContainer.appendChild(wrap);
  }
}

const PLAYER_COLORS = [
  'red', 'blue', 'green', 'purple', 'magenta'
];

function renderBoard(board) {
  let boardContainer = document.getElementsByClassName('board')[0];
  boardContainer.innerHTML = '';
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
  for (const [idx, player] of board.players.entries()) {
    const [x, y] = PORT_LOCATIONS[player.port];
    let tile = boardContainer.querySelector(`.board > .r${player.row}.c${player.col}`);
    let token = document.createElement('div');
    token.classList.add('token');
    token.style.backgroundColor = PLAYER_COLORS[idx];
    token.style.top = `${y}%`;
    token.style.left = `${x}%`;
    tile.appendChild(token);
  }
}

function renderTile(tile) {
  let svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  svg.setAttribute('viewBox', '0 0 100 100');
  svg.setAttribute('xmlns', 'http://www.w3.org/2000/svg');
  svg.classList.add(tile.facing);
  let code = '';
  for (const [src, dst] of tile.layout) {
    code += pathCode(src, dst);
  }
  svg.appendChild(makePath(code));
  return svg;
}

function makeBorder(p0, p1) {
  let svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  svg.setAttribute('viewBox', '0 0 100 100');
  svg.setAttribute('xmlns', 'http://www.w3.org/2000/svg');
  const [x0, y0] = PORT_LOCATIONS[p0];
  const [x1, y1] = PORT_LOCATIONS[p1];

  let code;
  if (x0 == 0) {
    code = `M${x0} ${y0} h10 M${x1} ${y1} h10`;
  } else if (x0 == 100) {
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
  A: [33, 0], B: [66, 0], C: [100, 33], D: [100, 66],
  E: [66, 100], F: [33, 100], G: [0, 66], H: [0, 33],
};

function pathCode(src, dst) {
  const [x0, y0] = PORT_LOCATIONS[src];
  const [x1, y1] = PORT_LOCATIONS[dst];
  return `M${x0} ${y0} Q50 50 ${x1} ${y1} `;
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
  let xhr = new XMLHttpRequest();
  xhr.open('GET', url);
  xhr.responseType = 'json';
  xhr.send();
  xhr.onload = () => {
    if (xhr.status != 200) {
      console.error('Got', xhr.status, xhr.response);
      return;
    }
    cb(xhr.response);
  };
  xhr.onerror = () => { console.error('Error', xhr.status, xhr.response); };
}