// Generate ring-band OBJ files and oblate spheroid for the solar system scene.
const fs = require('fs');
const path = require('path');

const OUT = path.join(__dirname, '..', 'mods', 'asteroids', 'assets', '3d');

function fmt(f) { return f.toFixed(6); }
function vline(x, y, z) { return `v ${fmt(x)} ${fmt(y)} ${fmt(z)}`; }

// ── ring half-annulus ─────────────────────────────────────────────────────────
function ringHalf(innerR, nSegs, front) {
  const verts = [], faces = [];
  const start = front ? 0 : Math.PI;
  const end   = front ? Math.PI : 2 * Math.PI;
  const angles = Array.from({length: nSegs + 1}, (_, i) => start + i * (end - start) / nSegs);
  for (const theta of angles) {
    verts.push([Math.cos(theta), 0.0, Math.sin(theta)]);
    verts.push([innerR * Math.cos(theta), 0.0, innerR * Math.sin(theta)]);
  }
  for (let i = 0; i < nSegs; i++) {
    const o0 = 2*i+1, i0 = 2*i+2, o1 = 2*(i+1)+1, i1 = 2*(i+1)+2;  // 1-indexed
    faces.push([o0, o1, i1, i0]);
  }
  return { verts, faces };
}

function writeRingHalf(name, objId, innerR, nSegs, front) {
  const { verts, faces } = ringHalf(innerR, nSegs, front);
  const lines = [
    `# ${name}  inner_r=${innerR.toFixed(3)}  n_segs=${nSegs}  ${front ? 'front' : 'back'}`,
    `o ${objId}`, ''
  ];
  for (const [x,y,z] of verts) lines.push(vline(x,y,z));
  lines.push('');
  for (const [a,b,c,d] of faces) lines.push(`f ${a} ${b} ${c} ${d}`);
  lines.push('');
  fs.writeFileSync(path.join(OUT, name), lines.join('\n'));
  console.log(`  wrote ${name}  (${verts.length} verts, ${faces.length} quads)`);
}

// ── oblate spheroid ───────────────────────────────────────────────────────────
function writeOblate(name, nLon=24, nLat=16, flattenY=0.902) {
  const verts = [], faces = [];
  const latAngles = Array.from({length: nLat+1}, (_, i) => -Math.PI/2 + i*Math.PI/nLat);
  const lonAngles = Array.from({length: nLon}, (_, j) => j*2*Math.PI/nLon);
  for (const phi of latAngles) {
    for (const lam of lonAngles) {
      verts.push([Math.cos(phi)*Math.cos(lam), Math.sin(phi)*flattenY, Math.cos(phi)*Math.sin(lam)]);
    }
  }
  const idx = (lat, lon) => lat * nLon + lon;
  for (let i = 0; i < nLat; i++) {
    for (let j = 0; j < nLon; j++) {
      const a = idx(i,j)+1, b = idx(i,(j+1)%nLon)+1, c = idx(i+1,(j+1)%nLon)+1, d = idx(i+1,j)+1;
      faces.push([a,b,c]);
      faces.push([a,c,d]);
    }
  }
  const lines = [
    `# ${name} -- oblate spheroid a=1.0 b=${flattenY} (Saturn flattening f~0.098)`,
    'o oblate_spheroid', ''
  ];
  for (const [x,y,z] of verts) lines.push(vline(x,y,z));
  lines.push('');
  for (const [a,b,c] of faces) lines.push(`f ${a} ${b} ${c}`);
  lines.push('');
  fs.writeFileSync(path.join(OUT, name), lines.join('\n'));
  console.log(`  wrote ${name}  (${verts.length} verts, ${faces.length} tris)`);
}

// ── main ──────────────────────────────────────────────────────────────────────
const N = 24;
// Saturn-proportional inner radii (planet Req=5.8):
//   C  ring: inner=7.17  outer=8.86  scale=8.9  inner_r=0.809
//   B  ring: inner=8.86  outer=11.32 scale=11.3 inner_r=0.783
//   Cassini: inner=11.32 outer=11.76 scale=11.8 inner_r=0.963
//   A  ring: inner=11.76 outer=13.17 scale=13.2 inner_r=0.893
const rings = [
  ['ring_c', 0.809],
  ['ring_b', 0.783],
  ['ring_cassini', 0.963],
  ['ring_a', 0.893],
];

console.log('Generating ring OBJ files ...');
for (const [base, innerR] of rings) {
  writeRingHalf(`${base}_front.obj`, `${base}_front`, innerR, N, true);
  writeRingHalf(`${base}_back.obj`,  `${base}_back`,  innerR, N, false);
}

console.log('\nGenerating oblate spheroid ...');
writeOblate('planet_oblate.obj');
console.log('\nDone.');
