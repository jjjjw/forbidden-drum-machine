import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { StepGrid } from "./components/StepGrid";
import "./App.css";

function App() {
  const [audioStarted, setAudioStarted] = useState(false);
  const [bpm, setBpm] = useState(120);
  const [kickPattern, setKickPattern] = useState([
    true, false, false, false,
    false, false, true, false,
    false, false, false, false,
    false, false, true, false
  ]);
  const [snarePattern, setSnarePattern] = useState([
    false, false, false, false,
    true, false, false, false,
    false, false, false, false,
    true, false, false, false
  ]);
  const [status, setStatus] = useState("");
  const [currentStep, setCurrentStep] = useState(0);
  const [modulatorValues, setModulatorValues] = useState({ delayTime: 0.25, reverbSize: 0.5, reverbDecay: 0.5 });
  const [delaySend, setDelaySend] = useState(0.2);
  const [reverbSend, setReverbSend] = useState(0.3);
  const [delayFreeze, setDelayFreeze] = useState(false);
  const [kickAttack, setKickAttack] = useState(0.005);
  const [kickRelease, setKickRelease] = useState(0.2);
  const [snareAttack, setSnareAttack] = useState(0.001);
  const [snareRelease, setSnareRelease] = useState(0.08);

  // Listen for step changes and modulator values from audio thread
  useEffect(() => {
    let stepUnlisten: (() => void) | null = null;
    let modulatorUnlisten: (() => void) | null = null;

    const setupListeners = async () => {
      try {
        // Listen for step changes
        stepUnlisten = await listen<number>("step_changed", (event) => {
          setCurrentStep(event.payload);
        });

        // Listen for modulator value updates
        modulatorUnlisten = await listen<[number, number, number]>("modulator_values", (event) => {
          const [delayTime, reverbSize, reverbDecay] = event.payload;
          setModulatorValues({ delayTime, reverbSize, reverbDecay });
        });
      } catch (error) {
        console.error("Error setting up event listeners:", error);
      }
    };

    if (audioStarted) {
      setupListeners();
    }

    return () => {
      if (stepUnlisten) stepUnlisten();
      if (modulatorUnlisten) modulatorUnlisten();
    };
  }, [audioStarted]);

  async function startAudio() {
    try {
      const result = await invoke("start_audio");
      setAudioStarted(true);
      setStatus(result as string);
    } catch (error) {
      setStatus(`Error: ${error}`);
    }
  }

  async function stopAudio() {
    try {
      const result = await invoke("stop_audio");
      setAudioStarted(false);
      setStatus(result as string);
    } catch (error) {
      setStatus(`Error: ${error}`);
    }
  }

  async function updateBpm(newBpm: number) {
    setBpm(newBpm);
    try {
      await invoke("set_bpm", { bpm: newBpm });
    } catch (error) {
      setStatus(`Error setting BPM: ${error}`);
    }
  }

  async function updateKickPattern(newPattern: boolean[]) {
    setKickPattern(newPattern);
    try {
      await invoke("set_kick_pattern", { pattern: newPattern });
    } catch (error) {
      setStatus(`Error setting kick pattern: ${error}`);
    }
  }

  async function updateSnarePattern(newPattern: boolean[]) {
    setSnarePattern(newPattern);
    try {
      await invoke("set_snare_pattern", { pattern: newPattern });
    } catch (error) {
      setStatus(`Error setting snare pattern: ${error}`);
    }
  }

  function toggleStep(pattern: boolean[], index: number, isKick: boolean) {
    const newPattern = [...pattern];
    newPattern[index] = !newPattern[index];
    if (isKick) {
      updateKickPattern(newPattern);
    } else {
      updateSnarePattern(newPattern);
    }
  }

  async function updateDelaySend(value: number) {
    setDelaySend(value);
    try {
      await invoke("set_delay_send", { send: value });
    } catch (error) {
      setStatus(`Error setting delay send: ${error}`);
    }
  }

  async function updateReverbSend(value: number) {
    setReverbSend(value);
    try {
      await invoke("set_reverb_send", { send: value });
    } catch (error) {
      setStatus(`Error setting reverb send: ${error}`);
    }
  }

  async function toggleDelayFreeze() {
    const newFreeze = !delayFreeze;
    setDelayFreeze(newFreeze);
    try {
      await invoke("set_delay_freeze", { freeze: newFreeze });
    } catch (error) {
      setStatus(`Error setting delay freeze: ${error}`);
    }
  }

  async function updateKickAttack(value: number) {
    setKickAttack(value);
    try {
      await invoke("set_kick_attack", { attack: value });
    } catch (error) {
      setStatus(`Error setting kick attack: ${error}`);
    }
  }

  async function updateKickRelease(value: number) {
    setKickRelease(value);
    try {
      await invoke("set_kick_release", { release: value });
    } catch (error) {
      setStatus(`Error setting kick release: ${error}`);
    }
  }

  async function updateSnareAttack(value: number) {
    setSnareAttack(value);
    try {
      await invoke("set_snare_attack", { attack: value });
    } catch (error) {
      setStatus(`Error setting snare attack: ${error}`);
    }
  }

  async function updateSnareRelease(value: number) {
    setSnareRelease(value);
    try {
      await invoke("set_snare_release", { release: value });
    } catch (error) {
      setStatus(`Error setting snare release: ${error}`);
    }
  }

  return (
    <main className="min-h-screen bg-gray-900 text-white p-8 font-mono">
      <div className="max-w-6xl mx-auto">
        <h1 className="text-5xl font-bold text-center mb-8 bg-gradient-to-r from-purple-400 to-pink-600 bg-clip-text text-transparent">
          Forbidden Drum Machine
        </h1>
        
        {/* Audio Control & Status */}
        <div className="bg-gray-800 rounded-xl p-6 mb-6 border border-gray-700">
          <h2 className="text-2xl font-bold mb-4 text-green-400">Audio Control</h2>
          <div className="flex gap-4 mb-4">
            <button 
              onClick={startAudio} 
              className="bg-green-600 hover:bg-green-700 text-white font-bold py-3 px-8 rounded-lg transition-all transform hover:scale-105 disabled:opacity-50"
              disabled={audioStarted}
            >
              ▶ Start Audio
            </button>
            <button 
              onClick={stopAudio} 
              className="bg-red-600 hover:bg-red-700 text-white font-bold py-3 px-8 rounded-lg transition-all transform hover:scale-105 disabled:opacity-50"
              disabled={!audioStarted}
            >
              ⏹ Stop Audio
            </button>
          </div>
          <div className="flex justify-between items-center">
            {audioStarted && (
              <div className="text-green-400 font-semibold flex items-center gap-2">
                <div className="w-3 h-3 bg-green-400 rounded-full animate-pulse"></div>
                Audio Running - Step: {currentStep + 1}/16
              </div>
            )}
            {status && <p className="text-gray-300 text-sm">{status}</p>}
          </div>
        </div>

        {/* BPM Control */}
        <div className="bg-gray-800 rounded-xl p-6 mb-6 border border-gray-700">
          <h2 className="text-2xl font-bold mb-4 text-blue-400">Tempo</h2>
          <div className="flex items-center gap-6">
            <label htmlFor="bpm" className="text-xl font-bold min-w-fit">BPM: {bpm}</label>
            <input
              id="bpm"
              type="range"
              min="60"
              max="200"
              value={bpm}
              onChange={(e) => updateBpm(parseInt(e.target.value))}
              className="flex-1 h-3 bg-gray-700 rounded-lg appearance-none cursor-pointer"
            />
          </div>
        </div>

        {/* Pattern Grid */}
        <div className="grid md:grid-cols-2 gap-6 mb-6">
          <StepGrid
            pattern={kickPattern}
            currentStep={currentStep}
            audioStarted={audioStarted}
            onStepToggle={(index) => toggleStep(kickPattern, index, true)}
            label="Kick Pattern"
          />
          <StepGrid
            pattern={snarePattern}
            currentStep={currentStep}
            audioStarted={audioStarted}
            onStepToggle={(index) => toggleStep(snarePattern, index, false)}
            label="Snare Pattern"
          />
        </div>

        {/* Instrument Controls */}
        <div className="grid md:grid-cols-2 gap-6 mb-6">
          {/* Kick Controls */}
          <div className="bg-gray-800 rounded-xl p-6 border border-gray-700">
            <h2 className="text-2xl font-bold mb-4 text-red-400">Kick Envelope</h2>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-bold mb-2">Attack: {(kickAttack * 1000).toFixed(1)}ms</label>
                <input
                  type="range"
                  min="0.001"
                  max="0.1"
                  step="0.001"
                  value={kickAttack}
                  onChange={(e) => updateKickAttack(parseFloat(e.target.value))}
                  className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer"
                />
              </div>
              <div>
                <label className="block text-sm font-bold mb-2">Release: {(kickRelease * 1000).toFixed(0)}ms</label>
                <input
                  type="range"
                  min="0.01"
                  max="1.0"
                  step="0.01"
                  value={kickRelease}
                  onChange={(e) => updateKickRelease(parseFloat(e.target.value))}
                  className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer"
                />
              </div>
            </div>
          </div>

          {/* Snare Controls */}
          <div className="bg-gray-800 rounded-xl p-6 border border-gray-700">
            <h2 className="text-2xl font-bold mb-4 text-blue-400">Snare Envelope</h2>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-bold mb-2">Attack: {(snareAttack * 1000).toFixed(1)}ms</label>
                <input
                  type="range"
                  min="0.001"
                  max="0.01"
                  step="0.0001"
                  value={snareAttack}
                  onChange={(e) => updateSnareAttack(parseFloat(e.target.value))}
                  className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer"
                />
              </div>
              <div>
                <label className="block text-sm font-bold mb-2">Release: {(snareRelease * 1000).toFixed(0)}ms</label>
                <input
                  type="range"
                  min="0.01"
                  max="0.5"
                  step="0.01"
                  value={snareRelease}
                  onChange={(e) => updateSnareRelease(parseFloat(e.target.value))}
                  className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer"
                />
              </div>
            </div>
          </div>
        </div>

        {/* Effects Controls */}
        <div className="grid md:grid-cols-2 gap-6 mb-6">
          {/* Delay Controls */}
          <div className="bg-gray-800 rounded-xl p-6 border border-gray-700">
            <h2 className="text-2xl font-bold mb-4 text-purple-400">Delay</h2>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-bold mb-2">Send: {(delaySend * 100).toFixed(0)}%</label>
                <input
                  type="range"
                  min="0"
                  max="1"
                  step="0.01"
                  value={delaySend}
                  onChange={(e) => updateDelaySend(parseFloat(e.target.value))}
                  className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer"
                />
              </div>
              <div>
                <label className="block text-sm font-bold mb-2">Time: {(modulatorValues.delayTime * 1000).toFixed(0)}ms (modulated)</label>
                <div className="h-2 bg-gray-700 rounded-lg relative overflow-hidden">
                  <div 
                    className="h-full bg-purple-500 transition-all duration-75"
                    style={{ width: `${(modulatorValues.delayTime / 0.5) * 100}%` }}
                  ></div>
                </div>
              </div>
              <button
                onClick={toggleDelayFreeze}
                className={`w-full py-2 px-4 rounded-lg font-bold transition-all ${
                  delayFreeze 
                    ? 'bg-yellow-600 hover:bg-yellow-700 text-white' 
                    : 'bg-gray-700 hover:bg-gray-600 text-gray-300'
                }`}
              >
                {delayFreeze ? '❄️ Frozen' : '🌊 Flowing'}
              </button>
            </div>
          </div>

          {/* Reverb Controls */}
          <div className="bg-gray-800 rounded-xl p-6 border border-gray-700">
            <h2 className="text-2xl font-bold mb-4 text-cyan-400">Reverb</h2>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-bold mb-2">Send: {(reverbSend * 100).toFixed(0)}%</label>
                <input
                  type="range"
                  min="0"
                  max="1"
                  step="0.01"
                  value={reverbSend}
                  onChange={(e) => updateReverbSend(parseFloat(e.target.value))}
                  className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer"
                />
              </div>
              <div>
                <label className="block text-sm font-bold mb-2">Size: {(modulatorValues.reverbSize * 100).toFixed(0)}% (modulated)</label>
                <div className="h-2 bg-gray-700 rounded-lg relative overflow-hidden">
                  <div 
                    className="h-full bg-cyan-500 transition-all duration-75"
                    style={{ width: `${modulatorValues.reverbSize * 100}%` }}
                  ></div>
                </div>
              </div>
              <div>
                <label className="block text-sm font-bold mb-2">Decay: {(modulatorValues.reverbDecay * 100).toFixed(0)}% (modulated)</label>
                <div className="h-2 bg-gray-700 rounded-lg relative overflow-hidden">
                  <div 
                    className="h-full bg-teal-500 transition-all duration-75"
                    style={{ width: `${modulatorValues.reverbDecay * 100}%` }}
                  ></div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </main>
  );
}

export default App;
