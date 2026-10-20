<template>
  <canvas
    ref="canvasRef"
    class="fixed inset-0 pointer-events-none"
    style="z-index: 61; width: 100vw; height: 100vh;"
  />
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue';
import { useTrackerStore } from '../stores/trackerStore';
import { cursorX, cursorY } from '../utils/cursor';

const store = useTrackerStore();
const canvasRef = ref<HTMLCanvasElement | null>(null);

let rafId = 0;

const rnd  = (a: number, b: number) => a + Math.random() * (b - a);
const rndI = (a: number, b: number) => Math.floor(rnd(a, b));
const lerp = (a: number, b: number, t: number) => a + (b - a) * t;

function priceColor(price: number): string {
  if (price >= 100) return '#fbbf24';
  if (price >= 20)  return '#a78bfa';
  if (price >= 5)   return '#60a5fa';
  return '#4ade80';
}

interface WebNode { x: number; y: number; vx: number; vy: number; age: number; maxAge: number; label: string; color: string; size: number; }
interface WebEdge { a: number; b: number; restLen: number; }
const web: { nodes: WebNode[]; edges: WebEdge[] } = { nodes: [], edges: [] };

function webSpawn(label: string, color: string) {
  const id = web.nodes.length;
  web.nodes.push({
    x: cursorX +rnd(-80, 80), y: cursorY +rnd(-80, 80),
    vx: rnd(-40, 40), vy: rnd(-40, 40),
    age: 0, maxAge: store.fxDuration * 60,
    label, color, size: 14 + label.length * 3,
  });
  const near = web.nodes.slice(0, id).sort((a, b) => {
    const da = Math.hypot(a.x - web.nodes[id].x, a.y - web.nodes[id].y);
    const db = Math.hypot(b.x - web.nodes[id].x, b.y - web.nodes[id].y);
    return da - db;
  }).slice(0, 3);
  near.forEach(n => {
    const ni = web.nodes.indexOf(n);
    const dx = web.nodes[id].x - n.x, dy = web.nodes[id].y - n.y;
    web.edges.push({ a: ni, b: id, restLen: Math.hypot(dx, dy) });
  });
}

function webUpdate(ctx: CanvasRenderingContext2D, _dt: number) {
  const k = 0.018, damp = 0.92, rep = 1800;
  web.nodes.forEach((n, i) => {
    n.age++;
    n.vx += (cursorX - n.x) * 0.002;
    n.vy += (cursorY - n.y) * 0.002;
    web.nodes.forEach((m, j) => {
      if (i === j) return;
      const dx = n.x - m.x, dy = n.y - m.y;
      const d2 = dx * dx + dy * dy + 1;
      n.vx += dx / d2 * rep;
      n.vy += dy / d2 * rep;
    });
    n.vx *= damp; n.vy *= damp;
    n.x += n.vx * 0.016;
    n.y += n.vy * 0.016;
  });
  web.edges.forEach(e => {
    const a = web.nodes[e.a], b = web.nodes[e.b];
    if (!a || !b) return;
    const dx = b.x - a.x, dy = b.y - a.y;
    const d = Math.hypot(dx, dy) + 0.1;
    const f = (d - e.restLen) * k;
    const nx2 = dx / d * f, ny2 = dy / d * f;
    a.vx += nx2; a.vy += ny2;
    b.vx -= nx2; b.vy -= ny2;
  });

  web.edges.forEach(e => {
    const a = web.nodes[e.a], b = web.nodes[e.b];
    if (!a || !b) return;
    const alpha = Math.min(1, 1 - b.age / b.maxAge) * 0.35;
    ctx.strokeStyle = `rgba(96,165,250,${alpha})`;
    ctx.lineWidth = 1;
    ctx.beginPath(); ctx.moveTo(a.x, a.y); ctx.lineTo(b.x, b.y); ctx.stroke();
  });
  web.nodes.forEach(n => {
    const alpha = Math.min(1, 1 - n.age / n.maxAge) * 0.2;
    ctx.strokeStyle = `rgba(96,165,250,${alpha})`;
    ctx.lineWidth = 0.5;
    ctx.beginPath(); ctx.moveTo(n.x, n.y); ctx.lineTo(cursorX, cursorY); ctx.stroke();
  });
  web.nodes.forEach(n => {
    const t = n.age / n.maxAge;
    const alpha = t < 0.1 ? t / 0.1 : t > 0.85 ? (1 - t) / 0.15 : 1;
    ctx.save();
    ctx.globalAlpha = alpha;
    ctx.fillStyle = 'rgba(10,8,20,0.85)';
    ctx.strokeStyle = n.color;
    ctx.lineWidth = 1.5;
    const hw = n.size / 2 + 4, hh = 10;
    ctx.beginPath();
    ctx.roundRect(n.x - hw, n.y - hh, hw * 2, hh * 2, 5);
    ctx.fill(); ctx.stroke();
    ctx.fillStyle = n.color;
    ctx.font = 'bold 11px "Segoe UI"';
    ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
    ctx.fillText(n.label, n.x, n.y);
    ctx.restore();
  });
  const alive = new Set<number>();
  web.nodes.forEach((n, i) => { if (n.age < n.maxAge) alive.add(i); });
  web.edges   = web.edges.filter(e => alive.has(e.a) && alive.has(e.b));
  web.nodes   = web.nodes.filter((_, i) => alive.has(i));
  const remap = new Map<number, number>();
  let ni2 = 0;
  for (let i = 0; i < web.nodes.length + web.edges.length; i++) {
    if (alive.has(i) && i < web.nodes.length + (alive.size)) remap.set(i, ni2++);
  }
}

