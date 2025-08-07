// import { useState, useEffect } from "react";
// import { invoke } from "@tauri-apps/api/core";
// import { listen } from "@tauri-apps/api/event";
// import { StepGrid } from "./StepGrid";
// import { TransportControls } from "./TransportControls";

// export function DrumMachinePage() {
//   const [audioPaused, setAudioPaused] = useState(true);
//   const [bpm, setBpm] = useState(120);
//   const [kickPattern, setKickPattern] = useState([
//     true,
//     false,
//     false,
//     false,
//     false,
//     false,
//     true,
//     false,
//     false,
//     false,
//     false,
//     false,
//     false,
//     false,
//     true,
//     false,
//   ]);
//   const [status, setStatus] = useState("");
//   const [currentKickStep, setCurrentKickStep] = useState(0);
//   const [currentClapStep, setCurrentClapStep] = useState(0);
//   const [modulatorValues, setModulatorValues] = useState({
//     delayTime: 0.25,
//     reverbSize: 0.5,
//     reverbDecay: 0.5,
//   });
//   const [delaySend, setDelaySend] = useState(0.2);
//   const [reverbSend, setReverbSend] = useState(0.3);
//   const [delayReturn, setDelayReturn] = useState(0.8);
//   const [reverbReturn, setReverbReturn] = useState(0.6);
//   const [delayFreeze, setDelayFreeze] = useState(false);
//   const [kickAttack, setKickAttack] = useState(0.005);
//   const [kickRelease, setKickRelease] = useState(0.2);

//   // Clap pattern state
//   const [clapPattern, setClapPattern] = useState([
//     false,
//     false,
//     false,
//     false,
//     true,
//     false,
//     false,
//     false,
//     false,
//     false,
//     false,
//     false,
//     true,
//     false,
//     false,
//     false,
//   ]);
//   const [clapAttack, setClapAttack] = useState(0.01);
//   const [clapRelease, setClapRelease] = useState(0.15);

//   // Generation parameters
//   const [markovDensity, setMarkovDensity] = useState(0.3);
//   const [kickLoopBias, setKickLoopBias] = useState(0.5);
//   const [clapLoopBias, setClapLoopBias] = useState(0.5);

//   // Volume controls
//   const [kickVolume, setKickVolume] = useState(0.8);
//   const [clapVolume, setClapVolume] = useState(0.6);

//   // Listen for events from audio thread
//   useEffect(() => {
//     let kickStepUnlisten: (() => void) | null = null;
//     let clapStepUnlisten: (() => void) | null = null;
//     let modulatorUnlisten: (() => void) | null = null;
//     let kickPatternUnlisten: (() => void) | null = null;
//     let clapPatternUnlisten: (() => void) | null = null;

//     const setupListeners = async () => {
//       try {
//         // Listen for step changes
//         kickStepUnlisten = await listen<number>(
//           "kick_step_changed",
//           (event) => {
//             setCurrentKickStep(event.payload);
//           },
//         );

//         clapStepUnlisten = await listen<number>(
//           "clap_step_changed",
//           (event) => {
//             setCurrentClapStep(event.payload);
//           },
//         );

//         // Listen for modulator value updates
//         modulatorUnlisten = await listen<[number, number, number]>(
//           "modulator_values",
//           (event) => {
//             const [delayTime, reverbSize, reverbDecay] = event.payload;
//             setModulatorValues({ delayTime, reverbSize, reverbDecay });
//           },
//         );

//         // Listen for generated kick patterns
//         kickPatternUnlisten = await listen<boolean[]>(
//           "kick_pattern_generated",
//           (event) => {
//             setKickPattern(event.payload);
//           },
//         );

//         // Listen for generated clap patterns
//         clapPatternUnlisten = await listen<boolean[]>(
//           "clap_pattern_generated",
//           (event) => {
//             setClapPattern(event.payload);
//           },
//         );
//       } catch (error) {
//         console.error("Error setting up event listeners:", error);
//       }
//     };

//     setupListeners();

//     return () => {
//       if (kickStepUnlisten) kickStepUnlisten();
//       if (clapStepUnlisten) clapStepUnlisten();
//       if (modulatorUnlisten) modulatorUnlisten();
//       if (kickPatternUnlisten) kickPatternUnlisten();
//       if (clapPatternUnlisten) clapPatternUnlisten();
//     };
//   }, []);

