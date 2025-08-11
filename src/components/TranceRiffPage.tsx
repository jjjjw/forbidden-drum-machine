import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"
import { TranceRiff, SystemNames, NodeNames, Commands } from "../events"
import { ChordArpControls } from "./ChordArpControls"

export function TranceRiffPage(): JSX.Element {
  const [bpm, setBpm] = useState(138)
  const [isPaused, setIsPaused] = useState(false)

  // Synth parameters
  const [synthGain, setSynthGain] = useState(0.5)
  const [detune, setDetune] = useState(1.0)
  const [stereoWidth, setStereoWidth] = useState(0.8)
  const [filterCutoff, setFilterCutoff] = useState(1000)
  const [filterResonance, setFilterResonance] = useState(0.7)
  const [filterEnvAmount, setFilterEnvAmount] = useState(2000)
  const [ampAttack, setAmpAttack] = useState(0.01)
  const [ampRelease, setAmpRelease] = useState(0.5)
  const [filterAttack, setFilterAttack] = useState(0.3)
  const [filterRelease, setFilterRelease] = useState(0.3)

  // Switch to trance riff system when this page loads
  useEffect(() => {
    const switchToTranceRiff = async () => {
      try {
        await invoke(Commands.SwitchAudioSystem, {
          systemName: SystemNames.TranceRiff,
        })
      } catch (error) {
        console.error("Error switching to TranceRiff system:", error)
      }
    }

    switchToTranceRiff()
  }, [])

  const sendAudioEvent = async (
    nodeName: string,
    eventName: string,
    parameter: number
  ) => {
    try {
      await invoke(Commands.SendClientEvent, {
        systemName: SystemNames.TranceRiff,
        nodeName,
        eventName,
        parameter,
      })
    } catch (error) {
      console.error("Error sending audio event:", error)
    }
  }

  const handleBpmChange = (newBpm: number) => {
    setBpm(newBpm)
    sendAudioEvent(NodeNames.System, TranceRiff.System.SetBpm, newBpm)
  }

  const handlePauseToggle = () => {
    const newPaused = !isPaused
    setIsPaused(newPaused)
    sendAudioEvent(
      NodeNames.System,
      TranceRiff.System.SetPaused,
      newPaused ? 1 : 0
    )
  }

  const handleSequenceGenerated = (
    _sequence: Array<[number, number, number]>
  ) => {
    // Sequence is now sent directly from ChordArpControls
  }

  const handleSynthParameter = (
    eventName: string,
    value: number,
    setter: (val: number) => void
  ) => {
    setter(value)
    sendAudioEvent(NodeNames.Supersaw, eventName, value)
  }

  return (
    <div className="space-y-8">
      {/* Header */}
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-2xl font-bold text-green-400 mb-4">
          Trance Riff Generator
        </h2>

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
      </div>

      {/* Chord Arp Controls */}
      <ChordArpControls
        onSequenceGenerated={handleSequenceGenerated}
        bpm={bpm}
      />

      {/* Synth Parameters */}
      <div className="bg-gray-800 rounded-lg p-6">
        <h3 className="text-xl font-bold text-green-400 mb-4">
          Synth Parameters
        </h3>

        <div className="grid grid-cols-2 gap-6">
          {/* Oscillator Section */}
          <div className="space-y-4">
            <h4 className="text-lg font-medium text-gray-300">Oscillator</h4>

            <div className="space-y-3">
              <div>
                <div className="flex justify-between">
                  <label className="text-sm text-gray-400">Gain</label>
                  <span className="text-sm text-gray-500">
                    {Math.round(synthGain * 100)}%
                  </span>
                </div>
                <input
                  type="range"
                  min={0}
                  max={1}
                  step={0.01}
                  value={synthGain}
                  onChange={(e) =>
                    handleSynthParameter(
                      TranceRiff.Supersaw.SetGain,
                      parseFloat(e.target.value),
                      setSynthGain
                    )
                  }
                  className="w-full"
                />
              </div>

              <div>
                <div className="flex justify-between">
                  <label className="text-sm text-gray-400">Detune</label>
                  <span className="text-sm text-gray-500">
                    {detune.toFixed(2)}x
                  </span>
                </div>
                <input
                  type="range"
                  min={0}
                  max={2}
                  step={0.01}
                  value={detune}
                  onChange={(e) =>
                    handleSynthParameter(
                      TranceRiff.Supersaw.SetDetune,
                      parseFloat(e.target.value),
                      setDetune
                    )
                  }
                  className="w-full"
                />
              </div>

              <div>
                <div className="flex justify-between">
                  <label className="text-sm text-gray-400">Stereo Width</label>
                  <span className="text-sm text-gray-500">
                    {Math.round(stereoWidth * 100)}%
                  </span>
                </div>
                <input
                  type="range"
                  min={0}
                  max={1}
                  step={0.01}
                  value={stereoWidth}
                  onChange={(e) =>
                    handleSynthParameter(
                      TranceRiff.Supersaw.SetStereoWidth,
                      parseFloat(e.target.value),
                      setStereoWidth
                    )
                  }
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
                  <span className="text-sm text-gray-500">
                    {filterCutoff} Hz
                  </span>
                </div>
                <input
                  type="range"
                  min={100}
                  max={8000}
                  step={10}
                  value={filterCutoff}
                  onChange={(e) =>
                    handleSynthParameter(
                      TranceRiff.Supersaw.SetFilterCutoff,
                      parseInt(e.target.value),
                      setFilterCutoff
                    )
                  }
                  className="w-full"
                />
              </div>

              <div>
                <div className="flex justify-between">
                  <label className="text-sm text-gray-400">Resonance</label>
                  <span className="text-sm text-gray-500">
                    {filterResonance.toFixed(1)}
                  </span>
                </div>
                <input
                  type="range"
                  min={0.1}
                  max={10}
                  step={0.1}
                  value={filterResonance}
                  onChange={(e) =>
                    handleSynthParameter(
                      TranceRiff.Supersaw.SetFilterResonance,
                      parseFloat(e.target.value),
                      setFilterResonance
                    )
                  }
                  className="w-full"
                />
              </div>

              <div>
                <div className="flex justify-between">
                  <label className="text-sm text-gray-400">Env Amount</label>
                  <span className="text-sm text-gray-500">
                    {filterEnvAmount} Hz
                  </span>
                </div>
                <input
                  type="range"
                  min={0}
                  max={5000}
                  step={10}
                  value={filterEnvAmount}
                  onChange={(e) =>
                    handleSynthParameter(
                      TranceRiff.Supersaw.SetFilterEnvAmount,
                      parseInt(e.target.value),
                      setFilterEnvAmount
                    )
                  }
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
                  <span className="text-sm text-gray-500">
                    {ampAttack.toFixed(3)}s
                  </span>
                </div>
                <input
                  type="range"
                  min={0.001}
                  max={2}
                  step={0.001}
                  value={ampAttack}
                  onChange={(e) =>
                    handleSynthParameter(
                      TranceRiff.Supersaw.SetAmpAttack,
                      parseFloat(e.target.value),
                      setAmpAttack
                    )
                  }
                  className="w-full"
                />
              </div>

              <div>
                <div className="flex justify-between">
                  <label className="text-sm text-gray-400">Release</label>
                  <span className="text-sm text-gray-500">
                    {ampRelease.toFixed(2)}s
                  </span>
                </div>
                <input
                  type="range"
                  min={0.01}
                  max={10}
                  step={0.01}
                  value={ampRelease}
                  onChange={(e) =>
                    handleSynthParameter(
                      TranceRiff.Supersaw.SetAmpRelease,
                      parseFloat(e.target.value),
                      setAmpRelease
                    )
                  }
                  className="w-full"
                />
              </div>
            </div>
          </div>

          {/* Filter Envelope */}
          <div className="space-y-4">
            <h4 className="text-lg font-medium text-gray-300">
              Filter Envelope
            </h4>

            <div className="space-y-3">
              <div>
                <div className="flex justify-between">
                  <label className="text-sm text-gray-400">Attack</label>
                  <span className="text-sm text-gray-500">
                    {filterAttack.toFixed(2)}s
                  </span>
                </div>
                <input
                  type="range"
                  min={0.001}
                  max={2}
                  step={0.001}
                  value={filterAttack}
                  onChange={(e) =>
                    handleSynthParameter(
                      TranceRiff.Supersaw.SetFilterAttack,
                      parseFloat(e.target.value),
                      setFilterAttack
                    )
                  }
                  className="w-full"
                />
              </div>

              <div>
                <div className="flex justify-between">
                  <label className="text-sm text-gray-400">Release</label>
                  <span className="text-sm text-gray-500">
                    {filterRelease.toFixed(2)}s
                  </span>
                </div>
                <input
                  type="range"
                  min={0.01}
                  max={10}
                  step={0.01}
                  value={filterRelease}
                  onChange={(e) =>
                    handleSynthParameter(
                      TranceRiff.Supersaw.SetFilterRelease,
                      parseFloat(e.target.value),
                      setFilterRelease
                    )
                  }
                  className="w-full"
                />
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