interface ChainBead { x: number; y: number; vx: number; vy: number; label: string; color: string; age: number; maxAge: number; }
const chain: { beads: ChainBead[] } = { beads: [] };
const CHAIN_SEG = 42;

function chainSpawn(label: string, color: string) {
  const prev = chain.beads[chain.beads.length - 1];
  const startY = prev ? prev.y + CHAIN_SEG : cursorY +CHAIN_SEG;
  chain.beads.push({
    x: cursorX, y: startY, vx: 0, vy: 0,
    label, color, age: 0, maxAge: store.fxDuration * 60,
  });
}

function chainUpdate(ctx: CanvasRenderingContext2D) {
  const gravity = 0.5, damp = 0.88, iters = 8;
  chain.beads.forEach((b) => {
    b.age++;
    b.vy += gravity;
    b.vx *= damp; b.vy *= damp;
    b.x  += b.vx; b.y  += b.vy;
  });
  for (let it = 0; it < iters; it++) {
    chain.beads.forEach((b, i) => {
      const anchor = i === 0 ? { x: cursorX, y: cursorY } : chain.beads[i - 1];
      const dx = b.x - anchor.x, dy = b.y - anchor.y;
      const d  = Math.hypot(dx, dy) + 0.001;
      const diff = (d - CHAIN_SEG) / d;
      if (i === 0) {
        b.x -= dx * diff * 0.5; b.y -= dy * diff * 0.5;
      } else {
        b.x -= dx * diff * 0.5; b.y -= dy * diff * 0.5;
        anchor.x += dx * diff * 0.5; anchor.y += dy * diff * 0.5;
      }
    });
  }
  ctx.save();
  chain.beads.forEach((b, i) => {
    const t = b.age / b.maxAge;
    const alpha = t < 0.1 ? t / 0.1 : t > 0.85 ? (1 - t) / 0.15 : 1;
    const anchor = i === 0 ? { x: cursorX, y: cursorY } : chain.beads[i - 1];
    ctx.globalAlpha = alpha;
    ctx.strokeStyle = 'rgba(148,163,184,0.5)';
    ctx.lineWidth = 2;
    ctx.beginPath(); ctx.moveTo(anchor.x, anchor.y); ctx.lineTo(b.x, b.y); ctx.stroke();
    ctx.beginPath();
    ctx.arc(b.x, b.y, 18, 0, Math.PI * 2);
    ctx.fillStyle = 'rgba(15,12,25,0.9)';
    ctx.fill();
    ctx.strokeStyle = b.color; ctx.lineWidth = 1.5; ctx.stroke();
    ctx.fillStyle = b.color;
    ctx.font = 'bold 10px "Segoe UI"';
    ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
    ctx.fillText(b.label, b.x, b.y);
  });
  ctx.restore();
  chain.beads = chain.beads.filter(b => b.age < b.maxAge);
}

interface OrbitalItem { angle: number; speed: number; a: number; b: number; tilt: number; label: string; color: string; age: number; maxAge: number; }
const orbitals: OrbitalItem[] = [];

