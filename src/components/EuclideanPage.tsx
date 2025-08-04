import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { TransportControls } from "./TransportControls";

interface EuclideanInstrument {
  steps: number;
  beats: number;
  tempoMult: number;
}

export function EuclideanPage() {
  const [bpm, setBpm] = useState(120);
  const [isPaused, setIsPaused] = useState(true);
  const [status, setStatus] = useState("");


  // Beat flash states - these will flash briefly when beats trigger
  const [kickFlash, setKickFlash] = useState(false);
  const [clapFlash, setClapFlash] = useState(false);

  // Euclidean parameters for each instrument
  const [kick, setKick] = useState<EuclideanInstrument>({
    steps: 8,
    beats: 3,
    tempoMult: 1.0,
  });

  const [clap, setClap] = useState<EuclideanInstrument>({
    steps: 8,
    beats: 2,
    tempoMult: 1.0,
  });

  const [hihat, setHihat] = useState<EuclideanInstrument>({
    steps: 16,
    beats: 7,
    tempoMult: 2.0,
  });

  const [chord, setChord] = useState<EuclideanInstrument>({
    steps: 8,
    beats: 1,
    tempoMult: 0.5,
  });

  // Set up event listeners
  useEffect(() => {
    let kickStepUnlisten: (() => void) | null = null;
    let clapStepUnlisten: (() => void) | null = null;

    const setupListeners = async () => {
      try {
        // Listen for step changes
        kickStepUnlisten = await listen<number>(
          "kick_step_changed",
          (event) => {
            const step = event.payload;
            
            // Check if this step should trigger a beat and flash
            if (shouldTriggerBeat(step, kick.steps, kick.beats)) {
              setKickFlash(true);
              setTimeout(() => setKickFlash(false), 100); // Flash for 100ms
            }
          },
        );

        clapStepUnlisten = await listen<number>(
          "clap_step_changed",
          (event) => {
            const step = event.payload;
            
            // Check if this step should trigger a beat and flash
            if (shouldTriggerBeat(step, clap.steps, clap.beats)) {
              setClapFlash(true);
              setTimeout(() => setClapFlash(false), 100); // Flash for 100ms
            }
          },
        );
      } catch (error) {
        console.error("Error setting up event listeners:", error);
      }
    };

    setupListeners();

    return () => {
      if (kickStepUnlisten) kickStepUnlisten();
      if (clapStepUnlisten) clapStepUnlisten();
    };
  }, []);


  // Calculate if a step should trigger a beat using Euclidean algorithm
  function shouldTriggerBeat(
    step: number,
    steps: number,
    beats: number,
  ): boolean {
    if (beats === 0 || steps === 0) return false;
    if (beats >= steps) return true;

    // Simple Euclidean distribution - same algorithm as backend
    let remainder = 0;
    let triggers = 0;

    for (let i = 0; i <= step; i++) {
      remainder += beats;
      if (remainder >= steps) {
        remainder -= steps;
        if (i === step) return true;
        triggers++;
      }
    }
    return false;
  }


  async function updateEuclideanParams(
    newKick = kick,
    newClap = clap,
    newHihat = hihat,
    newChord = chord,
  ) {
    try {
      await invoke("set_sequence", {
        systemName: "euclidean",
        sequenceData: {
          kick: {
            steps: newKick.steps,
            beats: newKick.beats,
            tempo_mult: newKick.tempoMult,
          },
          clap: {
            steps: newClap.steps,
            beats: newClap.beats,
            tempo_mult: newClap.tempoMult,
          },
          hihat: {
            steps: newHihat.steps,
            beats: newHihat.beats,
            tempo_mult: newHihat.tempoMult,
          },
          chord: {
            steps: newChord.steps,
            beats: newChord.beats,
            tempo_mult: newChord.tempoMult,
          },
        },
      });
    } catch (error) {
      setStatus(`Error updating parameters: ${error}`);
    }
  }

  async function updateKickSteps(steps: number) {
    const newKick = { ...kick, steps: Math.max(1, Math.min(32, steps)) };
    setKick(newKick);
    await updateEuclideanParams(newKick, clap, hihat, chord);
  }

  async function updateKickBeats(beats: number) {
    const newKick = {
      ...kick,
      beats: Math.max(0, Math.min(kick.steps, beats)),
    };
    setKick(newKick);
    await updateEuclideanParams(newKick, clap, hihat, chord);
  }

  async function updateKickTempoMult(tempoMult: number) {
    const newKick = {
      ...kick,
      tempoMult: Math.max(0.1, Math.min(4.0, tempoMult)),
    };
    setKick(newKick);
    await updateEuclideanParams(newKick, clap, hihat, chord);
  }

  async function updateClapSteps(steps: number) {
    const newClap = { ...clap, steps: Math.max(1, Math.min(32, steps)) };
    setClap(newClap);
    await updateEuclideanParams(kick, newClap, hihat, chord);
  }

  async function updateClapBeats(beats: number) {
    const newClap = {
      ...clap,
      beats: Math.max(0, Math.min(clap.steps, beats)),
    };
    setClap(newClap);
    await updateEuclideanParams(kick, newClap, hihat, chord);
  }

  async function updateClapTempoMult(tempoMult: number) {
    const newClap = {
      ...clap,
      tempoMult: Math.max(0.1, Math.min(4.0, tempoMult)),
    };
    setClap(newClap);
    await updateEuclideanParams(kick, newClap, hihat, chord);
  }

  async function updateHihatSteps(steps: number) {
    const newHihat = { ...hihat, steps: Math.max(1, Math.min(32, steps)) };
    setHihat(newHihat);
    await updateEuclideanParams(kick, clap, newHihat, chord);
  }

  async function updateHihatBeats(beats: number) {
    const newHihat = {
      ...hihat,
      beats: Math.max(0, Math.min(hihat.steps, beats)),
    };
    setHihat(newHihat);
    await updateEuclideanParams(kick, clap, newHihat, chord);
  }

  async function updateHihatTempoMult(tempoMult: number) {
    const newHihat = {
      ...hihat,
      tempoMult: Math.max(0.1, Math.min(4.0, tempoMult)),
    };
    setHihat(newHihat);
    await updateEuclideanParams(kick, clap, newHihat, chord);
  }

  async function updateChordSteps(steps: number) {
    const newChord = { ...chord, steps: Math.max(1, Math.min(32, steps)) };
    setChord(newChord);
    await updateEuclideanParams(kick, clap, hihat, newChord);
  }

  async function updateChordBeats(beats: number) {
    const newChord = {
      ...chord,
      beats: Math.max(0, Math.min(chord.steps, beats)),
    };
    setChord(newChord);
    await updateEuclideanParams(kick, clap, hihat, newChord);
  }

  async function updateChordTempoMult(tempoMult: number) {
    const newChord = {
      ...chord,
      tempoMult: Math.max(0.1, Math.min(4.0, tempoMult)),
    };
    setChord(newChord);
    await updateEuclideanParams(kick, clap, hihat, newChord);
  }

  return (
    <div className="space-y-6">
      <div className="text-neutral-400">
        {status && <p className="text-yellow-400">{status}</p>}
      </div>

      {/* Playback Control */}
      <TransportControls
        systemName="euclidean"
        isPaused={isPaused}
        bpm={bpm}
        onPausedChange={setIsPaused}
        onBpmChange={setBpm}
        sliderWidth="w-32"
      />

      {/* Instrument Controls */}
      <div className="grid grid-cols-2 gap-6">
        {/* Kick */}
        <div className="bg-gray-800 p-4 rounded">
          <h3 className="text-lg mb-4 flex items-center gap-2">
            Kick
            <div
              className={`w-3 h-3 rounded-full transition-all duration-100 ${
                kickFlash ? "bg-red-400 shadow-lg shadow-red-400" : "bg-red-800"
              }`}
            />
          </h3>

          <div className="space-y-3">
            <div>
              <label className="block text-sm mb-1">Steps: {kick.steps}</label>
              <input
                type="range"
                min="1"
                max="32"
                value={kick.steps}
                onChange={(e) => updateKickSteps(parseInt(e.target.value))}
                className="w-full"
              />
            </div>

            <div>
              <label className="block text-sm mb-1">Beats: {kick.beats}</label>
              <input
                type="range"
                min="0"
                max={kick.steps}
                value={kick.beats}
                onChange={(e) => updateKickBeats(parseInt(e.target.value))}
                className="w-full"
              />
            </div>

            <div>
              <label className="block text-sm mb-1">
                Tempo Mult: {kick.tempoMult.toFixed(1)}x
              </label>
              <input
                type="range"
                min="0.1"
                max="4.0"
                step="0.1"
                value={kick.tempoMult}
                onChange={(e) =>
                  updateKickTempoMult(parseFloat(e.target.value))
                }
                className="w-full"
              />
            </div>
          </div>
        </div>

        {/* Clap */}
        <div className="bg-gray-800 p-4 rounded">
          <h3 className="text-lg mb-4 flex items-center gap-2">
            Clap
            <div
              className={`w-3 h-3 rounded-full transition-all duration-100 ${
                clapFlash ? "bg-blue-400 shadow-lg shadow-blue-400" : "bg-blue-800"
              }`}
            />
          </h3>

          <div className="space-y-3">
            <div>
              <label className="block text-sm mb-1">Steps: {clap.steps}</label>
              <input
                type="range"
                min="1"
                max="32"
                value={clap.steps}
                onChange={(e) => updateClapSteps(parseInt(e.target.value))}
                className="w-full"
              />
            </div>

            <div>
              <label className="block text-sm mb-1">Beats: {clap.beats}</label>
              <input
                type="range"
                min="0"
                max={clap.steps}
                value={clap.beats}
                onChange={(e) => updateClapBeats(parseInt(e.target.value))}
                className="w-full"
              />
            </div>

            <div>
              <label className="block text-sm mb-1">
                Tempo Mult: {clap.tempoMult.toFixed(1)}x
              </label>
              <input
                type="range"
                min="0.1"
                max="4.0"
                step="0.1"
                value={clap.tempoMult}
                onChange={(e) =>
                  updateClapTempoMult(parseFloat(e.target.value))
                }
                className="w-full"
              />
            </div>
          </div>
        </div>

        {/* HiHat */}
        <div className="bg-gray-800 p-4 rounded">
          <h3 className="text-lg mb-4 flex items-center gap-2">
            HiHat
            <div className="w-3 h-3 rounded-full bg-yellow-600" />
          </h3>

          <div className="space-y-3">
            <div>
              <label className="block text-sm mb-1">Steps: {hihat.steps}</label>
              <input
                type="range"
                min="1"
                max="32"
                value={hihat.steps}
                onChange={(e) => updateHihatSteps(parseInt(e.target.value))}
                className="w-full"
              />
            </div>

            <div>
              <label className="block text-sm mb-1">Beats: {hihat.beats}</label>
              <input
                type="range"
                min="0"
                max={hihat.steps}
                value={hihat.beats}
                onChange={(e) => updateHihatBeats(parseInt(e.target.value))}
                className="w-full"
              />
            </div>

            <div>
              <label className="block text-sm mb-1">
                Tempo Mult: {hihat.tempoMult.toFixed(1)}x
              </label>
              <input
                type="range"
                min="0.1"
                max="4.0"
                step="0.1"
                value={hihat.tempoMult}
                onChange={(e) =>
                  updateHihatTempoMult(parseFloat(e.target.value))
                }
                className="w-full"
              />
            </div>
          </div>
        </div>

        {/* Chord */}
        <div className="bg-gray-800 p-4 rounded">
          <h3 className="text-lg mb-4 flex items-center gap-2">
            Chord
            <div className="w-3 h-3 rounded-full bg-purple-600" />
          </h3>

          <div className="space-y-3">
            <div>
              <label className="block text-sm mb-1">Steps: {chord.steps}</label>
              <input
                type="range"
                min="1"
                max="32"
                value={chord.steps}
                onChange={(e) => updateChordSteps(parseInt(e.target.value))}
                className="w-full"
              />
            </div>

            <div>
              <label className="block text-sm mb-1">Beats: {chord.beats}</label>
              <input
                type="range"
                min="0"
                max={chord.steps}
                value={chord.beats}
                onChange={(e) => updateChordBeats(parseInt(e.target.value))}
                className="w-full"
              />
            </div>

            <div>
              <label className="block text-sm mb-1">
                Tempo Mult: {chord.tempoMult.toFixed(1)}x
              </label>
              <input
                type="range"
                min="0.1"
                max="4.0"
                step="0.1"
                value={chord.tempoMult}
                onChange={(e) =>
                  updateChordTempoMult(parseFloat(e.target.value))
                }
                className="w-full"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
