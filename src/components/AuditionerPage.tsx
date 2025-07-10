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
      name: "Frequency Mod Amount",
      node: "kick",
      event: "set_frequency_mod_amount",
      min: 0,
      max: 100,
      step: 1,
      defaultValue: 40,
      unit: "hz",
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
    {
      name: "Density",
      node: "clap",
      event: "set_density", 
      min: 0,
      max: 1,
      step: 0.01,
      defaultValue: 0.7,
      unit: "%",
    },
  ],
};

interface AuditionerPageProps {
  onBack: () => void;
  isPaused: boolean;
}

export function AuditionerPage({ onBack, isPaused }: AuditionerPageProps) {
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
    </div>
  );
}