function orbitalSpawn(label: string, color: string) {
  orbitals.push({
    angle: Math.random() * Math.PI * 2,
    speed: rnd(0.008, 0.025) * (Math.random() < 0.5 ? 1 : -1),
    a: rnd(80, 200), b: rnd(40, 100),
    tilt: Math.random() * Math.PI * 2,
    label, color, age: 0, maxAge: store.fxDuration * 60,
  });
}

function orbitalUpdate(ctx: CanvasRenderingContext2D) {
  orbitals.forEach(o => {
    o.age++;
    o.angle += o.speed;
    const ox = Math.cos(o.angle) * o.a;
    const oy = Math.sin(o.angle) * o.b;
    const px = cursorX +Math.cos(o.tilt) * ox - Math.sin(o.tilt) * oy;
    const py = cursorY +Math.sin(o.tilt) * ox + Math.cos(o.tilt) * oy;
    const t = o.age / o.maxAge;
    const alpha = t < 0.1 ? t / 0.1 : t > 0.85 ? (1 - t) / 0.15 : 1;
    ctx.save();
    ctx.globalAlpha = alpha * 0.18;
    ctx.strokeStyle = o.color;
    ctx.lineWidth = 1;
    ctx.beginPath();
    for (let a2 = 0; a2 < Math.PI * 2; a2 += 0.05) {
      const ox2 = Math.cos(a2) * o.a, oy2 = Math.sin(a2) * o.b;
      const px2 = cursorX +Math.cos(o.tilt) * ox2 - Math.sin(o.tilt) * oy2;
      const py2 = cursorY +Math.sin(o.tilt) * ox2 + Math.cos(o.tilt) * oy2;
      if (a2 === 0) ctx.moveTo(px2, py2); else ctx.lineTo(px2, py2);
    }
    ctx.closePath(); ctx.stroke();
    ctx.restore();
    ctx.save();
    ctx.globalAlpha = alpha;
    ctx.fillStyle = 'rgba(10,8,20,0.9)';
    ctx.strokeStyle = o.color; ctx.lineWidth = 1.5;
    const hw = o.label.length * 4 + 8;
    ctx.beginPath(); ctx.roundRect(px - hw, py - 10, hw * 2, 20, 5); ctx.fill(); ctx.stroke();
    ctx.fillStyle = o.color;
    ctx.font = 'bold 10px "Segoe UI"';
    ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
    ctx.fillText(o.label, px, py);
    ctx.restore();
  });
  orbitals.splice(0, orbitals.length, ...orbitals.filter(o => o.age < o.maxAge));
}

interface Bubble { x: number; y: number; vx: number; vy: number; r: number; label: string; color: string; age: number; maxAge: number; popping: boolean; popT: number; }
const bubbles: Bubble[] = [];

function bubbleSpawn(label: string, color: string) {
  bubbles.push({
    x: cursorX +rnd(-60, 60), y: cursorY +rnd(40, 120),
    vx: rnd(-15, 15), vy: rnd(-30, -10),
    r: rnd(24, 38), label, color,
    age: 0, maxAge: store.fxDuration * 60,
    popping: false, popT: 0,
  });
}

