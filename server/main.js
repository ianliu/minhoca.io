// const configuration = { 
//   "iceServers": [{ "urls": "stun:stun2.1.google.com:19302" }] 
// };

const configuration = {};
const pc1 = new RTCPeerConnection(configuration);
const pc2 = new RTCPeerConnection(configuration);

pc1.onicecandidate = ev => console.log({ pc1_ice_candidate: ev });
pc2.onicecandidate = ev => console.log({ pc2_ice_candidate: ev });

async function main() {
  const chan1 = pc1.createDataChannel("test", {reliable: true});
  const chan2 = pc2.createDataChannel("test", {reliable: true});
  console.log({ chan1, chan2 });

  const offer = await pc1.createOffer();
  console.log({ offer });

  try {
    await pc1.setLocalDescription(offer);
  } catch(error) {
    console.log({msg: "setLocalDescription offer", error});
  }

  try {
    await pc2.setRemoteDescription(offer);
  } catch(error) {
    console.log({msg: "setRemoteDescription offer", error});
  }

  const answer = await pc2.createAnswer();
  console.log({ answer });

  try {
    await pc1.setRemoteDescription(answer);
  } catch(error) {
    console.log({msg: "setLocalDescription answer", error});
  }

  try {
    await pc2.setLocalDescription(answer);
  } catch(error) {
    console.log({msg: "setLocalDescription answer", error});
  }
}

main();