//   async function updateKickPattern(newPattern: boolean[]) {
//     setKickPattern(newPattern);
//     try {
//       await invoke("set_sequence", {
//         systemName: "drum_machine",
//         sequenceData: { kick_pattern: newPattern },
//       });
//     } catch (error) {
//       setStatus(`Error setting kick pattern: ${error}`);
//     }
//   }

//   function toggleKickStep(index: number) {
//     const newPattern = [...kickPattern];
//     newPattern[index] = !newPattern[index];
//     updateKickPattern(newPattern);
//   }

//   async function updateDelaySend(value: number) {
//     setDelaySend(value);
//     try {
//       await invoke("send_audio_event", {
//         systemName: "drum_machine",
//         nodeName: "system",
//         eventName: "set_delay_send",
//         parameter: value,
//       });
//     } catch (error) {
//       setStatus(`Error setting delay send: ${error}`);
//     }
//   }

//   async function updateReverbSend(value: number) {
//     setReverbSend(value);
//     try {
//       await invoke("send_audio_event", {
//         systemName: "drum_machine",
//         nodeName: "system",
//         eventName: "set_reverb_send",
//         parameter: value,
//       });
//     } catch (error) {
//       setStatus(`Error setting reverb send: ${error}`);
//     }
//   }

//   async function updateDelayReturn(value: number) {
//     setDelayReturn(value);
//     try {
//       await invoke("send_audio_event", {
//         systemName: "drum_machine",
//         nodeName: "system",
//         eventName: "set_delay_return",
//         parameter: value,
//       });
//     } catch (error) {
//       setStatus(`Error setting delay return: ${error}`);
//     }
//   }

//   async function updateReverbReturn(value: number) {
//     setReverbReturn(value);
//     try {
//       await invoke("send_audio_event", {
//         systemName: "drum_machine",
//         nodeName: "system",
//         eventName: "set_reverb_return",
//         parameter: value,
//       });
//     } catch (error) {
//       setStatus(`Error setting reverb return: ${error}`);
//     }
//   }

//   async function updateDelayFreeze(freeze: boolean) {
//     setDelayFreeze(freeze);
//     try {
//       await invoke("send_audio_event", {
//         systemName: "drum_machine",
//         nodeName: "delay",
//         eventName: "set_freeze",
//         parameter: freeze ? 1.0 : 0.0,
//       });
//     } catch (error) {
//       setStatus(`Error setting delay freeze: ${error}`);
//     }
//   }

//   async function updateKickAttack(value: number) {
//     setKickAttack(value);
//     try {
//       await invoke("send_audio_event", {
//         systemName: "drum_machine",
//         nodeName: "kick",
//         eventName: "set_attack",
//         parameter: value,
//       });
//     } catch (error) {
//       setStatus(`Error setting kick attack: ${error}`);
//     }
//   }

//   async function updateKickRelease(value: number) {
//     setKickRelease(value);
//     try {
//       await invoke("send_audio_event", {
//         systemName: "drum_machine",
//         nodeName: "kick",
//         eventName: "set_release",
//         parameter: value,
//       });
//     } catch (error) {
//       setStatus(`Error setting kick release: ${error}`);
//     }
//   }

//   async function updateClapPattern(newPattern: boolean[]) {
//     setClapPattern(newPattern);
//     try {
//       await invoke("set_sequence", {
//         systemName: "drum_machine",
//         sequenceData: { clap_pattern: newPattern },
//       });
//     } catch (error) {
//       setStatus(`Error setting clap pattern: ${error}`);
//     }
//   }

//   function toggleClapStep(index: number) {
//     const newPattern = [...clapPattern];
//     newPattern[index] = !newPattern[index];
//     updateClapPattern(newPattern);
//   }

//   async function updateMarkovDensity(value: number) {
//     setMarkovDensity(value);
//     try {
//       await invoke("send_audio_event", {
//         systemName: "drum_machine",
//         nodeName: "clap",
//         eventName: "set_density",
//         parameter: value,
//       });
//     } catch (error) {
//       setStatus(`Error setting markov density: ${error}`);
//     }
//   }