function bubbleUpdate(ctx: CanvasRenderingContext2D) {
  bubbles.forEach(b => {
    b.age++;
    if (b.age >= b.maxAge && !b.popping) { b.popping = true; }
    if (b.popping) {
      b.popT += 0.12;
      if (b.popT > 1) { b.age = b.maxAge + 1; return; }
    }
    b.vy -= 0.12;
    b.vy += 0.06;
    b.vx *= 0.98; b.vy *= 0.98;
    b.x += b.vx * 0.016 * 60;
    b.y += b.vy * 0.016 * 60;
    bubbles.forEach(c => {
      if (c === b) return;
      const dx = b.x - c.x, dy = b.y - c.y;
      const d = Math.hypot(dx, dy);
      const minD = b.r + c.r;
      if (d < minD && d > 0.01) {
        const push = (minD - d) / d * 0.5;
        b.vx += dx * push * 0.3; b.vy += dy * push * 0.3;
      }
    });
  });
  bubbles.forEach(b => {
    if (b.age > b.maxAge) return;
    const t = b.age / b.maxAge;
    const baseAlpha = t < 0.1 ? t / 0.1 : t > 0.85 ? (1 - t) / 0.15 : 1;
    ctx.save();
    if (b.popping) {
      ctx.globalAlpha = (1 - b.popT) * 0.8;
      ctx.strokeStyle = b.color;
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.arc(b.x, b.y, b.r + b.popT * 20, 0, Math.PI * 2);
      ctx.stroke();
    } else {
      ctx.globalAlpha = baseAlpha * 0.85;
      const grad = ctx.createRadialGradient(b.x - b.r * 0.3, b.y - b.r * 0.3, b.r * 0.1, b.x, b.y, b.r);
      grad.addColorStop(0, 'rgba(255,255,255,0.15)');
      grad.addColorStop(1, 'rgba(10,8,20,0.1)');
      ctx.beginPath(); ctx.arc(b.x, b.y, b.r, 0, Math.PI * 2);
      ctx.fillStyle = grad; ctx.fill();
      ctx.strokeStyle = b.color; ctx.lineWidth = 1.5; ctx.stroke();
      ctx.beginPath();
      ctx.arc(b.x - b.r * 0.3, b.y - b.r * 0.35, b.r * 0.22, 0, Math.PI * 2);
      ctx.fillStyle = 'rgba(255,255,255,0.25)'; ctx.fill();
      ctx.fillStyle = '#fff';
      ctx.font = `bold ${Math.max(9, b.r * 0.45)}px "Segoe UI"`;
      ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
      ctx.fillText(b.label, b.x, b.y);
    }
    ctx.restore();
  });
  bubbles.splice(0, bubbles.length, ...bubbles.filter(b => b.age <= b.maxAge && b.popT <= 1));
}

interface Boid { x: number; y: number; vx: number; vy: number; label: string; color: string; age: number; maxAge: number; }
const boids: Boid[] = [];

function swarmSpawn(label: string, color: string) {
  const angle = Math.random() * Math.PI * 2;
  const dist  = rnd(200, 400);
  boids.push({
    x: cursorX +Math.cos(angle) * dist,
    y: cursorY +Math.sin(angle) * dist,
    vx: rnd(-30, 30), vy: rnd(-30, 30),
    label, color, age: 0, maxAge: store.fxDuration * 60,
  });
}

function swarmUpdate(ctx: CanvasRenderingContext2D) {
  const maxSpd = 3.5, sep = 40, cohR = 120;
  boids.forEach(b => {
    b.age++;
    let fx = 0, fy = 0;
    const dx = cursorX - b.x, dy = cursorY - b.y;
    const d = Math.hypot(dx, dy) + 0.1;
    fx += dx / d * 2.5;
    fy += dy / d * 2.5;
    boids.forEach(c => {
      if (c === b) return;
      const cdx = b.x - c.x, cdy = b.y - c.y;
      const cd  = Math.hypot(cdx, cdy) + 0.1;
      if (cd < sep) { fx += cdx / cd * 1.8; fy += cdy / cd * 1.8; }
      else if (cd < cohR) { fx -= cdx / cd * 0.4; fy -= cdy / cd * 0.4; }
    });
    b.vx += fx * 0.1; b.vy += fy * 0.1;
    const spd = Math.hypot(b.vx, b.vy);
    if (spd > maxSpd) { b.vx = b.vx / spd * maxSpd; b.vy = b.vy / spd * maxSpd; }
    b.x += b.vx; b.y += b.vy;
  });
  boids.forEach(b => {
    const t = b.age / b.maxAge;
    const alpha = t < 0.1 ? t / 0.1 : t > 0.85 ? (1 - t) / 0.15 : 1;
    const angle = Math.atan2(b.vy, b.vx);
    ctx.save();
    ctx.globalAlpha = alpha;
    ctx.translate(b.x, b.y); ctx.rotate(angle);
    ctx.beginPath();
    ctx.moveTo(14, 0); ctx.lineTo(-8, 7); ctx.lineTo(-4, 0); ctx.lineTo(-8, -7); ctx.closePath();
    ctx.fillStyle = b.color;
    ctx.globalAlpha = alpha * 0.9;
    ctx.fill();
    ctx.restore();
    ctx.save();
    ctx.globalAlpha = alpha;
    ctx.fillStyle = b.color;
    ctx.font = 'bold 10px "Segoe UI"';
    ctx.textAlign = 'center'; ctx.textBaseline = 'bottom';
    ctx.fillText(b.label, b.x, b.y - 12);
    ctx.restore();
  });
  boids.splice(0, boids.length, ...boids.filter(b => b.age < b.maxAge));
}

