
function bodyLoaded() { fetchBoard(renderBoard); }

function fetchBoard(cb) {
  let xhr = new XMLHttpRequest();
  xhr.open('GET', '/board');
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

function renderBoard(board) {
  console.log('Grid:', board.grid);
  console.log('Players:', board.players);
}