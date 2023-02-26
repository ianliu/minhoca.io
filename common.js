function log(obj) {
  document.getElementById("log").textContent += JSON.stringify(obj) + "\n";
}

const pc = new RTCPeerConnection({
  iceServers: [ { urls: "stun:stun.l.google.com:19302" } ]
});

let chan = null;

let my_payload = {description: null, candidates: []};

pc.onicecandidate = event => {
  if (event.candidate) {
    log(event.candidate);
    my_payload.candidates.push(event.candidate.toJSON());
    setValue("my_payload", my_payload);
  }
};

function setValue(id, obj) {
  document.getElementById(id).textContent = JSON.stringify(obj);
}

function getValue(id) {
  return JSON.parse(document.getElementById(id).value);
}

async function addCandidates(candidates) {
  for (const candidate of candidates)
    await pc.addIceCandidate(candidate);
}

function setup_channel(who, chan) {
  chan.onopen = event => log({ onopen: event });
  chan.onclose = event => log({ onclose: event });
  chan.onmessage = event => {
    const chatlog = document.getElementById("chatlog");
    chatlog.textContent += who + ": " + event.data + "\n";
  };
}
