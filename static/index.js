'use strict';

function doLogin() {
    let xhr = new XMLHttpRequest();
    xhr.open('POST', '/login');
    xhr.setRequestHeader('Content-type', 'application/json');
    xhr.send(JSON.stringify({ username: 'CJ', password: 'abcd' }));
    xhr.onload = () => {
      if (xhr.status != 200) {
        alert(`Got ${xhr.status}: ${xhr.response}`);
      } else {
        alert('Logged in');
      }
    };
    xhr.onerror = () => { console.error('Error', xhr.status, xhr.response); };
  }
  
  function doRegister() {
    let xhr = new XMLHttpRequest();
    xhr.open('POST', '/register');
    xhr.setRequestHeader('Content-type', 'application/json');
    xhr.send(JSON.stringify({ username: 'CJ', password: 'abcd' }));
    xhr.onload = () => {
      if (xhr.status != 200) {
        alert(`Got ${xhr.status}: ${xhr.response}`);
      } else {
        alert('Registered user');
      }
    };
    xhr.onerror = () => { console.error('Error', xhr.status, xhr.response); };
  }