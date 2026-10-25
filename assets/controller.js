// MAZ Launcher – Controller-Modus.
// Steuerkreuz / linker Stick = Fokus bewegen, A = auswählen, B = zurück/schließen,
// LB / RB = Seite blättern. Läuft nur im Fenster, liest nur den Controller aus.
(function () {
  window.__mazPadOn = __ON__;
  document.documentElement.classList.toggle("controller-mode", window.__mazPadOn);
  if (window.__mazPad) return;
  window.__mazPad = true;

  var SEL = 'button:not([disabled]),a[href],input:not([type=hidden]):not([disabled]),select,textarea,[tabindex]:not([tabindex="-1"])';
  var last = {};
  var lastMove = 0;

  function visible(e) {
    var r = e.getBoundingClientRect();
    return r.width > 0 && r.height > 0 && r.bottom > 0 && r.top < window.innerHeight;
  }

  function scope() {
    var dialogs = document.querySelectorAll("[data-dialog]");
    return dialogs.length ? dialogs[dialogs.length - 1] : document;
  }

  function move(dx, dy) {
    var items = Array.prototype.slice.call(scope().querySelectorAll(SEL)).filter(visible);
    if (!items.length) return;
    var cur = document.activeElement;
    if (!cur || items.indexOf(cur) < 0) {
      items[0].focus();
      return;
    }
    var a = cur.getBoundingClientRect();
    var ax = a.left + a.width / 2, ay = a.top + a.height / 2;
    var best = null, bestScore = Infinity;
    items.forEach(function (e) {
      if (e === cur) return;
      var b = e.getBoundingClientRect();
      var vx = b.left + b.width / 2 - ax, vy = b.top + b.height / 2 - ay;
      var main = dx ? vx * dx : vy * dy;
      if (main <= 4) return;
      var side = dx ? Math.abs(vy) : Math.abs(vx);
      var score = main + side * 2.5;
      if (score < bestScore) { bestScore = score; best = e; }
    });
    if (best) {
      best.focus();
      best.scrollIntoView({ block: "nearest", inline: "nearest" });
    }
  }

  function back() {
    var close = document.querySelectorAll("[data-dialog-close]");
    if (close.length) close[close.length - 1].click();
  }

  function page(dir) {
    var m = document.activeElement && document.activeElement.closest("main, [data-dialog]");
    (m || document.querySelector("main") || document.scrollingElement).scrollBy({ top: dir * 480, behavior: "smooth" });
  }

  function loop(ts) {
    requestAnimationFrame(loop);
    if (!window.__mazPadOn || !navigator.getGamepads) return;
    var pads = navigator.getGamepads();
    var p = null;
    for (var i = 0; i < pads.length; i++) if (pads[i]) { p = pads[i]; break; }
    if (!p) return;
    var b = function (n) { return !!(p.buttons[n] && p.buttons[n].pressed); };
    var x = p.axes[0] || 0, y = p.axes[1] || 0;
    var st = {
      up: b(12) || y < -0.6, down: b(13) || y > 0.6, left: b(14) || x < -0.6, right: b(15) || x > 0.6,
      a: b(0), b: b(1), lb: b(4), rb: b(5)
    };
    var repeat = ts - lastMove > 170;
    Object.keys(st).forEach(function (k) {
      var dir = k === "up" || k === "down" || k === "left" || k === "right";
      if (!st[k] || (last[k] && !(dir && repeat))) return;
      if (dir) lastMove = ts;
      if (k === "up") move(0, -1);
      else if (k === "down") move(0, 1);
      else if (k === "left") move(-1, 0);
      else if (k === "right") move(1, 0);
      else if (k === "a") { var e = document.activeElement; if (e && e !== document.body) e.click(); }
      else if (k === "b") back();
      else if (k === "lb") page(-1);
      else if (k === "rb") page(1);
    });
    last = st;
  }
  requestAnimationFrame(loop);
})();
