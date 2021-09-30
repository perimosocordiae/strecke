'use strict';

function fetchLobby() {
  const urlParams = new URLSearchParams(window.location.search);
  const lobbyCode = urlParams.get('code');
  fetch(`/lobby_data/${lobbyCode}`)
      .then((response) => response.json())
      .then((data) => {
        document.getElementById('lobby').innerText = JSON.stringify(data);
      });
}