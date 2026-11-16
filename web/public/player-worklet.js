class PlayerProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.queue = [];
    this.offset = 0;
    this.frames = 0;
    this.port.onmessage = (event) => {
      if (event.data && event.data.type === "pcm") {
        this.queue.push(event.data.buffer);
        this.frames += event.data.buffer.length / 2;
      }
    };
  }

  process(_inputs, outputs) {
    const left = outputs[0] && outputs[0][0];
    const right = outputs[0] && outputs[0][1];
    if (!left) return true;
    for (let i = 0; i < left.length; i += 1) {
      while (this.queue.length && this.offset >= this.queue[0].length) {
        this.queue.shift();
        this.offset = 0;
      }
      if (!this.queue.length) {
        left[i] = 0;
        if (right) right[i] = 0;
        continue;
      }
      const buffer = this.queue[0];
      left[i] = buffer[this.offset];
      if (right) right[i] = buffer[this.offset + 1];
      this.offset += 2;
      this.frames -= 1;
    }
    this.port.postMessage({ type: "queued", frames: Math.max(0, this.frames) });
    return true;
  }
}

registerProcessor("player", PlayerProcessor);
