'use strict';

function renderTile(tile, facing, svg) {
  if (!svg) {
    svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    svg.setAttribute('viewBox', '0 0 99 99');
    svg.setAttribute('xmlns', 'http://www.w3.org/2000/svg');
    svg.classList.add(facing);
  } else {
    svg.classList.value = facing;
    svg.removeChild(svg.firstChild);
  }
  let code = '';
  for (const [src, dst] of tile.layout) {
    code += pathCode(src, dst);
  }
  svg.appendChild(makePath(code));
  return svg;
}

const PORT_LOCATIONS = {
  A: [33, 0],
  B: [66, 0],
  C: [99, 33],
  D: [99, 66],
  E: [66, 99],
  F: [33, 99],
  G: [0, 66],
  H: [0, 33],
};

function pathCode(src, dst) {
  const [x0, y0] = PORT_LOCATIONS[src];
  const [x1, y1] = PORT_LOCATIONS[dst];
  const [cx0, cy0] = controlPoint(x0, y0);
  const [cx1, cy1] = controlPoint(x1, y1);
  return `M${x0} ${y0} C${cx0} ${cy0} ${cx1} ${cy1} ${x1} ${y1}`;
}

function controlPoint(x, y) {
  if (x == 0) {
    return [33, y];
  } else if (x == 99) {
    return [66, y];
  } else if (y == 0) {
    return [x, 33];
  } else {
    return [x, 66];
  }
}

function makePath(code) {
  let path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
  path.setAttribute('fill', 'none');
  path.setAttribute('stroke', '#000');
  path.setAttribute('d', code);
  return path;
}
