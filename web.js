let touch_added = false;
let touch_events = [];

export let on_start = () => {
  var spinner = window.document.getElementById("spinner");
  window.document.body.removeChild(spinner);
};

export let has_touch = () => {
  return !!("ontouchstart" in window);
};

export let pop_touch_event = () => {
  let e = touch_events.shift();
  return e;
};

export let request_fullscreen = () => {
  var doc = window.document;
  var docEl = doc.getElementById("game");

  var requestFullScreen =
    docEl.requestFullscreen ||
    docEl.mozRequestFullScreen ||
    docEl.webkitRequestFullScreen ||
    docEl.msRequestFullscreen;
  var cancelFullScreen =
    doc.exitFullscreen ||
    doc.mozCancelFullScreen ||
    doc.webkitExitFullscreen ||
    doc.msExitFullscreen;

  if (
    !doc.fullscreenElement &&
    !doc.mozFullScreenElement &&
    !doc.webkitFullscreenElement &&
    !doc.msFullscreenElement
  ) {
    requestFullScreen.call(docEl, { navigationUI: "hide" });
  } else {
    cancelFullScreen.call(doc);
  }
};

export let enable_touch = () => {
  if (has_touch() == true && touch_added == false) {
    let canvas = document.getElementById("game");
    canvas.addEventListener(
      "touchstart",
      (ev) => {
        ev.preventDefault();
        touch_events.push(ev);
      },
      { passive: false }
    );
    canvas.addEventListener(
      "touchend",
      (ev) => {
        ev.preventDefault();
        touch_events.push(ev);
      },
      { passive: false }
    );
    canvas.addEventListener(
      "touchmove",
      (ev) => {
        ev.preventDefault();
        touch_events.push(ev);
      },
      { passive: false }
    );

    touch_added = true;
  }
};

export let resize_canvas = (width, height) => {
  let canvas = document.getElementById("game");
  canvas.width = width * window.devicePixelRatio;
  canvas.height = height * window.devicePixelRatio;
  canvas.style = `width: ${width}; height: ${height}`;
};