interface Ring { x: number; y: number; r: number; maxR: number; age: number; maxAge: number; color: string; label: string; labelShown: boolean; }
const rings: Ring[] = [];

function ringsSpawn(label: string, color: string) {
  const maxAge = store.fxDuration * 60;
  for (let i = 0; i < 3; i++) {
    rings.push({
      x: cursorX, y: cursorY, r: 0, maxR: rnd(80 + i * 60, 120 + i * 80),
      age: -i * 8, maxAge,
      color, label: i === 0 ? label : '', labelShown: false,
    });
  }
}

function ringsUpdate(ctx: CanvasRenderingContext2D) {
  rings.forEach(r => {
    r.age++;
    if (r.age < 0) return;
    const t = r.age / r.maxAge;
    r.r = r.maxR * (1 - Math.pow(1 - t, 2));
    const alpha = t < 0.15 ? t / 0.15 : 1 - t;
    ctx.save();
    ctx.globalAlpha = alpha * 0.7;
    ctx.strokeStyle = r.color;
    ctx.lineWidth   = 2 * (1 - t);
    ctx.shadowColor = r.color;
    ctx.shadowBlur  = 8;
    ctx.beginPath(); ctx.arc(r.x, r.y, r.r, 0, Math.PI * 2); ctx.stroke();
    ctx.restore();
    if (r.label && !r.labelShown && t > 0.08) {
      r.labelShown = true;
    }
    if (r.label && r.labelShown) {
      ctx.save();
      ctx.globalAlpha = Math.max(0, 1 - t * 1.2);
      ctx.fillStyle = r.color;
      ctx.font = 'bold 14px "Segoe UI"';
      ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
      ctx.fillText(r.label, r.x, r.y);
      ctx.restore();
    }
  });
  rings.splice(0, rings.length, ...rings.filter(r => r.age < r.maxAge));
}

interface Slash { age: number; maxAge: number; x1: number; y1: number; x2: number; y2: number; color: string; label: string; }
const slashes: Slash[] = [];

function slashSpawn(label: string, color: string) {
  const len = rnd(200, 350);
  const angle = rnd(-0.6, 0.6) + Math.PI * 0.25;
  slashes.push({
    age: 0, maxAge: 40,
    x1: cursorX - Math.cos(angle) * len / 2, y1: cursorY - Math.sin(angle) * len / 2,
    x2: cursorX +Math.cos(angle) * len / 2, y2: cursorY +Math.sin(angle) * len / 2,
    color, label,
  });
}

function slashUpdate(ctx: CanvasRenderingContext2D) {
  slashes.forEach(s => {
    s.age++;
    const t = s.age / s.maxAge;
    const progress = t < 0.3 ? t / 0.3 : 1;
    const alpha    = t < 0.3 ? 1 : 1 - (t - 0.3) / 0.7;
    ctx.save();
    ctx.globalAlpha = alpha;
    ctx.strokeStyle = s.color;
    ctx.lineWidth   = 3 * (1 - t * 0.5);
    ctx.shadowColor = s.color; ctx.shadowBlur = 12;
    ctx.beginPath();
    ctx.moveTo(s.x1, s.y1);
    ctx.lineTo(
      lerp(s.x1, s.x2, progress),
      lerp(s.y1, s.y2, progress),
    );
    ctx.stroke();
    if (t > 0.25) {
      ctx.fillStyle = s.color;
      ctx.font = 'bold 22px "Segoe UI Black"';
      ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
      ctx.fillText(s.label, (s.x1 + s.x2) / 2, (s.y1 + s.y2) / 2 - 20);
    }
    ctx.restore();
  });
  slashes.splice(0, slashes.length, ...slashes.filter(s => s.age < s.maxAge));
}

interface Shockwave { x: number; y: number; r: number; age: number; maxAge: number; color: string; label: string; }
const shockwaves: Shockwave[] = [];

function shockwaveSpawn(label: string, color: string) {
  shockwaves.push({ x: cursorX, y: cursorY, r: 0, age: 0, maxAge: 55, color, label });
}

