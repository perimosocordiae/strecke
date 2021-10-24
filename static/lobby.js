'use strict';
let LOBBY_CODE = null;
let USERNAME = null;

function initLobby() {
  fetch('/check_login').then(response => {
    if (!response.ok) {
      window.location.href = '/';
      return;
    }
    response.text().then(text => { USERNAME = text; });
    const urlParams = new URLSearchParams(location.search);
    LOBBY_CODE = urlParams.get('code');
    const ws = new WebSocket(`ws://${location.host}/ws/${LOBBY_CODE}`);
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
    fetch(`/lobby_data/${LOBBY_CODE}`)
      .then((response) => response.json())
      .then(renderLobby);
  });
}

function renderLobby(data) {
  const isHost = data.names[0] === USERNAME;
  const isInLobby = isHost || data.names.includes(USERNAME);
  // TODO: Render the seats graphically.
  const lobbyDiv = document.getElementById('lobby');
  lobbyDiv.innerHTML = '';
  const table = document.createElement('table');
  const header = document.createElement('tr');
  header.innerHTML = '<th>Player</th><th>Seat</th>';
  table.appendChild(header);
  for (let i = 0; i < data.max_num_players; i++) {
    const username = data.names[i];
    const seat = data.start_positions[i];

    const tableRow = document.createElement('tr');
    if (!username) {
      const openCell = document.createElement('td');
      openCell.colSpan = 2;
      if (isInLobby) {
        openCell.innerText = 'Waiting for player...';
      } else {
        renderTakeSeatForm(openCell, 48);
      }
      tableRow.appendChild(openCell);
    } else {
      const nameCell = document.createElement('td');
      const seatCell = document.createElement('td');
      nameCell.innerText = username;
      if (username === USERNAME) {
        renderTakeSeatForm(seatCell, seat);
      } else {
        seatCell.innerText = seat === 48 ? 'Not seated' : `Seat ${seat}`;
      }
      tableRow.appendChild(nameCell);
      tableRow.appendChild(seatCell);
    }
    table.appendChild(tableRow);
  }
  lobbyDiv.appendChild(table);
  if (isHost) {
    // TODO: allow the host to add more players.
  }
  if (isInLobby && isHost) {
    const startGameButton = document.createElement('button');
    startGameButton.innerText = 'Start Game';
    startGameButton.onclick = startGame;
    lobbyDiv.appendChild(startGameButton);
  }
}

function renderTakeSeatForm(parent, seat) {
  parent.innerHTML = 'Take a seat: ';
  const seatForm = document.createElement('form');
  const seatInput = document.createElement('input');
  seatInput.name = 'seat';
  seatInput.type = 'number';
  seatInput.min = 0;
  seatInput.max = 47;
  if (seat < 48) {
    seatInput.value = seat;
  }
  seatInput.onchange = takeSeat;
  seatForm.appendChild(seatInput);
  parent.appendChild(seatForm);
}

function renderError(message) {
  document.getElementById('error').innerText = message;
}

function takeSeat() {
  const seat = document.forms[0].seat.value;
  if (seat >= 0 && seat <= 47) {
    renderError('');
    fetch(`/lobby_seat/${LOBBY_CODE}/${seat}`, { method: 'POST' });
  } else {
    renderError('Please enter a valid seat number: 0 to 47.');
  }
}

function startGame() {
  renderError('');
  fetch(`/new_game/${LOBBY_CODE}`, { method: 'POST' });
}
