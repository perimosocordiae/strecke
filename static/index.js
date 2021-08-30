'use strict';

function startLogin() {
  let form = document.createElement('form');
  form.appendChild(makeInput('text', 'username', true, 'Username'));
  form.appendChild(makeInput('password', 'password', true, 'Password'));
  form.appendChild(makeInput('submit'));
  form.onsubmit = () => {
    if (!form.checkValidity()) return false;
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
  form.appendChild(makeInput('password', 'password', true, 'Choose a Password'));
  form.appendChild(makeInput('password', 'password2', true, 'Confirm Password'));
  form.appendChild(makeInput('submit'));
  form.onsubmit = () => {
    if (!form.checkValidity()) return false;
    doLogin(new FormData(form));
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

function doLogin(formData) {
  let xhr = new XMLHttpRequest();
  xhr.open('POST', '/login');
  xhr.setRequestHeader('Content-type', 'application/json');
  xhr.send(JSON.stringify(Object.fromEntries(formData)));
  xhr.onload = () => {
    let notAuth = document.getElementById('notAuth');
    if (xhr.status != 200) {
      let err = errorText(xhr.response);
      notAuth.lastElementChild.appendChild(err);
    } else {
      notAuth.style.display = 'none';
      document.getElementById('hasAuth').style.display = '';
    }
  };
  xhr.onerror = () => {
    let err = errorText(`Error code ${xhr.status}: ${xhr.response}`);
    document.getElementById('notAuth').lastElementChild.appendChild(err);
  };
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