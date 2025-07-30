import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Auditioner, InstrumentConfig } from "./Auditioner";

// Kick drum configuration
const kickDrumConfig: InstrumentConfig = {
  name: "Kick Drum",
  color: "red",
  triggerNode: "kick",
  parameters: [
    {
      name: "Gain",
      node: "kick",
      event: "set_gain",
      min: 0,
      max: 2,
      step: 0.01,
      defaultValue: 0.8,
      unit: "%",
    },
    {
      name: "Base Frequency",
      node: "kick",
      event: "set_base_frequency", 
      min: 20,
      max: 200,
      step: 1,
      defaultValue: 60,
      unit: "hz",
    },
    {
      name: "Frequency Ratio",
      node: "kick",
      event: "set_frequency_ratio",
      min: 1,
      max: 20,
      step: 0.1,
      defaultValue: 7.0,
      unit: "x",
    },
    {
      name: "Amp Attack",
      node: "kick",
      event: "set_amp_attack",
      min: 0.001,
      max: 0.1,
      step: 0.001,
      defaultValue: 0.005,
      unit: "ms",
    },
    {
      name: "Amp Release", 
      node: "kick",
      event: "set_amp_release",
      min: 0.01,
      max: 2.0,
      step: 0.01,
      defaultValue: 0.2,
      unit: "ms",
    },
    {
      name: "Freq Attack",
      node: "kick",
      event: "set_freq_attack", 
      min: 0.001,
      max: 0.1,
      step: 0.001,
      defaultValue: 0.002,
      unit: "ms",
    },
    {
      name: "Freq Release",
      node: "kick",
      event: "set_freq_release",
      min: 0.001,
      max: 0.2,
      step: 0.001,
      defaultValue: 0.05,
      unit: "ms",
    },
  ],
};

// Clap drum configuration  
const clapDrumConfig: InstrumentConfig = {
  name: "Clap Drum",
  color: "blue", 
  triggerNode: "clap",
  parameters: [
    {
      name: "Gain",
      node: "clap", 
      event: "set_gain",
      min: 0,
      max: 2,
      step: 0.01,
      defaultValue: 0.6,
      unit: "%",
    },
  ],
};

// Hi-hat configuration
const hiHatConfig: InstrumentConfig = {
  name: "Hi-Hat",
  color: "yellow",
  triggerNode: "hihat",
  parameters: [
    {
      name: "Gain",
      node: "hihat",
      event: "set_gain",
      min: 0,
      max: 2,
      step: 0.01,
      defaultValue: 1.0,
      unit: "%",
    },
    {
      name: "Length",
      node: "hihat", 
      event: "set_amp_release",
      min: 0.002,
      max: 0.5,
      step: 0.001,
      defaultValue: 0.05,
      unit: "ms",
    },
  ],
};

// Chord synth configuration
const chordSynthConfig: InstrumentConfig = {
  name: "Chord Synth",
  color: "purple",
  triggerNode: "chord",
  parameters: [
    {
      name: "Gain",
      node: "chord",
      event: "set_gain",
      min: 0,
      max: 1,
      step: 0.01,
      defaultValue: 0.25,
      unit: "%",
    },
    {
      name: "Base Frequency",
      node: "chord",
      event: "set_base_frequency",
      min: 110,
      max: 440,
      step: 1,
      defaultValue: 220,
      unit: "hz",
    },
    {
      name: "Modulation Index",
      node: "chord",
      event: "set_modulation_index",
      min: 0,
      max: 2,
      step: 0.01,
      defaultValue: 0.5,
      unit: "x",
    },
    {
      name: "Feedback",
      node: "chord",
      event: "set_feedback",
      min: 0,
      max: 0.99,
      step: 0.01,
      defaultValue: 0.1,
      unit: "%",
    },
    {
      name: "Attack",
      node: "chord",
      event: "set_amp_attack",
      min: 0.01,
      max: 2,
      step: 0.01,
      defaultValue: 0.5,
      unit: "s",
    },
    {
      name: "Release",
      node: "chord",
      event: "set_amp_release",
      min: 0.1,
      max: 8,
      step: 0.1,
      defaultValue: 4,
      unit: "s",
    },
  ],
};

// Reverb configuration  
const reverbConfig: InstrumentConfig = {
  name: "Reverb",
  color: "teal", 
  triggerNode: null, // No trigger for reverb
  parameters: [
    {
      name: "Send",
      node: "system",
      event: "set_reverb_send",
      min: 0,
      max: 1,
      step: 0.01,
      defaultValue: 0.3,
      unit: "%",
    },
    {
      name: "Return",
      node: "system", 
      event: "set_reverb_return",
      min: 0,
      max: 1,
      step: 0.01,
      defaultValue: 0.5,
      unit: "%",
    },
    {
      name: "Size",
      node: "reverb",
      event: "set_size",
      min: 0,
      max: 1,
      step: 0.01,
      defaultValue: 0.5,
      unit: "%",
    },
    {
      name: "Feedback",
      node: "reverb",
      event: "set_feedback",
      min: 0,
      max: 1,
      step: 0.01,
      defaultValue: 0.5,
      unit: "%",
    },
  ],
};

export function AuditionerPage(): JSX.Element {
  // Switch to auditioner system when this page loads
  useEffect(() => {
    const switchToAuditioner = async () => {
      try {
        await invoke("switch_audio_system", { system_name: "auditioner" });
      } catch (error) {
        console.error("Error switching to auditioner system:", error);
      }
    };
    
    switchToAuditioner();
  }, []);

  return (
    <div className="space-y-8">
      <Auditioner config={kickDrumConfig} />
      <Auditioner config={clapDrumConfig} />
      <Auditioner config={hiHatConfig} />
      <Auditioner config={chordSynthConfig} />
      <Auditioner config={reverbConfig} />
    </div>
  );
}