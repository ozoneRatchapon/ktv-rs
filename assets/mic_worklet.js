// AudioWorklet processor for the singer's mic: slices the 128-sample render quanta into
// overlapping analysis frames (frame_size long, every hop_size samples) and transfers each
// frame to the main thread, where the Rust pitch detector runs. Loaded from a Blob URL by
// `src/mic/web.rs`; the processor name must match `PROCESSOR_NAME` there.
// `channels: 2` (duet on a stereo receiver): each message holds both channels' frames back to back
// (left then right); a right channel the device does not have is sent as silence.
class KtvMicFrames extends AudioWorkletProcessor {
    constructor(options) {
        super();
        const { frame_size, hop_size, channels = 1 } = options.processorOptions;
        this.frames = Array.from({ length: channels }, () => new Float32Array(frame_size));
        this.hop = hop_size;
        this.filled = 0;
    }

    process(inputs) {
        const input = inputs[0];
        if (!input || !input[0]) return true;
        const length = input[0].length;
        const size = this.frames[0].length;
        for (let i = 0; i < length; i++) {
            for (let c = 0; c < this.frames.length; c++) this.frames[c][this.filled] = input[c] ? input[c][i] : 0;
            this.filled++;
            if (this.filled === size) {
                const out = new Float32Array(size * this.frames.length);
                this.frames.forEach((frame, c) => {
                    out.set(frame, c * size);
                    frame.copyWithin(0, this.hop);
                });
                this.port.postMessage(out, [out.buffer]);
                this.filled -= this.hop;
            }
        }
        return true;
    }
}

registerProcessor('ktv-mic-frames', KtvMicFrames);
