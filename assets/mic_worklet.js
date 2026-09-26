// AudioWorklet processor for the singer's mic: slices the 128-sample render quanta into
// overlapping analysis frames (frame_size long, every hop_size samples) and transfers each
// frame to the main thread, where the Rust pitch detector runs. Loaded from a Blob URL by
// `src/mic/web.rs`; the processor name must match `PROCESSOR_NAME` there.
class KtvMicFrames extends AudioWorkletProcessor {
    constructor(options) {
        super();
        const { frame_size, hop_size } = options.processorOptions;
        this.frame = new Float32Array(frame_size);
        this.hop = hop_size;
        this.filled = 0;
    }

    process(inputs) {
        const channel = inputs[0] && inputs[0][0];
        if (!channel) return true;
        for (let i = 0; i < channel.length; i++) {
            this.frame[this.filled++] = channel[i];
            if (this.filled === this.frame.length) {
                const out = this.frame.slice();
                this.port.postMessage(out, [out.buffer]);
                this.frame.copyWithin(0, this.hop);
                this.filled -= this.hop;
            }
        }
        return true;
    }
}

registerProcessor('ktv-mic-frames', KtvMicFrames);
