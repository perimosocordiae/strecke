'use strict';

function startLogin() {
  let form = document.createElement('form');
  form.appendChild(makeInput('text', 'username', true, 'Username'));
  form.appendChild(makeInput('password', 'password', true, 'Password'));
  form.appendChild(makeInput('submit'));
  form.onsubmit = () => {
    if (!form.checkValidity())
      return false;
    doLogin(new FormData(form));
    return false;
  };
  let parent = document.getElementById('notAuth').lastElementChild;
  parent.innerHTML = '';
  parent.appendChild(form);
}

function startRegister() {
  let form = document.createElement('form');
  form.appendChild(makeInput('text', 'username', true, 'Choose a Username'));
  form.appendChild(
    makeInput('password', 'password', true, 'Choose a Password'));
  form.appendChild(
    makeInput('password', 'password2', true, 'Confirm Password'));
  form.appendChild(makeInput('submit'));
  form.onsubmit = () => {
    if (!form.checkValidity())
      return false;
    doRegister(new FormData(form));
    return false;
  };
  let parent = document.getElementById('notAuth').lastElementChild;
  parent.innerHTML = '';
  parent.appendChild(form);
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

function doLogin(formData) {
  fetch('/login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(Object.fromEntries(formData)),
  }).then(handleLoginResponse);
}

function doRegister(formData) {
  fetch('/register', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(Object.fromEntries(formData)),
  }).then(handleLoginResponse);
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
    s.appendChild(renderTile({ facing: 'North', layout: LETTERS[letter] }));
    logo.appendChild(s);
  }
}