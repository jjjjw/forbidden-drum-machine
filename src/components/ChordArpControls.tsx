import { useState } from "react"
import { Scale, Note } from "tonal"
import { invoke } from "@tauri-apps/api/core"
import { TranceRiff, SystemNames, NodeNames, Commands } from "../events"

interface ChordArpControlsProps {
  onSequenceGenerated: (sequence: Array<[number, number, number]>) => void
  bpm?: number // Optional since it's not used
}

const ROOT_NOTES = [
  "C",
  "C#",
  "D",
  "D#",
  "E",
  "F",
  "F#",
  "G",
  "G#",
  "A",
  "A#",
  "B",
]

const SCALE_TYPES = [
  {
    name: "Major",
  },
  {
    name: "Minor",
  },
]

export function ChordArpControls({
  onSequenceGenerated,
  bpm: _bpm, // Renamed to indicate it's unused
}: ChordArpControlsProps) {
  const [rootNote, setRootNote] = useState("A")
  const [octave, setOctave] = useState(3)
  const [scaleType, setScaleType] = useState("Minor")
  const [sequencePattern, setSequencePattern] = useState("1 3 5 7")
  const [noteLength, setNoteLength] = useState(0.0625) // 1/16 note

  const sendSequenceToBackend = async (
    sequence: Array<[number, number, number]>
  ) => {
    try {
      await invoke(Commands.SendClientEvent, {
        systemName: SystemNames.TranceRiff,
        nodeName: NodeNames.System,
        eventName: TranceRiff.System.SetSequence,
        data: sequence,
      })
    } catch (error) {
      console.error("Error sending sequence:", error)
    }
  }

  const generateSequence = () => {
    // Parse the sequence pattern (space or comma separated)
    const patternTokens = sequencePattern
      .split(/[\s,]+/)
      .filter((token) => token.length > 0)

    // Create scale degrees function for the selected scale
    const scaleName = `${rootNote}${octave} ${scaleType.toLowerCase()}`
    const getScaleDegree = Scale.degrees(scaleName)

    // Create sequence from pattern
    const sequence: Array<[number, number, number]> = patternTokens.map(
      (token) => {
        // Convert note length to PPQN pulses (8 PPQN, 1 whole note = 4 beats)
        // noteLength is fraction of whole note: 0.125 = 1/8 note = 0.5 beats = 4 pulses
        const duration = Math.max(1, Math.round(noteLength * 4 * 8)) // noteLength * beats_per_whole_note * ppqn
        const velocity = 0.7 // Default velocity

        if (token.toLowerCase() === "x") {
          // Silence - frequency 0
          return [0, duration, 0]
        } else {
          // Parse scale degree
          const scaleDegree = parseInt(token)
          if (isNaN(scaleDegree)) {
            // Invalid scale degree, treat as silence
            return [0, duration, 0]
          }

          // Get note name for this scale degree
          const noteName = getScaleDegree(scaleDegree)
          const frequency = Note.freq(noteName) || 0
          return [frequency, duration, velocity]
        }
      }
    )

    console.log("Sequence:", sequence)

    onSequenceGenerated(sequence)
    sendSequenceToBackend(sequence)
  }

  return (
    <div className="bg-gray-800 p-6">
      <h3 className="text-xl font-bold text-green-400 mb-4">
        Scale Sequence Generator
      </h3>

      <div className="grid grid-cols-2 gap-4 mb-4">
        {/* Root Note */}
        <div>
          <label className="block text-sm font-medium text-gray-300 mb-2">
            Root Note
          </label>
          <select
            value={rootNote}
            onChange={(e) => setRootNote(e.target.value)}
            className="w-full bg-gray-700 text-white px-3 py-2"
          >
            {ROOT_NOTES.map((note) => (
              <option key={note} value={note}>
                {note}
              </option>
            ))}
          </select>
        </div>

        {/* Octave */}
        <div>
          <label className="block text-sm font-medium text-gray-300 mb-2">
            Octave
          </label>
          <select
            value={octave}
            onChange={(e) => setOctave(parseInt(e.target.value))}
            className="w-full bg-gray-700 text-white px-3 py-2"
          >
            {[1, 2, 3, 4, 5, 6].map((oct) => (
              <option key={oct} value={oct}>
                {oct}
              </option>
            ))}
          </select>
        </div>

        {/* Scale Type */}
        <div>
          <label className="block text-sm font-medium text-gray-300 mb-2">
            Scale Type
          </label>
          <select
            value={scaleType}
            onChange={(e) => setScaleType(e.target.value)}
            className="w-full bg-gray-700 text-white px-3 py-2"
          >
            {SCALE_TYPES.map((scale) => (
              <option key={scale.name} value={scale.name}>
                {scale.name}
              </option>
            ))}
          </select>
        </div>
      </div>

      {/* Sequence Pattern */}
      <div className="mb-4">
        <label className="block text-sm font-medium text-gray-300 mb-2">
          Sequence Pattern
        </label>
        <input
          type="text"
          value={sequencePattern}
          onChange={(e) => setSequencePattern(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && generateSequence()}
          placeholder="1 3 5 7"
          className="w-full bg-gray-700 text-white px-3 py-2"
        />
        <p className="text-xs text-gray-400 mt-1">
          {
            "Enter scale degrees (1-7) or 'x' for silence. Negative numbers go down octaves."
          }
        </p>
      </div>

      {/* Note Length */}
      <div className="mb-4">
        <div className="flex justify-between mb-1">
          <label className="text-sm text-gray-400">Note Length</label>
          <span className="text-sm text-gray-500">
            1/{Math.round(1 / noteLength)} note
          </span>
        </div>
        <input
          type="range"
          min={0.03125}
          max={0.5}
          step={0.03125}
          value={noteLength}
          onChange={(e) => setNoteLength(parseFloat(e.target.value))}
          className="w-full"
        />
      </div>

      {/* Generate & Send Button */}
      <button
        onClick={generateSequence}
        className="w-full bg-green-600 hover:bg-green-500 text-white font-medium py-2 px-4 transition-colors duration-200 cursor-pointer"
      >
        Regen
      </button>
    </div>
  )
}
