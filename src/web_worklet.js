
var audio_running = false;

function run_on_worklet() {
    class KAudioProcessor extends AudioWorkletProcessor {
        constructor(...args) {
            super(...args)
            this.port.onmessage = (e) => {
                console.log("NEW WORKLET");
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
                            console.log(item.name + "is unimplemented in audio worklet");
                        }
                    }

                    if (item.kind == "memory") {
                        imports[item.module][item.name] = {};
                    }
                });

                imports.wbg.memory = e.data.wasm_memory;

                WebAssembly.instantiate(e.data.wasm_module, imports).then(results => {
                    this.wasm_exports = results.exports;
                    this.wasm_memory = e.data.wasm_memory;
                    // Pass the callback function pointer back to the Wasm module.
                    // This time running within this AudioWorklet.
                    //  console.log(e.data.entry_point);
                    this.wasm_exports.__wbindgen_initialize_thread();
                    this.wasm_exports.kaudio_thread_initialize(e.data.entry_point);
                });
            }
        }

        process(inputs, outputs, parameters) {
            let channel_count = outputs[0].length;
            // console.log(channel_count);
            let frame_size = outputs[0][0].length; // It's probably fine to assume all channels have the same frame size. 
            this.wasm_exports.kaudio_run_callback(channel_count, frame_size, sampleRate);

            for (let i = 0; i < channel_count; i++) {
                let location = this.wasm_exports.kaudio_audio_buffer_location(i);
                let data = new Float32Array(this.wasm_memory.buffer, location, frame_size);
                outputs[0][i].set(data);
            }

            return true
        }
    }

    registerProcessor('kaudio-processor', KAudioProcessor);
}


export function setup_worklet(entry_point) {

    document.onpointerdown = (event) => {
        if (!audio_running) {
            setup_worklet();
            audio_running = true;
        }

        async function setup_worklet() {
            const audioContext = new AudioContext({ sampleRate: 44100 });

            var blobURL = URL.createObjectURL(new Blob(
                ['(', run_on_worklet.toString(), ')()'],
                { type: 'application/javascript' }
            ));

            await audioContext.audioWorklet.addModule(blobURL);
            URL.revokeObjectURL(blobURL);

            const worklet = new AudioWorkletNode(audioContext, 'kaudio-processor', {
                outputChannelCount: [2],
            });
            worklet.connect(audioContext.destination);
            let message = {};

            // Smuggling these values via document properties
            // is hack for now, but it requires a specific index.html setup
            // and should be replaced.
            message.wasm_memory = document.wasm_memory;
            message.wasm_module = document.wasm_module;
            message.entry_point = entry_point;

            worklet.port.postMessage(message);
        }

    };

}
