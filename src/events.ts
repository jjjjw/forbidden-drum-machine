// Event type definitions that match the Rust event structure
// Source of truth for all client events organized by System -> Node -> Events

// ============================================================================
// SYSTEM NAMES - Active systems only
// ============================================================================
export enum SystemName {
  Auditioner = "auditioner",
  TranceRiff = "trance_riff",
}

// ============================================================================
// NODE NAMES - Organized by system
// ============================================================================
export enum NodeName {
  // System-level nodes (available in all systems)
  System = "system",
  
  // Auditioner system nodes
  Kick = "kick",
  Clap = "clap",
  HiHat = "hihat", 
  Chord = "chord",
  Supersaw = "supersaw",
  Delay = "delay",
  Reverb = "reverb",
}

// ============================================================================
// EVENTS - Organized by node type
// ============================================================================

// System node events (available in all systems)
export enum SystemEvents {
  SetBpm = "set_bpm",
  SetPaused = "set_paused",
  SetDelaySend = "set_delay_send",
  SetReverbSend = "set_reverb_send", 
  SetDelayReturn = "set_delay_return",
  SetReverbReturn = "set_reverb_return",
}

// TranceRiff system-specific events
export enum TranceRiffSystemEvents {
  SetSequence = "set_sequence",
  ResetSequence = "reset_sequence",
}

// Common instrument events
export enum CommonInstrumentEvents {
  Trigger = "trigger",
  SetGain = "set_gain",
  SetBaseFrequency = "set_base_frequency",
}

// Kick drum specific events  
export enum KickEvents {
  SetFrequencyRatio = "set_frequency_ratio",
  SetAmpAttack = "set_amp_attack",
  SetAmpRelease = "set_amp_release", 
  SetFreqAttack = "set_freq_attack",
  SetFreqRelease = "set_freq_release",
}

// Clap drum events (only common events)
// HiHat events
export enum HiHatEvents {
  SetLength = "set_length",
}

// Chord synth events
export enum ChordEvents {
  SetModulationIndex = "set_modulation_index",
  SetFeedback = "set_feedback",
  SetAttack = "set_attack",
  SetRelease = "set_release",
}

// Supersaw synth events
export enum SupersawEvents {
  SetDetune = "set_detune",
  SetStereoWidth = "set_stereo_width",
  SetFilterCutoff = "set_filter_cutoff",
  SetFilterResonance = "set_filter_resonance",
  SetFilterEnvAmount = "set_filter_env_amount",
  SetAmpAttack = "set_amp_attack",
  SetAmpRelease = "set_amp_release",
  SetFilterAttack = "set_filter_attack", 
  SetFilterRelease = "set_filter_release",
}

// Delay events
export enum DelayEvents {
  SetDelaySeconds = "set_delay_seconds",
  SetFeedback = "set_feedback",
  SetFreeze = "set_freeze",
  SetHighpassFreq = "set_highpass_freq",
  SetLowpassFreq = "set_lowpass_freq",
}

// Reverb events
export enum ReverbEvents {
  SetSize = "set_size",
  SetModulationDepth = "set_modulation_depth",
}

// ============================================================================
// CONVENIENCE GROUPINGS - For easier access in components
// ============================================================================
export const ClientEvent = {
  // System events
  System: SystemEvents,
  TranceRiffSystem: TranceRiffSystemEvents,
  
  // Instrument events
  Common: CommonInstrumentEvents,
  Kick: KickEvents,
  HiHat: HiHatEvents,
  Chord: ChordEvents,
  Supersaw: SupersawEvents,
  Delay: DelayEvents,
  Reverb: ReverbEvents,
} as const;