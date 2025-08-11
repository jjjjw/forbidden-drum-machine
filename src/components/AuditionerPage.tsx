import { useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"
import { Auditioner, InstrumentConfig } from "./Auditioner"
import {
  Auditioner as AuditionerEvents,
  SystemNames,
  NodeNames,
  Commands,
} from "../events"

// Kick drum configuration
const kickDrumConfig: InstrumentConfig = {
  name: "Kick Drum",
  color: "red",
  triggerNode: NodeNames.Kick,
  parameters: [
    {
      name: "Gain",
      node: NodeNames.Kick,
      event: AuditionerEvents.Kick.SetGain,
      min: 0,
      max: 2,
      step: 0.01,
      defaultValue: 0.8,
      unit: "%",
    },
    {
      name: "Base Frequency",
      node: NodeNames.Kick,
      event: AuditionerEvents.Kick.SetBaseFrequency,
      min: 20,
      max: 200,
      step: 1,
      defaultValue: 60,
      unit: "hz",
    },
    {
      name: "Frequency Ratio",
      node: NodeNames.Kick,
      event: AuditionerEvents.Kick.SetFrequencyRatio,
      min: 1,
      max: 20,
      step: 0.1,
      defaultValue: 7.0,
      unit: "x",
    },
    {
      name: "Amp Attack",
      node: NodeNames.Kick,
      event: AuditionerEvents.Kick.SetAmpAttack,
      min: 0.001,
      max: 0.1,
      step: 0.001,
      defaultValue: 0.005,
      unit: "s",
    },
    {
      name: "Amp Release",
      node: NodeNames.Kick,
      event: AuditionerEvents.Kick.SetAmpRelease,
      min: 0.01,
      max: 2.0,
      step: 0.01,
      defaultValue: 0.2,
      unit: "s",
    },
    {
      name: "Freq Attack",
      node: NodeNames.Kick,
      event: AuditionerEvents.Kick.SetFreqAttack,
      min: 0.001,
      max: 0.1,
      step: 0.001,
      defaultValue: 0.002,
      unit: "s",
    },
    {
      name: "Freq Release",
      node: NodeNames.Kick,
      event: AuditionerEvents.Kick.SetFreqRelease,
      min: 0.001,
      max: 0.2,
      step: 0.001,
      defaultValue: 0.05,
      unit: "s",
    },
  ],
}

// Clap drum configuration
const clapDrumConfig: InstrumentConfig = {
  name: "Clap Drum",
  color: "blue",
  triggerNode: NodeNames.Clap,
  parameters: [
    {
      name: "Gain",
      node: NodeNames.Clap,
      event: AuditionerEvents.Clap.SetGain,
      min: 0,
      max: 2,
      step: 0.01,
      defaultValue: 0.6,
      unit: "%",
    },
  ],
}

// Hi-hat configuration
const hiHatConfig: InstrumentConfig = {
  name: "Hi-Hat",
  color: "yellow",
  triggerNode: NodeNames.HiHat,
  parameters: [
    {
      name: "Gain",
      node: NodeNames.HiHat,
      event: AuditionerEvents.HiHat.SetGain,
      min: 0,
      max: 2,
      step: 0.01,
      defaultValue: 1.0,
      unit: "%",
    },
    {
      name: "Length",
      node: NodeNames.HiHat,
      event: AuditionerEvents.HiHat.SetLength,
      min: 0.002,
      max: 0.5,
      step: 0.001,
      defaultValue: 0.05,
      unit: "s",
    },
  ],
}

// Chord synth configuration
const chordSynthConfig: InstrumentConfig = {
  name: "Chord Synth",
  color: "purple",
  triggerNode: NodeNames.Chord,
  parameters: [
    {
      name: "Gain",
      node: NodeNames.Chord,
      event: AuditionerEvents.Chord.SetGain,
      min: 0,
      max: 1,
      step: 0.01,
      defaultValue: 0.25,
      unit: "%",
    },
    {
      name: "Base Frequency",
      node: NodeNames.Chord,
      event: AuditionerEvents.Chord.SetBaseFrequency,
      min: 110,
      max: 440,
      step: 1,
      defaultValue: 220,
      unit: "hz",
    },
    {
      name: "Modulation Index",
      node: NodeNames.Chord,
      event: AuditionerEvents.Chord.SetModulationIndex,
      min: 0,
      max: 2,
      step: 0.01,
      defaultValue: 0.5,
      unit: "x",
    },
    {
      name: "Feedback",
      node: NodeNames.Chord,
      event: AuditionerEvents.Chord.SetFeedback,
      min: 0,
      max: 0.99,
      step: 0.01,
      defaultValue: 0.1,
      unit: "%",
    },
    {
      name: "Attack",
      node: NodeNames.Chord,
      event: AuditionerEvents.Chord.SetAttack,
      min: 0.01,
      max: 2,
      step: 0.01,
      defaultValue: 0.5,
      unit: "s",
    },
    {
      name: "Release",
      node: NodeNames.Chord,
      event: AuditionerEvents.Chord.SetRelease,
      min: 0.1,
      max: 8,
      step: 0.1,
      defaultValue: 4,
      unit: "s",
    },
  ],
}

// Supersaw synth configuration
const supersawConfig: InstrumentConfig = {
  name: "Supersaw Synth",
  color: "green",
  triggerNode: NodeNames.Supersaw,
  parameters: [
    {
      name: "Gain",
      node: NodeNames.Supersaw,
      event: AuditionerEvents.Supersaw.SetGain,
      min: 0,
      max: 1,
      step: 0.01,
      defaultValue: 0.5,
      unit: "%",
    },
    {
      name: "Base Frequency",
      node: NodeNames.Supersaw,
      event: AuditionerEvents.Supersaw.SetBaseFrequency,
      min: 110,
      max: 880,
      step: 1,
      defaultValue: 440,
      unit: "hz",
    },
    {
      name: "Detune",
      node: NodeNames.Supersaw,
      event: AuditionerEvents.Supersaw.SetDetune,
      min: 0,
      max: 2,
      step: 0.01,
      defaultValue: 1.0,
      unit: "x",
    },
    {
      name: "Stereo Width",
      node: NodeNames.Supersaw,
      event: AuditionerEvents.Supersaw.SetStereoWidth,
      min: 0,
      max: 1,
      step: 0.01,
      defaultValue: 0.8,
      unit: "%",
    },
    {
      name: "Filter Cutoff",
      node: NodeNames.Supersaw,
      event: AuditionerEvents.Supersaw.SetFilterCutoff,
      min: 100,
      max: 8000,
      step: 10,
      defaultValue: 1000,
      unit: "hz",
    },
    {
      name: "Filter Resonance",
      node: NodeNames.Supersaw,
      event: AuditionerEvents.Supersaw.SetFilterResonance,
      min: 0.1,
      max: 10,
      step: 0.1,
      defaultValue: 0.7,
      unit: "q",
    },
    {
      name: "Filter Env Amount",
      node: NodeNames.Supersaw,
      event: AuditionerEvents.Supersaw.SetFilterEnvAmount,
      min: 0,
      max: 5000,
      step: 10,
      defaultValue: 2000,
      unit: "hz",
    },
    {
      name: "Amp Attack",
      node: NodeNames.Supersaw,
      event: AuditionerEvents.Supersaw.SetAmpAttack,
      min: 0.001,
      max: 2,
      step: 0.001,
      defaultValue: 0.01,
      unit: "s",
    },
    {
      name: "Amp Release",
      node: NodeNames.Supersaw,
      event: AuditionerEvents.Supersaw.SetAmpRelease,
      min: 0.01,
      max: 10,
      step: 0.01,
      defaultValue: 0.5,
      unit: "s",
    },
    {
      name: "Filter Attack",
      node: NodeNames.Supersaw,
      event: AuditionerEvents.Supersaw.SetFilterAttack,
      min: 0.001,
      max: 2,
      step: 0.001,
      defaultValue: 0.3,
      unit: "s",
    },
    {
      name: "Filter Release",
      node: NodeNames.Supersaw,
      event: AuditionerEvents.Supersaw.SetFilterRelease,
      min: 0.01,
      max: 10,
      step: 0.01,
      defaultValue: 0.3,
      unit: "s",
    },
  ],
}

// Reverb configuration
const reverbConfig: InstrumentConfig = {
  name: "Reverb",
  color: "teal",
  triggerNode: null, // No trigger for reverb
  parameters: [
    {
      name: "Send",
      node: NodeNames.System,
      event: AuditionerEvents.System.SetReverbSend,
      min: 0,
      max: 1,
      step: 0.01,
      defaultValue: 0.3,
      unit: "%",
    },
    {
      name: "Return",
      node: NodeNames.System,
      event: AuditionerEvents.System.SetReverbReturn,
      min: 0,
      max: 1,
      step: 0.01,
      defaultValue: 0.5,
      unit: "%",
    },
    {
      name: "Size",
      node: NodeNames.Reverb,
      event: AuditionerEvents.Reverb.SetSize,
      min: 0,
      max: 1,
      step: 0.01,
      defaultValue: 0.5,
      unit: "%",
    },
    {
      name: "Feedback",
      node: NodeNames.Reverb,
      event: AuditionerEvents.Reverb.SetFeedback,
      min: 0,
      max: 1,
      step: 0.01,
      defaultValue: 0.2,
      unit: "%",
    },
  ],
}

export function AuditionerPage(): JSX.Element {
  // Switch to auditioner system when this page loads
  useEffect(() => {
    const switchToAuditioner = async () => {
      try {
        await invoke(Commands.SwitchAudioSystem, {
          systemName: SystemNames.Auditioner,
        })
      } catch (error) {
        console.error("Error switching to auditioner system:", error)
      }
    }

    switchToAuditioner()
  }, [])

  return (
    <div className="space-y-8">
      <Auditioner config={kickDrumConfig} />
      <Auditioner config={clapDrumConfig} />
      <Auditioner config={hiHatConfig} />
      <Auditioner config={chordSynthConfig} />
      <Auditioner config={supersawConfig} />
      <Auditioner config={reverbConfig} />
    </div>
  )
}
