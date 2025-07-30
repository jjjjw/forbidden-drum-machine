import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { StepGrid } from "./components/StepGrid";
import { AuditionerPage } from "./components/AuditionerPage";
import "./App.css";

function App() {
  const [audioStarted] = useState(true); // Audio is always started now
  const [audioPaused, setAudioPaused] = useState(true); // Starts paused
  const [bpm, setBpm] = useState(120);
  const [kickPattern, setKickPattern] = useState([
    true,
    false,
    false,
    false,
    false,
    false,
    true,
    false,
    false,
    false,
    false,
    false,
    false,
    false,
    true,
    false,
  ]);
  const [status, setStatus] = useState("");
  const [currentKickStep, setCurrentKickStep] = useState(0);
  const [currentClapStep, setCurrentClapStep] = useState(0);
  const [modulatorValues, setModulatorValues] = useState({
    delayTime: 0.25,
    reverbSize: 0.5,
    reverbDecay: 0.5,
  });
  const [delaySend, setDelaySend] = useState(0.2);
  const [reverbSend, setReverbSend] = useState(0.3);
  const [delayReturn, setDelayReturn] = useState(0.8);
  const [reverbReturn, setReverbReturn] = useState(0.6);
  const [delayFreeze, setDelayFreeze] = useState(false);
  const [kickAttack, setKickAttack] = useState(0.005);
  const [kickRelease, setKickRelease] = useState(0.2);

  // Clap pattern state
  const [clapPattern, setClapPattern] = useState([
    false,
    false,
    false,
    false,
    true,
    false,
    false,
    false,
    false,
    false,
    false,
    false,
    true,
    false,
    false,
    false,
  ]);

  // Markov and clock bias controls
  const [markovDensity, setMarkovDensity] = useState(0.3);
  const [kickLoopBias, setKickLoopBias] = useState(0.5);
  const [clapLoopBias, setClapLoopBias] = useState(0.5);

  // Volume controls
  const [kickVolume, setKickVolume] = useState(0.8);
  const [clapVolume, setClapVolume] = useState(0.6);

  // Navigation state
  const [currentSystem, setCurrentSystem] = useState<
    "drum_machine" | "auditioner"
  >("drum_machine");

  // Listen for events from audio thread
  useEffect(() => {
    let kickStepUnlisten: (() => void) | null = null;
    let clapStepUnlisten: (() => void) | null = null;
    let modulatorUnlisten: (() => void) | null = null;
    let kickPatternUnlisten: (() => void) | null = null;
    let clapPatternUnlisten: (() => void) | null = null;

    const setupListeners = async () => {
      try {
        // Listen for kick step changes
        kickStepUnlisten = await listen<number>(
          "kick_step_changed",
          (event) => {
            setCurrentKickStep(event.payload);
          },
        );

        // Listen for clap step changes
        clapStepUnlisten = await listen<number>(
          "clap_step_changed",
          (event) => {
            setCurrentClapStep(event.payload);
          },
        );

        // Listen for modulator value updates
        modulatorUnlisten = await listen<[number, number, number]>(
          "modulator_values",
          (event) => {
            const [delayTime, reverbSize, reverbDecay] = event.payload;
            setModulatorValues({ delayTime, reverbSize, reverbDecay });
          },
        );

        // Listen for generated kick patterns
        kickPatternUnlisten = await listen<boolean[]>(
          "kick_pattern_generated",
          (event) => {
            setKickPattern(event.payload);
          },
        );

        // Listen for generated clap patterns
        clapPatternUnlisten = await listen<boolean[]>(
          "clap_pattern_generated",
          (event) => {
            setClapPattern(event.payload);
          },
        );
      } catch (error) {
        console.error("Error setting up event listeners:", error);
      }
    };

    // Setup listeners immediately since audio is always running
    setupListeners();

    return () => {
      if (kickStepUnlisten) kickStepUnlisten();
      if (clapStepUnlisten) clapStepUnlisten();
      if (modulatorUnlisten) modulatorUnlisten();
      if (kickPatternUnlisten) kickPatternUnlisten();
      if (clapPatternUnlisten) clapPatternUnlisten();
    };
  }, []);

  async function stopAudio() {
    try {
      await invoke("send_audio_event", {
        systemName: "drum_machine",
        nodeName: "system",
        eventName: "set_paused",
        parameter: 1.0,
      });
      setAudioPaused(true);
      setStatus("Audio paused");
    } catch (error) {
      setStatus(`Error: ${error}`);
    }
  }

  async function resumeAudio() {
    try {
      await invoke("send_audio_event", {
        systemName: "drum_machine",
        nodeName: "system",
        eventName: "set_paused",
        parameter: 0.0,
      });
      setAudioPaused(false);
      setStatus("Audio resumed");
    } catch (error) {
      setStatus(`Error: ${error}`);
    }
  }

  async function updateBpm(newBpm: number) {
    setBpm(newBpm);
    try {
      await invoke("send_audio_event", {
        systemName: "drum_machine",
        nodeName: "system",
        eventName: "set_bpm",
        parameter: newBpm,
      });
    } catch (error) {
      setStatus(`Error setting BPM: ${error}`);
    }
  }

  async function updateKickPattern(newPattern: boolean[]) {
    setKickPattern(newPattern);
    try {
      await invoke("set_sequence", {
        systemName: "drum_machine",
        sequenceData: { kick_pattern: newPattern },
      });
    } catch (error) {
      setStatus(`Error setting kick pattern: ${error}`);
    }
  }

  function toggleKickStep(index: number) {
    const newPattern = [...kickPattern];
    newPattern[index] = !newPattern[index];
    updateKickPattern(newPattern);
  }

  async function updateDelaySend(value: number) {
    setDelaySend(value);
    try {
      await invoke("send_audio_event", {
        systemName: "drum_machine",
        nodeName: "system",
        eventName: "set_delay_send",
        parameter: value,
      });
    } catch (error) {
      setStatus(`Error setting delay send: ${error}`);
    }
  }

  async function updateReverbSend(value: number) {
    setReverbSend(value);
    try {
      await invoke("send_audio_event", {
        systemName: "drum_machine",
        nodeName: "system",
        eventName: "set_reverb_send",
        parameter: value,
      });
    } catch (error) {
      setStatus(`Error setting reverb send: ${error}`);
    }
  }

  async function updateDelayReturn(value: number) {
    setDelayReturn(value);
    try {
      await invoke("send_audio_event", {
        systemName: "drum_machine",
        nodeName: "system",
        eventName: "set_delay_return",
        parameter: value,
      });
    } catch (error) {
      setStatus(`Error setting delay return: ${error}`);
    }
  }

  async function updateReverbReturn(value: number) {
    setReverbReturn(value);
    try {
      await invoke("send_audio_event", {
        systemName: "drum_machine",
        nodeName: "system",
        eventName: "set_reverb_return",
        parameter: value,
      });
    } catch (error) {
      setStatus(`Error setting reverb return: ${error}`);
    }
  }

  async function toggleDelayFreeze() {
    const newFreeze = !delayFreeze;
    setDelayFreeze(newFreeze);
    try {
      await invoke("send_audio_event", {
        systemName: "drum_machine",
        nodeName: "delay",
        eventName: "set_freeze",
        parameter: newFreeze ? 1.0 : 0.0,
      });
    } catch (error) {
      setStatus(`Error setting delay freeze: ${error}`);
    }
  }

  async function updateKickAttack(value: number) {
    setKickAttack(value);
    try {
      await invoke("send_audio_event", {
        systemName: "drum_machine",
        nodeName: "kick",
        eventName: "set_attack",
        parameter: value,
      });
    } catch (error) {
      setStatus(`Error setting kick attack: ${error}`);
    }
  }

  async function updateKickRelease(value: number) {
    setKickRelease(value);
    try {
      await invoke("send_audio_event", {
        systemName: "drum_machine",
        nodeName: "kick",
        eventName: "set_release",
        parameter: value,
      });
    } catch (error) {
      setStatus(`Error setting kick release: ${error}`);
    }
  }

  async function updateClapPattern(newPattern: boolean[]) {
    setClapPattern(newPattern);
    try {
      await invoke("set_sequence", {
        systemName: "drum_machine",
        sequenceData: { clap_pattern: newPattern },
      });
    } catch (error) {
      setStatus(`Error setting clap pattern: ${error}`);
    }
  }

  function toggleClapStep(index: number) {
    const newPattern = [...clapPattern];
    newPattern[index] = !newPattern[index];
    updateClapPattern(newPattern);
  }

  async function updateMarkovDensity(value: number) {
    setMarkovDensity(value);
    try {
      await invoke("send_audio_event", {
        systemName: "drum_machine",
        nodeName: "clap",
        eventName: "set_density",
        parameter: value,
      });
    } catch (error) {
      setStatus(`Error setting markov density: ${error}`);
    }
  }

  async function generateKickPattern() {
    try {
      await invoke("send_audio_event", {
        systemName: "drum_machine",
        nodeName: "system",
        eventName: "generate_kick_pattern",
        parameter: 0.0,
      });
    } catch (error) {
      setStatus(`Error generating kick pattern: ${error}`);
    }
  }

  async function generateClapPattern() {
    try {
      await invoke("send_audio_event", {
        systemName: "drum_machine",
        nodeName: "system",
        eventName: "generate_clap_pattern",
        parameter: 0.0,
      });
    } catch (error) {
      setStatus(`Error generating clap pattern: ${error}`);
    }
  }

  async function updateKickLoopBias(value: number) {
    setKickLoopBias(value);
    try {
      await invoke("send_audio_event", {
        systemName: "drum_machine",
        nodeName: "system",
        eventName: "set_kick_loop_bias",
        parameter: value,
      });
    } catch (error) {
      setStatus(`Error setting kick loop bias: ${error}`);
    }
  }

  async function updateClapLoopBias(value: number) {
    setClapLoopBias(value);
    try {
      await invoke("send_audio_event", {
        systemName: "drum_machine",
        nodeName: "system",
        eventName: "set_clap_loop_bias",
        parameter: value,
      });
    } catch (error) {
      setStatus(`Error setting clap loop bias: ${error}`);
    }
  }

  async function updateKickVolume(value: number) {
    setKickVolume(value);
    try {
      await invoke("send_audio_event", {
        systemName: "drum_machine",
        nodeName: "kick",
        eventName: "set_volume",
        parameter: value,
      });
    } catch (error) {
      setStatus(`Error setting kick volume: ${error}`);
    }
  }

  async function updateClapVolume(value: number) {
    setClapVolume(value);
    try {
      await invoke("send_audio_event", {
        systemName: "drum_machine",
        nodeName: "clap",
        eventName: "set_volume",
        parameter: value,
      });
    } catch (error) {
      setStatus(`Error setting clap volume: ${error}`);
    }
  }

  // Handle system switching
  const switchSystem = async (systemName: "drum_machine" | "auditioner") => {
    try {
      await invoke("switch_audio_system", { systemName });
      setCurrentSystem(systemName);
    } catch (error) {
      console.error(`Error switching to ${systemName}:`, error);
    }
  };

  return (
    <main className="min-h-screen bg-gray-900 text-white p-8 font-mono">
      <div className="max-w-6xl mx-auto">
        <div className="mb-4">
          <h1 className="text-lg text-neutral-300 mb-6">
            Forbidden Drum Machine
          </h1>

          {/* System Tabs */}
          <div className="flex gap-2 mb-6">
            <button
              onClick={() => switchSystem("drum_machine")}
              className={`px-4 py-2 rounded-md transition-all ${
                currentSystem === "drum_machine"
                  ? "bg-blue-600 text-white"
                  : "bg-gray-700 text-gray-300 hover:bg-gray-600"
              }`}
            >
              Drum Machine
            </button>
            <button
              onClick={() => switchSystem("auditioner")}
              className={`px-4 py-2 rounded-md transition-all ${
                currentSystem === "auditioner"
                  ? "bg-purple-600 text-white"
                  : "bg-gray-700 text-gray-300 hover:bg-gray-600"
              }`}
            >
              Auditioner
            </button>
          </div>
        </div>

        {/* Render system-specific content */}
        {currentSystem === "drum_machine" && (
          <>
            {/* Audio Control & Status */}
            <div className="bg-gray-800 rounded-xl p-6 mb-6 border border-gray-700">
              <h2 className="text-lg font-medium mb-4 text-green-400">
                Playback Control
              </h2>
              <div className="flex gap-4 mb-4">
                {!audioPaused && (
                  <button
                    onClick={stopAudio}
                    className="bg-yellow-600 hover:bg-yellow-700 text-white py-3 px-8 rounded-md transition-all transform hover:scale-105"
                  >
                    ⏸ Pause
                  </button>
                )}
                {audioPaused && (
                  <button
                    onClick={resumeAudio}
                    className="bg-blue-600 hover:bg-blue-700 text-white py-3 px-8 rounded-md transition-all transform hover:scale-105"
                  >
                    ▶ Play
                  </button>
                )}
              </div>
              <div className="flex justify-between items-center">
                {!audioPaused && (
                  <div className="text-green-400 font-semibold flex items-center gap-2">
                    <div className="w-3 h-3 bg-green-400 rounded-full animate-pulse"></div>
                    Playing - Kick: {currentKickStep + 1}/16, Clap:{" "}
                    {currentClapStep + 1}/16
                  </div>
                )}
                {audioPaused && (
                  <div className="text-yellow-400 font-semibold flex items-center gap-2">
                    <div className="w-3 h-3 bg-yellow-400 rounded-full"></div>
                    Paused
                  </div>
                )}
                {status && <p className="text-gray-300 text-sm">{status}</p>}
              </div>
            </div>

            {/* BPM Control */}
            <div className="bg-gray-800 rounded-xl p-6 mb-6 border border-gray-700">
              <h2 className="text-2xl mb-4 text-blue-400">Tempo</h2>
              <div className="flex items-center gap-6">
                <label htmlFor="bpm" className="text-xl min-w-fit">
                  BPM: {bpm}
                </label>
                <input
                  id="bpm"
                  type="range"
                  min="60"
                  max="200"
                  value={bpm}
                  onChange={(e) => updateBpm(parseInt(e.target.value))}
                  className="flex-1 h-3 bg-gray-700 rounded-md appearance-none cursor-pointer"
                />
              </div>
            </div>

            {/* Pattern Grids */}
            <div className="grid md:grid-cols-2 gap-6 mb-6">
              {/* Kick Pattern Grid */}
              <div className="bg-gray-800 rounded-xl p-6 border border-gray-700">
                <div className="flex justify-between items-center mb-4">
                  <h2 className="text-2xl text-red-400">Kick Pattern</h2>
                  <button
                    onClick={generateKickPattern}
                    className="bg-red-600 hover:bg-red-700 text-white py-2 px-4 rounded-md transition-all transform hover:scale-105"
                  >
                    Generate New
                  </button>
                </div>
                <StepGrid
                  pattern={kickPattern}
                  currentStep={currentKickStep}
                  audioStarted={audioStarted}
                  onStepToggle={(index) => toggleKickStep(index)}
                  label=""
                />
              </div>

              {/* Clap Pattern Grid */}
              <div className="bg-gray-800 rounded-xl p-6 border border-gray-700">
                <div className="flex justify-between items-center mb-4">
                  <h2 className="text-2xl text-cyan-400">Clap Pattern</h2>
                  <button
                    onClick={generateClapPattern}
                    className="bg-cyan-600 hover:bg-cyan-700 text-white py-2 px-4 rounded-md transition-all transform hover:scale-105"
                  >
                    Generate New
                  </button>
                </div>
                <StepGrid
                  pattern={clapPattern}
                  currentStep={currentClapStep}
                  audioStarted={audioStarted}
                  onStepToggle={(index) => toggleClapStep(index)}
                  label=""
                />
              </div>
            </div>

            {/* Markov Generation */}
            <div className="bg-gray-800 rounded-xl p-6 mb-6 border border-gray-700">
              <h2 className="text-2xl mb-4 text-purple-400">
                Pattern Generation
              </h2>
              <div className="flex items-center gap-6">
                <div className="flex-1">
                  <label className="block text-sm mb-2">
                    Markov Density: {(markovDensity * 100).toFixed(0)}%
                  </label>
                  <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.01"
                    value={markovDensity}
                    onChange={(e) =>
                      updateMarkovDensity(parseFloat(e.target.value))
                    }
                    className="w-full h-2 bg-gray-700 rounded-md appearance-none cursor-pointer"
                  />
                </div>
              </div>
            </div>

            {/* Instrument Controls */}
            <div className="grid md:grid-cols-2 gap-6 mb-6">
              {/* Kick Controls */}
              <div className="bg-gray-800 rounded-xl p-6 border border-gray-700">
                <h2 className="text-2xl mb-4 text-red-400">Kick Controls</h2>
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm mb-2">
                      Volume: {(kickVolume * 100).toFixed(0)}%
                    </label>
                    <input
                      type="range"
                      min="0"
                      max="1"
                      step="0.01"
                      value={kickVolume}
                      onChange={(e) =>
                        updateKickVolume(parseFloat(e.target.value))
                      }
                      className="w-full h-2 bg-gray-700 rounded-md appearance-none cursor-pointer"
                    />
                  </div>
                  <div>
                    <label className="block text-sm mb-2">
                      Attack: {(kickAttack * 1000).toFixed(1)}ms
                    </label>
                    <input
                      type="range"
                      min="0.001"
                      max="0.1"
                      step="0.001"
                      value={kickAttack}
                      onChange={(e) =>
                        updateKickAttack(parseFloat(e.target.value))
                      }
                      className="w-full h-2 bg-gray-700 rounded-md appearance-none cursor-pointer"
                    />
                  </div>
                  <div>
                    <label className="block text-sm mb-2">
                      Release: {(kickRelease * 1000).toFixed(0)}ms
                    </label>
                    <input
                      type="range"
                      min="0.01"
                      max="1.0"
                      step="0.01"
                      value={kickRelease}
                      onChange={(e) =>
                        updateKickRelease(parseFloat(e.target.value))
                      }
                      className="w-full h-2 bg-gray-700 rounded-md appearance-none cursor-pointer"
                    />
                  </div>
                </div>
              </div>

              {/* Clap Controls */}
              <div className="bg-gray-800 rounded-xl p-6 border border-gray-700">
                <h2 className="text-2xl mb-4 text-cyan-400">Clap Controls</h2>
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm mb-2">
                      Volume: {(clapVolume * 100).toFixed(0)}%
                    </label>
                    <input
                      type="range"
                      min="0"
                      max="1"
                      step="0.01"
                      value={clapVolume}
                      onChange={(e) =>
                        updateClapVolume(parseFloat(e.target.value))
                      }
                      className="w-full h-2 bg-gray-700 rounded-md appearance-none cursor-pointer"
                    />
                  </div>
                </div>
              </div>
            </div>

            {/* Clock Bias Controls */}
            <div className="bg-gray-800 rounded-xl p-6 mb-6 border border-gray-700">
              <h2 className="text-2xl mb-4 text-orange-400">Clock Bias</h2>
              <div className="grid md:grid-cols-2 gap-6">
                <div>
                  <label className="block text-sm mb-2">
                    Kick Bias: {kickLoopBias.toFixed(2)}
                  </label>
                  <input
                    type="range"
                    min="0.03"
                    max="0.97"
                    step="0.01"
                    value={kickLoopBias}
                    onChange={(e) =>
                      updateKickLoopBias(parseFloat(e.target.value))
                    }
                    className="w-full h-2 bg-gray-700 rounded-md appearance-none cursor-pointer"
                  />
                </div>
                <div>
                  <label className="block text-sm mb-2">
                    Clap Bias: {clapLoopBias.toFixed(2)}
                  </label>
                  <input
                    type="range"
                    min="0.03"
                    max="0.97"
                    step="0.01"
                    value={clapLoopBias}
                    onChange={(e) =>
                      updateClapLoopBias(parseFloat(e.target.value))
                    }
                    className="w-full h-2 bg-gray-700 rounded-md appearance-none cursor-pointer"
                  />
                </div>
              </div>
            </div>

            {/* Effects Controls */}
            <div className="grid md:grid-cols-2 gap-6 mb-6">
              {/* Delay Controls */}
              <div className="bg-gray-800 rounded-xl p-6 border border-gray-700">
                <h2 className="text-2xl mb-4 text-purple-400">Delay</h2>
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm mb-2">
                      Send: {(delaySend * 100).toFixed(0)}%
                    </label>
                    <input
                      type="range"
                      min="0"
                      max="1"
                      step="0.01"
                      value={delaySend}
                      onChange={(e) =>
                        updateDelaySend(parseFloat(e.target.value))
                      }
                      className="w-full h-2 bg-gray-700 rounded-md appearance-none cursor-pointer"
                    />
                  </div>
                  <div>
                    <label className="block text-sm mb-2">
                      Return: {(delayReturn * 100).toFixed(0)}%
                    </label>
                    <input
                      type="range"
                      min="0"
                      max="1"
                      step="0.01"
                      value={delayReturn}
                      onChange={(e) =>
                        updateDelayReturn(parseFloat(e.target.value))
                      }
                      className="w-full h-2 bg-gray-700 rounded-md appearance-none cursor-pointer"
                    />
                  </div>
                  <div>
                    <label className="block text-sm mb-2">
                      Time: {(modulatorValues.delayTime * 1000).toFixed(0)}ms
                      (modulated)
                    </label>
                    <div className="h-2 bg-gray-700 rounded-md relative overflow-hidden">
                      <div
                        className="h-full bg-purple-500 transition-all duration-75"
                        style={{
                          width: `${(modulatorValues.delayTime / 0.5) * 100}%`,
                        }}
                      ></div>
                    </div>
                  </div>
                  <button
                    onClick={toggleDelayFreeze}
                    className={`w-full py-2 px-4 rounded-md transition-all ${
                      delayFreeze
                        ? "bg-yellow-600 hover:bg-yellow-700 text-white"
                        : "bg-gray-700 hover:bg-gray-600 text-gray-300"
                    }`}
                  >
                    {delayFreeze ? "Frozen" : "Flowing"}
                  </button>
                </div>
              </div>

              {/* Reverb Controls */}
              <div className="bg-gray-800 rounded-xl p-6 border border-gray-700">
                <h2 className="text-2xl mb-4 text-cyan-400">Reverb</h2>
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm mb-2">
                      Send: {(reverbSend * 100).toFixed(0)}%
                    </label>
                    <input
                      type="range"
                      min="0"
                      max="1"
                      step="0.01"
                      value={reverbSend}
                      onChange={(e) =>
                        updateReverbSend(parseFloat(e.target.value))
                      }
                      className="w-full h-2 bg-gray-700 rounded-md appearance-none cursor-pointer"
                    />
                  </div>
                  <div>
                    <label className="block text-sm mb-2">
                      Return: {(reverbReturn * 100).toFixed(0)}%
                    </label>
                    <input
                      type="range"
                      min="0"
                      max="1"
                      step="0.01"
                      value={reverbReturn}
                      onChange={(e) =>
                        updateReverbReturn(parseFloat(e.target.value))
                      }
                      className="w-full h-2 bg-gray-700 rounded-md appearance-none cursor-pointer"
                    />
                  </div>
                  <div>
                    <label className="block text-sm mb-2">
                      Size: {(modulatorValues.reverbSize * 100).toFixed(0)}%
                      (modulated)
                    </label>
                    <div className="h-2 bg-gray-700 rounded-md relative overflow-hidden">
                      <div
                        className="h-full bg-cyan-500 transition-all duration-75"
                        style={{
                          width: `${modulatorValues.reverbSize * 100}%`,
                        }}
                      ></div>
                    </div>
                  </div>
                  <div>
                    <label className="block text-sm mb-2">
                      Decay: {(modulatorValues.reverbDecay * 100).toFixed(0)}%
                      (modulated)
                    </label>
                    <div className="h-2 bg-gray-700 rounded-md relative overflow-hidden">
                      <div
                        className="h-full bg-teal-500 transition-all duration-75"
                        style={{
                          width: `${modulatorValues.reverbDecay * 100}%`,
                        }}
                      ></div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </>
        )}

        {/* Auditioner System */}
        {currentSystem === "auditioner" && <AuditionerPage />}
      </div>
    </main>
  );
}

export default App;