//   async function generateKickPattern() {
//     try {
//       await invoke("send_audio_event", {
//         systemName: "drum_machine",
//         nodeName: "system",
//         eventName: "generate_kick_pattern",
//         parameter: 0.0,
//       });
//     } catch (error) {
//       setStatus(`Error generating kick pattern: ${error}`);
//     }
//   }

//   async function generateClapPattern() {
//     try {
//       await invoke("send_audio_event", {
//         systemName: "drum_machine",
//         nodeName: "system",
//         eventName: "generate_clap_pattern",
//         parameter: 0.0,
//       });
//     } catch (error) {
//       setStatus(`Error generating clap pattern: ${error}`);
//     }
//   }

//   async function updateKickLoopBias(value: number) {
//     setKickLoopBias(value);
//     try {
//       await invoke("send_audio_event", {
//         systemName: "drum_machine",
//         nodeName: "system",
//         eventName: "set_kick_loop_bias",
//         parameter: value,
//       });
//     } catch (error) {
//       setStatus(`Error setting kick loop bias: ${error}`);
//     }
//   }

//   async function updateClapLoopBias(value: number) {
//     setClapLoopBias(value);
//     try {
//       await invoke("send_audio_event", {
//         systemName: "drum_machine",
//         nodeName: "system",
//         eventName: "set_clap_loop_bias",
//         parameter: value,
//       });
//     } catch (error) {
//       setStatus(`Error setting clap loop bias: ${error}`);
//     }
//   }

//   async function updateKickVolume(value: number) {
//     setKickVolume(value);
//     try {
//       await invoke("send_audio_event", {
//         systemName: "drum_machine",
//         nodeName: "kick",
//         eventName: "set_gain",
//         parameter: value,
//       });
//     } catch (error) {
//       setStatus(`Error setting kick volume: ${error}`);
//     }
//   }

//   async function updateClapVolume(value: number) {
//     setClapVolume(value);
//     try {
//       await invoke("send_audio_event", {
//         systemName: "drum_machine",
//         nodeName: "clap",
//         eventName: "set_gain",
//         parameter: value,
//       });
//     } catch (error) {
//       setStatus(`Error setting clap volume: ${error}`);
//     }
//   }

//   async function updateClapAttack(value: number) {
//     setClapAttack(value);
//     try {
//       await invoke("send_audio_event", {
//         systemName: "drum_machine",
//         nodeName: "clap",
//         eventName: "set_attack",
//         parameter: value,
//       });
//     } catch (error) {
//       setStatus(`Error setting clap attack: ${error}`);
//     }
//   }

//   async function updateClapRelease(value: number) {
//     setClapRelease(value);
//     try {
//       await invoke("send_audio_event", {
//         systemName: "drum_machine",
//         nodeName: "clap",
//         eventName: "set_release",
//         parameter: value,
//       });
//     } catch (error) {
//       setStatus(`Error setting clap release: ${error}`);
//     }
//   }

//   return (
//     <>
//       {/* Transport Controls */}
//       <div className="mb-6">
//         <TransportControls
//           systemName="drum_machine"
//           isPaused={audioPaused}
//           bpm={bpm}
//           onPausedChange={setAudioPaused}
//           onBpmChange={setBpm}
//           sliderWidth="w-64"
//         />
//       </div>

//       {/* Pattern Grids */}
//       <div className="grid grid-cols-2 gap-8 mb-8">
//         <div>
//           <h3 className="text-lg mb-4">Kick Pattern</h3>
//           <StepGrid
//             pattern={kickPattern}
//             onStepToggle={toggleKickStep}
//             currentStep={currentKickStep}
//           />
//         </div>

//         <div>
//           <h3 className="text-lg mb-4">Clap Pattern</h3>
//           <StepGrid
//             pattern={clapPattern}
//             onStepToggle={toggleClapStep}
//             currentStep={currentClapStep}
//           />
//         </div>
//       </div>

//       {/* Pattern Generation Controls */}
//       <div className="bg-gray-800 p-6 rounded-md mb-8">
//         <h3 className="text-lg mb-4">Pattern Generation</h3>

//         <div className="mb-4">
//           <label className="block text-sm mb-2">Markov Density</label>
//           <div className="flex items-center gap-4">
//             <input
//               type="range"
//               min="0"
//               max="1"
//               step="0.01"
//               value={markovDensity}
//               onChange={(e) => updateMarkovDensity(parseFloat(e.target.value))}
//               className="w-64"
//             />
//             <span className="text-sm">{(markovDensity * 100).toFixed(0)}%</span>
//           </div>
//         </div>