function shockwaveUpdate(ctx: CanvasRenderingContext2D) {
  const W = canvasRef.value?.width ?? window.innerWidth;
  shockwaves.forEach(s => {
    s.age++;
    const t = s.age / s.maxAge;
    s.r = W * 0.6 * t;
    const alpha = 1 - t;
    ctx.save();
    ctx.globalAlpha = alpha * 0.6;
    ctx.strokeStyle = s.color;
    ctx.lineWidth   = 14 * (1 - t);
    ctx.shadowColor = s.color; ctx.shadowBlur = 20;
    ctx.beginPath(); ctx.arc(s.x, s.y, s.r, 0, Math.PI * 2); ctx.stroke();
    if (t < 0.3) {
      ctx.globalAlpha = 1 - t * 3;
      ctx.fillStyle   = s.color;
      ctx.font        = 'bold 28px "Segoe UI Black"';
      ctx.textAlign   = 'center'; ctx.textBaseline = 'middle';
      ctx.fillText(s.label, s.x, s.y);
    }
    ctx.restore();
  });
  shockwaves.splice(0, shockwaves.length, ...shockwaves.filter(s => s.age < s.maxAge));
}

interface Confetto { x: number; y: number; vx: number; vy: number; rot: number; vrot: number; color: string; size: number; age: number; maxAge: number; shape: number; }
const confetti: Confetto[] = [];

function confettiSpawn(_label: string, color: string) {
  for (let i = 0; i < 20; i++) {
    const angle = Math.random() * Math.PI * 2;
    const speed = rnd(3, 10);
    confetti.push({
      x: cursorX, y: cursorY,
      vx: Math.cos(angle) * speed, vy: Math.sin(angle) * speed - 4,
      rot: Math.random() * Math.PI * 2, vrot: rnd(-0.3, 0.3),
      color: i % 3 === 0 ? color : ['#60a5fa','#a78bfa','#fbbf24','#4ade80','#f472b6'][i % 5],
      size: rnd(5, 12), age: 0, maxAge: store.fxDuration * 60 * 0.6,
      shape: rndI(0, 3),
    });
  }
}

function confettiUpdate(ctx: CanvasRenderingContext2D) {
  confetti.forEach(c => {
    c.age++;
    c.vy += 0.25;
    c.vx *= 0.99; c.vy *= 0.99;
    c.x  += c.vx; c.y  += c.vy;
    c.rot += c.vrot;
    const t     = c.age / c.maxAge;
    const alpha = t < 0.1 ? t / 0.1 : t > 0.7 ? (1 - t) / 0.3 : 1;
    ctx.save();
    ctx.globalAlpha = alpha;
    ctx.fillStyle   = c.color;
    ctx.translate(c.x, c.y); ctx.rotate(c.rot);
    if (c.shape === 0) {
      ctx.fillRect(-c.size / 2, -c.size / 4, c.size, c.size / 2);
    } else if (c.shape === 1) {
      ctx.beginPath(); ctx.arc(0, 0, c.size / 2, 0, Math.PI * 2); ctx.fill();
    } else {
      ctx.beginPath();
      ctx.moveTo(0, -c.size / 2); ctx.lineTo(c.size / 2, c.size / 2);
      ctx.lineTo(-c.size / 2, c.size / 2); ctx.closePath(); ctx.fill();
    }
    ctx.restore();
  });
  confetti.splice(0, confetti.length, ...confetti.filter(c => c.age < c.maxAge));
}

interface LightningBolt { points: Array<{ x: number; y: number }>; age: number; maxAge: number; color: string; label: string; }
const lightnings: LightningBolt[] = [];

function buildBolt(x1: number, y1: number, x2: number, y2: number, depth: number): Array<{ x: number; y: number }> {
  if (depth === 0) return [{ x: x1, y: y1 }, { x: x2, y: y2 }];
  const mx = (x1 + x2) / 2 + rnd(-60, 60);
  const my = (y1 + y2) / 2 + rnd(-30, 30);
  return [...buildBolt(x1, y1, mx, my, depth - 1), ...buildBolt(mx, my, x2, y2, depth - 1)];
}

function lightningSpawn(label: string, color: string) {
  lightnings.push({
    points: buildBolt(cursorX + rnd(-80, 80), 0, cursorX, cursorY, 5),
    age: 0, maxAge: 35, color, label,
  });
}

