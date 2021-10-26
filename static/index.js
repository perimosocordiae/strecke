'use strict';

function startLogin() {
  document.getElementById('registerForm').style.display = '';
  document.getElementById('loginForm').style.display = 'inline-flex';
}

function startRegister() {
  document.getElementById('loginForm').style.display = '';
  document.getElementById('registerForm').style.display = 'inline-flex';
}

function submitLogin(form) {
  if (!form.checkValidity())
    return false;
  fetch('/login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(Object.fromEntries(new FormData(form))),
  }).then(handleLoginResponse);
  return false;
}

function submitRegister(form) {
  if (!form.checkValidity())
    return false;
  fetch('/register', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(Object.fromEntries(new FormData(form))),
  }).then(handleLoginResponse);
  return false;
}

function makeInput(type, name, required, placeholder) {
  let elt = document.createElement('input');
  elt.type = type;
  if (name) {
    elt.name = name;
  }
  if (required) {
    elt.required = required;
  }
  if (placeholder) {
    elt.placeholder = placeholder;
  }
  return elt;
}

function errorText(msg) {
  let elt = document.createElement('span');
  elt.classList.add('error');
  elt.innerText = msg;
  return elt;
}

function handleLoginResponse(response) {
  let notAuth = document.getElementById('notAuth');
  if (response.ok) {
    notAuth.style.display = 'none';
    document.getElementById('hasAuth').style.display = '';
  } else {
    response.text().then(
      text => notAuth.lastElementChild.appendChild(errorText(text)));
  }
}

function checkLoginStatus() {
  fetch('/check_login').then(response => {
    if (response.ok) {
      document.getElementById('notAuth').style.display = 'none';
      document.getElementById('hasAuth').style.display = '';
    }
  });
}

function renderLogo() {
  const LETTERS = {
    s: [['A', 'E']],
    t: [['A', 'F'], ['H', 'C']],
    r: [['H', 'E'], ['H', 'B']],
    e: [['G', 'H'], ['H', 'B'], ['G', 'E']],
    c: [['C', 'H'], ['H', 'D']],
    k: [['H', 'E'], ['G', 'B'], ['A', 'F']],
  };
  let logo = document.getElementById('logo');
  for (let letter of 'strecke') {
    let s = document.createElement('div');
    s.classList.add('tile');
    s.appendChild(renderTile({ layout: LETTERS[letter] }, 'North'));
    logo.appendChild(s);
  }
  // Keep the tiles square.
  const resizer = () => {
    logo.style.height = logo.firstElementChild.offsetWidth + 'px';
  };
  window.addEventListener('resize', resizer);
  resizer();
}