//         <div className="flex gap-4">
//           <button
//             onClick={generateKickPattern}
//             className="px-4 py-2 bg-red-600 hover:bg-red-700 rounded"
//           >
//             Generate Kick Pattern
//           </button>
//           <button
//             onClick={generateClapPattern}
//             className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded"
//           >
//             Generate Clap Pattern
//           </button>
//         </div>
//       </div>

//       {/* Controls Grid */}
//       <div className="grid grid-cols-3 gap-8">
//         {/* Instrument Controls */}
//         <div className="bg-gray-800 p-6 rounded-md">
//           <h3 className="text-lg mb-4">Instruments</h3>

//           {/* Kick Controls */}
//           <div className="mb-6">
//             <h4 className="text-md mb-3 text-red-400">Kick</h4>
//             <div className="space-y-3">
//               <div>
//                 <label className="block text-sm mb-1">Volume</label>
//                 <input
//                   type="range"
//                   min="0"
//                   max="1"
//                   step="0.01"
//                   value={kickVolume}
//                   onChange={(e) => updateKickVolume(parseFloat(e.target.value))}
//                   className="w-full"
//                 />
//               </div>
//               <div>
//                 <label className="block text-sm mb-1">Attack</label>
//                 <input
//                   type="range"
//                   min="0.001"
//                   max="0.1"
//                   step="0.001"
//                   value={kickAttack}
//                   onChange={(e) => updateKickAttack(parseFloat(e.target.value))}
//                   className="w-full"
//                 />
//               </div>
//               <div>
//                 <label className="block text-sm mb-1">Release</label>
//                 <input
//                   type="range"
//                   min="0.05"
//                   max="1"
//                   step="0.01"
//                   value={kickRelease}
//                   onChange={(e) =>
//                     updateKickRelease(parseFloat(e.target.value))
//                   }
//                   className="w-full"
//                 />
//               </div>
//             </div>
//           </div>

//           {/* Clap Controls */}
//           <div>
//             <h4 className="text-md mb-3 text-blue-400">Clap</h4>
//             <div className="space-y-3">
//               <div>
//                 <label className="block text-sm mb-1">Volume</label>
//                 <input
//                   type="range"
//                   min="0"
//                   max="1"
//                   step="0.01"
//                   value={clapVolume}
//                   onChange={(e) => updateClapVolume(parseFloat(e.target.value))}
//                   className="w-full"
//                 />
//               </div>
//               <div>
//                 <label className="block text-sm mb-1">Attack</label>
//                 <input
//                   type="range"
//                   min="0.001"
//                   max="0.1"
//                   step="0.001"
//                   value={clapAttack}
//                   onChange={(e) => updateClapAttack(parseFloat(e.target.value))}
//                   className="w-full"
//                 />
//               </div>
//               <div>
//                 <label className="block text-sm mb-1">Release</label>
//                 <input
//                   type="range"
//                   min="0.05"
//                   max="1"
//                   step="0.01"
//                   value={clapRelease}
//                   onChange={(e) =>
//                     updateClapRelease(parseFloat(e.target.value))
//                   }
//                   className="w-full"
//                 />
//               </div>
//             </div>
//           </div>
//         </div>

//         {/* Clock Bias Controls */}
//         <div className="bg-gray-800 p-6 rounded-md">
//           <h3 className="text-lg mb-4">Clock Bias</h3>
//           <div className="mb-6">
//             <label className="block text-sm mb-2">Kick Loop Bias</label>
//             <input
//               type="range"
//               min="0.03"
//               max="0.97"
//               step="0.01"
//               value={kickLoopBias}
//               onChange={(e) => updateKickLoopBias(parseFloat(e.target.value))}
//               className="w-full"
//             />
//             <span className="text-xs text-gray-400">
//               {kickLoopBias < 0.5
//                 ? "← Earlier"
//                 : kickLoopBias > 0.5
//                   ? "Later →"
//                   : "Linear"}
//             </span>
//           </div>
//           <div>
//             <label className="block text-sm mb-2">Clap Loop Bias</label>
//             <input
//               type="range"
//               min="0.03"
//               max="0.97"
//               step="0.01"
//               value={clapLoopBias}
//               onChange={(e) => updateClapLoopBias(parseFloat(e.target.value))}
//               className="w-full"
//             />
//             <span className="text-xs text-gray-400">
//               {clapLoopBias < 0.5
//                 ? "← Earlier"
//                 : clapLoopBias > 0.5
//                   ? "Later →"
//                   : "Linear"}
//             </span>
//           </div>
//         </div>

