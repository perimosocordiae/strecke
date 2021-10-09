'use strict';

var lobbyCode = null;
var ws = null;

function initLobby() {
  const urlParams = new URLSearchParams(location.search);
  lobbyCode = urlParams.get('code');
  ws = new WebSocket(`ws://${location.host}/ws`);
  ws.onopen = () => console.log('Opened WS connection.');
  ws.onmessage = (msg) => console.log('Got WS message:', msg.data);
  ws.onclose = () => console.log('Closed WS connection.');
  fetchLobby();
}

function fetchLobby() {
  fetch(`/lobby_data/${lobbyCode}`)
    .then((response) => response.json())
    .then((data) => {
      document.getElementById('lobby').innerText = JSON.stringify(data);
      // document.forms[0].seat.value = data...
    });
}

function takeSeat() {
  const seat = document.forms[0].seat.value;
  fetch(`/lobby_seat/${lobbyCode}/${seat}`, {
    method: 'POST',
  }).then((response) => {
    if (response.ok) {
      fetchLobby();
    }
  })
}

function startGame() {
  fetch(`/new_game/${lobbyCode}`, { method: 'POST' }).then((response) => {
    if (response.redirected) {
      window.location.href = response.url;
    } else {
      response.text().then((msg) => document.getElementById('lobby').innerText =
        msg);
    }
  });
}
