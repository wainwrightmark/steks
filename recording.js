let user_recorder = null;
let canvas_recorder = null;
let user_chunks = [];
let canvas_chunks = [];

export async function start_recording() {
  console.log("Start Recording");

  const mediaConstraints = {
    video: {
      facingMode: "user",
    },
    audio: {
      // echoCancellation: true,
      // noiseSuppression: true,
      sampleRate: 44100,
    },
  };

  try {

    const canvas = document.getElementById("game");
    const canvas_stream = canvas.captureStream(30);
    canvas_recorder = new MediaRecorder(canvas_stream);

    canvas_recorder.ondataavailable = (event) => {
      if (event.data.size > 0) {
        canvas_chunks.push(event.data);
        console.log("Canvas Data Found");
      } else {
        console.log("No Canvas Data Found");
      }
    };
    canvas_recorder.onstop = () => {
      const blob = new Blob(canvas_chunks, {
        type: "video/webm;codecs=vp9",
      });

      canvas_chunks = [];
      console.log("Canvas Recording Stopped");
      saveFile("steks_canvas_recording.mpeg", blob);
    };


    const promise_user = navigator.mediaDevices.getUserMedia(mediaConstraints);

    let user_stream = await promise_user;
    let video = document.getElementById("video");
    video.removeAttribute("hidden");
    video.srcObject = user_stream;

    //canvas_stream.getTracks().forEach((track) => user_stream.addTrack(track));

    user_recorder = new MediaRecorder(user_stream);

    console.log("Created Recorder");

    user_recorder.ondataavailable = (event) => {
      if (event.data.size > 0) {
        user_chunks.push(event.data);
        console.log("User Data Found");
      } else {
        console.log("No User Data Found");
      }
    };

    user_recorder.onstop = () => {
      let video = document.getElementById("video");
      video.setAttribute("hidden", "");

      const blob = new Blob(user_chunks, {
        type: "video/webm;codecs=vp9",
      });

      user_chunks = [];
      console.log("User Recording Stopped");
      saveFile("steks_user_recording.mpeg", blob);
    };




    canvas_recorder.start(200);
    user_recorder.start(200);

  } catch (error) {
    console.error(error);
  }
}

export function stop_recording() {
  console.log("Stop Recording");
  if (user_recorder != null) {
    user_recorder.stream.getTracks().forEach((track) => track.stop());
  } else {
    console.warn("No User Recorder to stop");
  }

  if (canvas_recorder != null) {
    canvas_recorder.stream.getTracks().forEach((track) => track.stop());
  } else {
    console.warn("No Canvas Recorder to stop");
  }
}

function saveFile(filename, blob) {
  if (window.navigator.msSaveOrOpenBlob) {
    window.navigator.msSaveOrOpenBlob(blob, filename);
  } else {
    const a = document.createElement("a");
    document.body.appendChild(a);
    const url = window.URL.createObjectURL(blob);
    a.href = url;
    a.download = filename;
    a.click();
    setTimeout(() => {
      window.URL.revokeObjectURL(url);
      document.body.removeChild(a);
    }, 0);
  }
}
