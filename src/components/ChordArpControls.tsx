import { useState } from "react"
import * as Tonal from "tonal"
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

const CHORD_TYPES = [
  { name: "Major", symbol: "" },
  { name: "Minor", symbol: "m" },
  { name: "7th", symbol: "7" },
  { name: "Major 7th", symbol: "maj7" },
  { name: "Minor 7th", symbol: "m7" },
  { name: "Sus2", symbol: "sus2" },
  { name: "Sus4", symbol: "sus4" },
  { name: "Diminished", symbol: "dim" },
  { name: "Augmented", symbol: "aug" },
]

const ARP_PATTERNS = [
  { name: "Up", pattern: [0, 1, 2, 3] },
  { name: "Down", pattern: [3, 2, 1, 0] },
  { name: "Up-Down", pattern: [0, 1, 2, 3, 3, 2, 1, 0] },
  { name: "Down-Up", pattern: [3, 2, 1, 0, 0, 1, 2, 3] },
  { name: "Random", pattern: [] }, // Will be randomized
]

export function ChordArpControls({
  onSequenceGenerated,
  bpm: _bpm, // Renamed to indicate it's unused
}: ChordArpControlsProps) {
  const [rootNote, setRootNote] = useState("A")
  const [octave, setOctave] = useState(3)
  const [chordType, setChordType] = useState("")
  const [arpPattern, setArpPattern] = useState("Up")
  const [octaveUp, setOctaveUp] = useState(false)
  const [octaveDown, setOctaveDown] = useState(false)
  const [noteLength, setNoteLength] = useState(0.125) // 1/8 note

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
    // Construct chord name
    const chordName = `${rootNote}${chordType}`

    // Get chord notes using Tonal
    const chord = Tonal.Chord.get(chordName)
    if (!chord.notes || chord.notes.length === 0) {
      return
    }

    // Get frequencies for chord notes
    let notes = chord.notes.map(
      (note) => Tonal.Note.freq(`${note}${octave}`) || 440
    )

    // Apply octave modifications
    if (octaveUp) {
      notes = notes.concat(notes.map((freq) => freq * 2))
    }
    if (octaveDown) {
      notes = notes.concat(notes.map((freq) => freq / 2))
    }

    // Get pattern
    const patternObj = ARP_PATTERNS.find((p) => p.name === arpPattern)
    let pattern = patternObj?.pattern || [0, 1, 2, 3]

    // Handle random pattern
    if (arpPattern === "Random") {
      pattern = Array.from({ length: 8 }, () =>
        Math.floor(Math.random() * notes.length)
      )
    }

    // Create sequence with pattern
    const sequence: Array<[number, number, number]> = pattern.map((index) => {
      const noteIndex = index % notes.length
      const frequency = notes[noteIndex]
      // Convert note length to tatums (8 tatums per beat, 1 whole note = 4 beats)
      // noteLength is fraction of whole note: 0.125 = 1/8 note = 0.5 beats = 4 tatums
      const duration = Math.max(1, Math.round(noteLength * 4 * 8)) // noteLength * beats_per_whole_note * tatums_per_beat
      const velocity = 0.7 // Default velocity
      return [frequency, duration, velocity]
    })

    console.log(sequence)

    onSequenceGenerated(sequence)
    sendSequenceToBackend(sequence)
  }

  return (
    <div className="bg-gray-800 rounded-lg p-6">
      <h3 className="text-xl font-bold text-green-400 mb-4">
        Chord & Arpeggiator
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
            className="w-full bg-gray-700 text-white px-3 py-2 rounded-lg"
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
            className="w-full bg-gray-700 text-white px-3 py-2 rounded-lg"
          >
            {[1, 2, 3, 4, 5, 6].map((oct) => (
              <option key={oct} value={oct}>
                {oct}
              </option>
            ))}
          </select>
        </div>

        {/* Chord Type */}
        <div>
          <label className="block text-sm font-medium text-gray-300 mb-2">
            Chord Type
          </label>
          <select
            value={chordType}
            onChange={(e) => setChordType(e.target.value)}
            className="w-full bg-gray-700 text-white px-3 py-2 rounded-lg"
          >
            {CHORD_TYPES.map((type) => (
              <option key={type.symbol} value={type.symbol}>
                {type.name}
              </option>
            ))}
          </select>
        </div>

        {/* Arp Pattern */}
        <div>
          <label className="block text-sm font-medium text-gray-300 mb-2">
            Arp Pattern
          </label>
          <select
            value={arpPattern}
            onChange={(e) => setArpPattern(e.target.value)}
            className="w-full bg-gray-700 text-white px-3 py-2 rounded-lg"
          >
            {ARP_PATTERNS.map((pattern) => (
              <option key={pattern.name} value={pattern.name}>
                {pattern.name}
              </option>
            ))}
          </select>
        </div>
      </div>

      {/* Octave Toggles */}
      <div className="flex gap-4 mb-4">
        <label className="flex items-center gap-2">
          <input
            type="checkbox"
            checked={octaveDown}
            onChange={(e) => setOctaveDown(e.target.checked)}
            className="w-4 h-4 text-green-600 bg-gray-700 rounded"
          />
          <span className="text-sm text-gray-300">Octave Down</span>
        </label>
        <label className="flex items-center gap-2">
          <input
            type="checkbox"
            checked={octaveUp}
            onChange={(e) => setOctaveUp(e.target.checked)}
            className="w-4 h-4 text-green-600 bg-gray-700 rounded"
          />
          <span className="text-sm text-gray-300">Octave Up</span>
        </label>
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
        className="w-full bg-green-600 hover:bg-green-700 text-white font-medium py-2 px-4 rounded-lg transition-colors"
      >
        Generate & Send to Synth
      </button>
    </div>
  )
}
