import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AudioEvent, SystemName, NodeName } from "../events";

const SCALES = [
  { name: "Major", value: 0 },
  { name: "Minor", value: 1 },
  { name: "Dorian", value: 2 },
  { name: "Phrygian", value: 3 },
  { name: "Mixolydian", value: 4 },
  { name: "Blues", value: 5 },
];

const ROOT_NOTES = [
  { name: "C", freq: 261.63 },
  { name: "C#", freq: 277.18 },
  { name: "D", freq: 293.66 },
  { name: "D#", freq: 311.13 },
  { name: "E", freq: 329.63 },
  { name: "F", freq: 349.23 },
  { name: "F#", freq: 369.99 },
  { name: "G", freq: 392.00 },
  { name: "G#", freq: 415.30 },
  { name: "A", freq: 440.00 },
  { name: "A#", freq: 466.16 },
  { name: "B", freq: 493.88 },
];

export function TranceRiffPage(): JSX.Element {
  const [bpm, setBpm] = useState(138);
  const [isPaused, setIsPaused] = useState(false);
  const [selectedScale, setSelectedScale] = useState(1); // Minor
  const [selectedRootNote, setSelectedRootNote] = useState(9); // A

  // Synth parameters
  const [synthGain, setSynthGain] = useState(0.5);
  const [detune, setDetune] = useState(1.0);
  const [stereoWidth, setStereoWidth] = useState(0.8);
  const [filterCutoff, setFilterCutoff] = useState(1000);
  const [filterResonance, setFilterResonance] = useState(0.7);
  const [filterEnvAmount, setFilterEnvAmount] = useState(2000);
  const [ampAttack, setAmpAttack] = useState(0.01);
  const [ampRelease, setAmpRelease] = useState(0.5);
  const [filterAttack, setFilterAttack] = useState(0.3);
  const [filterRelease, setFilterRelease] = useState(0.3);

  // Switch to trance riff system when this page loads
  useEffect(() => {
    const switchToTranceRiff = async () => {
      try {
        await invoke("switch_audio_system", { systemName: SystemName.TranceRiff });
      } catch (error) {
        console.error("Error switching to trance riff system:", error);
      }
    };
    
    switchToTranceRiff();
  }, []);

  const sendAudioEvent = async (nodeName: string, eventName: string, parameter: number) => {
    try {
      await invoke("send_audio_event", {
        systemName: SystemName.TranceRiff,
        nodeName,
        eventName,
        parameter,
      });
    } catch (error) {
      console.error("Error sending audio event:", error);
    }
  };

  const handleBpmChange = (newBpm: number) => {
    setBpm(newBpm);
    sendAudioEvent(NodeName.System, AudioEvent.System.SetBpm, newBpm);
  };

  const handlePauseToggle = () => {
    const newPaused = !isPaused;
    setIsPaused(newPaused);
    sendAudioEvent(NodeName.System, AudioEvent.System.SetPaused, newPaused ? 1 : 0);
  };

  const handleScaleChange = (scaleValue: number) => {
    setSelectedScale(scaleValue);
    sendAudioEvent(NodeName.System, AudioEvent.TranceRiff.SetScale, scaleValue);
  };

  const handleRootNoteChange = (rootNoteIndex: number) => {
    setSelectedRootNote(rootNoteIndex);
    const frequency = ROOT_NOTES[rootNoteIndex].freq;
    sendAudioEvent(NodeName.System, AudioEvent.TranceRiff.SetRootNote, frequency);
  };

  const handleSynthParameter = (eventName: string, value: number, setter: (val: number) => void) => {
    setter(value);
    sendAudioEvent("synth", eventName, value);
  };

  return (
    <div className="space-y-8">
      {/* Header */}
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-2xl font-bold text-green-400 mb-4">Trance Riff Generator</h2>
        
        {/* Transport Controls */}
        <div className="flex items-center gap-6 mb-6">
          <button
            onClick={handlePauseToggle}
            className={`px-6 py-2 rounded-lg font-medium ${
              isPaused 
                ? "bg-green-600 hover:bg-green-700 text-white" 
                : "bg-red-600 hover:bg-red-700 text-white"
            }`}
          >
            {isPaused ? "▶ Play" : "⏸ Pause"}
          </button>
          
          <div className="flex items-center gap-3">
            <label className="text-sm font-medium text-gray-300">BPM:</label>
            <input
              type="range"
              min={60}
              max={200}
              value={bpm}
              onChange={(e) => handleBpmChange(parseInt(e.target.value))}
              className="w-32"
            />
            <span className="text-sm text-gray-400 w-12">{bpm}</span>
          </div>
        </div>

        {/* Musical Parameters */}
        <div className="grid grid-cols-2 gap-6">
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">Scale</label>
            <select
              value={selectedScale}
              onChange={(e) => handleScaleChange(parseInt(e.target.value))}
              className="w-full bg-gray-700 text-white px-3 py-2 rounded-lg"
            >
              {SCALES.map((scale) => (
                <option key={scale.value} value={scale.value}>
                  {scale.name}
                </option>
              ))}
            </select>
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">Root Note</label>
            <select
              value={selectedRootNote}
              onChange={(e) => handleRootNoteChange(parseInt(e.target.value))}
              className="w-full bg-gray-700 text-white px-3 py-2 rounded-lg"
            >
              {ROOT_NOTES.map((note, index) => (
                <option key={index} value={index}>
                  {note.name} ({note.freq.toFixed(0)} Hz)
                </option>
              ))}
            </select>
          </div>
        </div>
      </div>

      {/* Synth Parameters */}
      <div className="bg-gray-800 rounded-lg p-6">
        <h3 className="text-xl font-bold text-green-400 mb-4">Synth Parameters</h3>
        
        <div className="grid grid-cols-2 gap-6">
          {/* Oscillator Section */}
          <div className="space-y-4">
            <h4 className="text-lg font-medium text-gray-300">Oscillator</h4>
            
            <div className="space-y-3">
              <div>
                <div className="flex justify-between">
                  <label className="text-sm text-gray-400">Gain</label>
                  <span className="text-sm text-gray-500">{Math.round(synthGain * 100)}%</span>
                </div>
                <input
                  type="range"
                  min={0}
                  max={1}
                  step={0.01}
                  value={synthGain}
                  onChange={(e) => handleSynthParameter(AudioEvent.Common.SetGain, parseFloat(e.target.value), setSynthGain)}
                  className="w-full"
                />
              </div>
              
              <div>
                <div className="flex justify-between">
                  <label className="text-sm text-gray-400">Detune</label>
                  <span className="text-sm text-gray-500">{detune.toFixed(2)}x</span>
                </div>
                <input
                  type="range"
                  min={0}
                  max={2}
                  step={0.01}
                  value={detune}
                  onChange={(e) => handleSynthParameter(AudioEvent.Supersaw.SetDetune, parseFloat(e.target.value), setDetune)}
                  className="w-full"
                />
              </div>
              
              <div>
                <div className="flex justify-between">
                  <label className="text-sm text-gray-400">Stereo Width</label>
                  <span className="text-sm text-gray-500">{Math.round(stereoWidth * 100)}%</span>
                </div>
                <input
                  type="range"
                  min={0}
                  max={1}
                  step={0.01}
                  value={stereoWidth}
                  onChange={(e) => handleSynthParameter(AudioEvent.Supersaw.SetStereoWidth, parseFloat(e.target.value), setStereoWidth)}
                  className="w-full"
                />
              </div>
            </div>
          </div>

          {/* Filter Section */}
          <div className="space-y-4">
            <h4 className="text-lg font-medium text-gray-300">Filter</h4>
            
            <div className="space-y-3">
              <div>
                <div className="flex justify-between">
                  <label className="text-sm text-gray-400">Cutoff</label>
                  <span className="text-sm text-gray-500">{filterCutoff} Hz</span>
                </div>
                <input
                  type="range"
                  min={100}
                  max={8000}
                  step={10}
                  value={filterCutoff}
                  onChange={(e) => handleSynthParameter(AudioEvent.Supersaw.SetFilterCutoff, parseInt(e.target.value), setFilterCutoff)}
                  className="w-full"
                />
              </div>
              
              <div>
                <div className="flex justify-between">
                  <label className="text-sm text-gray-400">Resonance</label>
                  <span className="text-sm text-gray-500">{filterResonance.toFixed(1)}</span>
                </div>
                <input
                  type="range"
                  min={0.1}
                  max={10}
                  step={0.1}
                  value={filterResonance}
                  onChange={(e) => handleSynthParameter(AudioEvent.Supersaw.SetFilterResonance, parseFloat(e.target.value), setFilterResonance)}
                  className="w-full"
                />
              </div>
              
              <div>
                <div className="flex justify-between">
                  <label className="text-sm text-gray-400">Env Amount</label>
                  <span className="text-sm text-gray-500">{filterEnvAmount} Hz</span>
                </div>
                <input
                  type="range"
                  min={0}
                  max={5000}
                  step={10}
                  value={filterEnvAmount}
                  onChange={(e) => handleSynthParameter(AudioEvent.Supersaw.SetFilterEnvAmount, parseInt(e.target.value), setFilterEnvAmount)}
                  className="w-full"
                />
              </div>
            </div>
          </div>

          {/* Amp Envelope */}
          <div className="space-y-4">
            <h4 className="text-lg font-medium text-gray-300">Amp Envelope</h4>
            
            <div className="space-y-3">
              <div>
                <div className="flex justify-between">
                  <label className="text-sm text-gray-400">Attack</label>
                  <span className="text-sm text-gray-500">{ampAttack.toFixed(3)}s</span>
                </div>
                <input
                  type="range"
                  min={0.001}
                  max={2}
                  step={0.001}
                  value={ampAttack}
                  onChange={(e) => handleSynthParameter(AudioEvent.Supersaw.SetAmpAttack, parseFloat(e.target.value), setAmpAttack)}
                  className="w-full"
                />
              </div>
              
              <div>
                <div className="flex justify-between">
                  <label className="text-sm text-gray-400">Release</label>
                  <span className="text-sm text-gray-500">{ampRelease.toFixed(2)}s</span>
                </div>
                <input
                  type="range"
                  min={0.01}
                  max={10}
                  step={0.01}
                  value={ampRelease}
                  onChange={(e) => handleSynthParameter(AudioEvent.Supersaw.SetAmpRelease, parseFloat(e.target.value), setAmpRelease)}
                  className="w-full"
                />
              </div>
            </div>
          </div>

          {/* Filter Envelope */}
          <div className="space-y-4">
            <h4 className="text-lg font-medium text-gray-300">Filter Envelope</h4>
            
            <div className="space-y-3">
              <div>
                <div className="flex justify-between">
                  <label className="text-sm text-gray-400">Attack</label>
                  <span className="text-sm text-gray-500">{filterAttack.toFixed(2)}s</span>
                </div>
                <input
                  type="range"
                  min={0.001}
                  max={2}
                  step={0.001}
                  value={filterAttack}
                  onChange={(e) => handleSynthParameter(AudioEvent.Supersaw.SetFilterAttack, parseFloat(e.target.value), setFilterAttack)}
                  className="w-full"
                />
              </div>
              
              <div>
                <div className="flex justify-between">
                  <label className="text-sm text-gray-400">Release</label>
                  <span className="text-sm text-gray-500">{filterRelease.toFixed(2)}s</span>
                </div>
                <input
                  type="range"
                  min={0.01}
                  max={10}
                  step={0.01}
                  value={filterRelease}
                  onChange={(e) => handleSynthParameter(AudioEvent.Supersaw.SetFilterRelease, parseFloat(e.target.value), setFilterRelease)}
                  className="w-full"
                />
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}