class KAudioProcessor extends AudioWorkletProcessor {
    constructor(...args) {
        super(...args)
        this.port.onmessage = (e) => {
            let imports = {};
            // wbg is declared for compatibility with WasmBindgen.
            // I'm not sure how necessary it is that this exact name be used
            // for storing memory within Wasm.
            // More info is needed.
            imports.wbg = {};
            WebAssembly.Module.imports(e.data.wasm_module).forEach(item => {
                if (imports[item.module] === undefined) {
                    imports[item.module] = {};
                }

                if (item.kind == "function") {
                    imports[item.module][item.name] = function () {
                        console.log("Unimplemented in audio worklet");
                    }
                }

                if (item.kind == "memory") {
                    imports[item.module][item.name] = {};
                }
            });

            imports.env.memory = e.data.wasm_memory;

            WebAssembly.instantiate(e.data.wasm_module, imports).then(results => {
                console.log(results);
                console.log(e.data.wasm_memory);

                this.wasm_exports = results.exports;
                this.wasm_memory = e.data.wasm_memory;
            });
        }
    }

    process(inputs, outputs, parameters) {
        let channel_count = outputs[0].length;
        let frame_size = outputs[0][0].length; // It's probably fine to assume all channels have the same frame size. 
        this.wasm_exports.kaudio_run_callback(channel_count, frame_size, sampleRate)

        // console.log(frame_size);
        for (let i = 0; i < channel_count; i++) {
            let location = this.wasm_exports.kaudio_audio_buffer_location(i)
            //console.log(location);
            let data = new Float32Array(this.wasm_memory.buffer, location, frame_size);
            //console.log(data);
            outputs[0][i].set(data);
        }

        return true
    }
}

registerProcessor('kaudio-processor', KAudioProcessor)