//         {/* Effects Controls */}
//         <div className="bg-gray-800 p-6 rounded-md">
//           <h3 className="text-lg mb-4">Effects</h3>

//           {/* Sends */}
//           <div className="mb-6">
//             <h4 className="text-md mb-3">Sends</h4>
//             <div className="space-y-3">
//               <div>
//                 <label className="block text-sm mb-1">Delay Send</label>
//                 <input
//                   type="range"
//                   min="0"
//                   max="1"
//                   step="0.01"
//                   value={delaySend}
//                   onChange={(e) => updateDelaySend(parseFloat(e.target.value))}
//                   className="w-full"
//                 />
//               </div>
//               <div>
//                 <label className="block text-sm mb-1">Reverb Send</label>
//                 <input
//                   type="range"
//                   min="0"
//                   max="1"
//                   step="0.01"
//                   value={reverbSend}
//                   onChange={(e) => updateReverbSend(parseFloat(e.target.value))}
//                   className="w-full"
//                 />
//               </div>
//             </div>
//           </div>

//           {/* Returns */}
//           <div className="mb-6">
//             <h4 className="text-md mb-3">Returns</h4>
//             <div className="space-y-3">
//               <div>
//                 <label className="block text-sm mb-1">Delay Return</label>
//                 <input
//                   type="range"
//                   min="0"
//                   max="1"
//                   step="0.01"
//                   value={delayReturn}
//                   onChange={(e) =>
//                     updateDelayReturn(parseFloat(e.target.value))
//                   }
//                   className="w-full"
//                 />
//               </div>
//               <div>
//                 <label className="block text-sm mb-1">Reverb Return</label>
//                 <input
//                   type="range"
//                   min="0"
//                   max="1"
//                   step="0.01"
//                   value={reverbReturn}
//                   onChange={(e) =>
//                     updateReverbReturn(parseFloat(e.target.value))
//                   }
//                   className="w-full"
//                 />
//               </div>
//             </div>
//           </div>

//           {/* Delay Freeze */}
//           <div className="mb-6">
//             <label className="flex items-center gap-2">
//               <input
//                 type="checkbox"
//                 checked={delayFreeze}
//                 onChange={(e) => updateDelayFreeze(e.target.checked)}
//               />
//               <span className="text-sm">Delay Freeze</span>
//             </label>
//           </div>

//           {/* Modulated Parameters */}
//           <div>
//             <h4 className="text-md mb-3">Modulated Parameters</h4>
//             <div className="space-y-2">
//               <div>
//                 <label className="block text-xs text-gray-400">
//                   Delay Time
//                 </label>
//                 <div className="h-2 bg-gray-700 rounded-full overflow-hidden">
//                   <div
//                     className="h-full bg-yellow-600 transition-all duration-100"
//                     style={{
//                       width: `${modulatorValues.delayTime * 100}%`,
//                     }}
//                   ></div>
//                 </div>
//               </div>
//               <div>
//                 <label className="block text-xs text-gray-400">
//                   Reverb Size
//                 </label>
//                 <div className="h-2 bg-gray-700 rounded-full overflow-hidden">
//                   <div
//                     className="h-full bg-purple-600 transition-all duration-100"
//                     style={{
//                       width: `${modulatorValues.reverbSize * 100}%`,
//                     }}
//                   ></div>
//                 </div>
//               </div>
//               <div>
//                 <label className="block text-xs text-gray-400">
//                   Reverb Decay
//                 </label>
//                 <div className="h-2 bg-gray-700 rounded-full overflow-hidden">
//                   <div
//                     className="h-full bg-green-600 transition-all duration-100"
//                     style={{
//                       width: `${modulatorValues.reverbDecay * 100}%`,
//                     }}
//                   ></div>
//                 </div>
//               </div>
//             </div>
//           </div>
//         </div>
//       </div>
//     </>
//   );
// }
