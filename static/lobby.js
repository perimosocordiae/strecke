'use strict';
let lobbyCode = null;

function initLobby() {
  const urlParams = new URLSearchParams(location.search);
  lobbyCode = urlParams.get('code');
  const ws = new WebSocket(`ws://${location.host}/ws/${lobbyCode}`);
  ws.onopen = () => console.log('Opened WS connection.');
  ws.onmessage = (event) => {
    const msg = JSON.parse(event.data);
    console.log('Got WS message:', msg);
    if (msg.action === 'Update') {
      renderLobby(msg.lobby);
    } else if (msg.action === 'Start') {
      window.location.href = msg.url;
    } else if (msg.action === 'Error') {
      renderError(msg.message);
    }
  }
  ws.onclose = () => console.log('Closed WS connection.');
  fetch(`/lobby_data/${lobbyCode}`)
    .then((response) => response.json())
    .then(renderLobby);
}

function renderLobby(data) {
  // TODO!
  document.getElementById('lobby').innerText = JSON.stringify(data);
}

function renderError(message) {
  document.getElementById('error').innerText = message;
}

function takeSeat() {
  const seat = document.forms[0].seat.value;
  renderError('');
  fetch(`/lobby_seat/${lobbyCode}/${seat}`, { method: 'POST' });
}

function startGame() {
  renderError('');
  fetch(`/new_game/${lobbyCode}`, { method: 'POST' });
}
