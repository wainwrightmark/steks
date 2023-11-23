
let media_stream = null;

export async function startVideo() {
    const aspect_ratio = window.innerWidth / window.innerHeight;
    const constraints = {
        audio: false,
        video: {
            width: { max: window.innerWidth },
            height: { max: window.innerHeight },
            aspect_ratio: { exact: aspect_ratio },
            resizeMode: "crop-and-scale",
            facingMode: "user"
        },
    };

    let media_promise = navigator.mediaDevices
        .getUserMedia(constraints);

    media_stream = await media_promise;

    const video = document.querySelector("#videoElement");
    video.srcObject = media_stream;
    video.onloadedmetadata = () => {
        video.play();
    };

    console.log("Starting video");

    return;
}

export function stopVideo() {

    if (!Object.is(media_stream, null)) {



        let tracks = media_stream.getTracks();

        console.log("Got tracks");

        if (!Object.is(tracks, null)) {
            for (const track of tracks) {
                track.stop();
            }
            console.info("Stopping video");
        }
        else {
            console.warn("Video tracks were null so could not be stopped");
        }
        media_stream = null;

        const video = document.querySelector("#videoElement");
        video.srcObject = null;

    }
    else {
        console.warn("Video is null so could not be stopped");
    }



}