function lightningUpdate(ctx: CanvasRenderingContext2D) {
  lightnings.forEach(l => {
    l.age++;
    const t     = l.age / l.maxAge;
    const alpha = t < 0.2 ? t / 0.2 : 1 - (t - 0.2) / 0.8;
    ctx.save();
    ctx.globalAlpha = alpha;
    ctx.strokeStyle = l.color;
    ctx.lineWidth   = 2;
    ctx.shadowColor = l.color; ctx.shadowBlur = 16;
    ctx.beginPath();
    l.points.forEach((p, i) => i === 0 ? ctx.moveTo(p.x, p.y) : ctx.lineTo(p.x, p.y));
    ctx.stroke();
    ctx.lineWidth = 0.5;
    ctx.strokeStyle = '#fff';
    ctx.shadowBlur  = 0;
    ctx.beginPath();
    l.points.forEach((p, i) => i === 0 ? ctx.moveTo(p.x, p.y) : ctx.lineTo(p.x, p.y));
    ctx.stroke();
    if (t < 0.5) {
      ctx.globalAlpha = alpha;
      ctx.fillStyle   = '#fff';
      ctx.font        = 'bold 24px "Segoe UI Black"';
      ctx.textAlign   = 'center'; ctx.textBaseline = 'top';
      ctx.fillText(l.label, cursorX, cursorY + 10);
    }
    ctx.restore();
  });
  lightnings.splice(0, lightnings.length, ...lightnings.filter(l => l.age < l.maxAge));
}

function spawnEffect(price: number, _name: string) {
  const label = `${price.toFixed(1)} FE`;
  const color = priceColor(price);
  const style = store.extraFxStyle;

  if (style === 'web')       { webSpawn(label, color); return; }
  if (style === 'chain')     { chainSpawn(label, color); return; }
  if (style === 'orbital')   { orbitalSpawn(label, color); return; }
  if (style === 'bubbles')   { bubbleSpawn(label, color); return; }
  if (style === 'swarm')     { swarmSpawn(label, color); return; }
  if (style === 'rings')     { ringsSpawn(label, color); return; }
  if (style === 'slash'  && price >= 30) { slashSpawn(label, color); return; }
  if (style === 'shockwave'  && price >= 50) { shockwaveSpawn(label, color); return; }
  if (style === 'confetti')  { confettiSpawn(label, color); return; }
  if (style === 'lightning'  && price >= 80) { lightningSpawn(label, color); return; }
}

const physicsStyles = new Set(['web','chain','orbital','bubbles','swarm','rings','slash','shockwave','confetti','lightning']);

function hasParticles(): boolean {
  return web.nodes.length > 0 || chain.beads.length > 0 || orbitals.length > 0 ||
    bubbles.length > 0 || boids.length > 0 || rings.length > 0 ||
    slashes.length > 0 || shockwaves.length > 0 || confetti.length > 0 || lightnings.length > 0;
}

function startLoop() {
  if (rafId === 0) rafId = requestAnimationFrame(loop);
}

function loop() {
  const canvas = canvasRef.value;
  if (!canvas) { rafId = requestAnimationFrame(loop); return; }

  const ctx = canvas.getContext('2d');
  if (!ctx) { rafId = requestAnimationFrame(loop); return; }

  const W = window.innerWidth, H = window.innerHeight;
  if (canvas.width !== W || canvas.height !== H) {
    canvas.width = W; canvas.height = H;
  }
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  if (physicsStyles.has(store.extraFxStyle)) {
    webUpdate(ctx, 0);
    chainUpdate(ctx);
    orbitalUpdate(ctx);
    bubbleUpdate(ctx);
    swarmUpdate(ctx);
    ringsUpdate(ctx);
    slashUpdate(ctx);
    shockwaveUpdate(ctx);
    confettiUpdate(ctx);
    lightningUpdate(ctx);
  }

  if (hasParticles()) {
    rafId = requestAnimationFrame(loop);
  } else {
    rafId = 0;
  }
}

watch(() => store.latestDrop, (drop) => {
  if (!drop || !store.shouldAnimateDrop(drop)) return;
  if (!physicsStyles.has(store.extraFxStyle)) return;
  spawnEffect(drop.price, drop.item_name);
  startLoop();
});

watch(() => store.extraFxStyle, () => {
  web.nodes = []; web.edges = [];
  chain.beads.length = 0;
  orbitals.length = 0;
  bubbles.length = 0;
  boids.length = 0;
  rings.length = 0;
  slashes.length = 0;
  shockwaves.length = 0;
  confetti.length = 0;
  lightnings.length = 0;
  const canvas = canvasRef.value;
  if (canvas) canvas.getContext('2d')?.clearRect(0, 0, canvas.width, canvas.height);
});

onMounted(() => {
});

onUnmounted(() => {
  if (rafId) cancelAnimationFrame(rafId);
});
</script>
