// Event type definitions that match the Rust event structure
// Organized as System -> Node -> Events to mirror backend architecture

// ============================================================================
// AUDITIONER SYSTEM
// ============================================================================
export const Auditioner = {
  // System node events
  System: {
    SetReverbSend: "set_reverb_send",
    SetReverbReturn: "set_reverb_return",
  },

  // Kick node events
  Kick: {
    Trigger: "trigger",
    SetGain: "set_gain",
    SetBaseFrequency: "set_base_frequency",
    SetFrequencyRatio: "set_frequency_ratio",
    SetAmpAttack: "set_amp_attack",
    SetAmpRelease: "set_amp_release",
    SetFreqAttack: "set_freq_attack",
    SetFreqRelease: "set_freq_release",
  },

  // Clap node events
  Clap: {
    Trigger: "trigger",
    SetGain: "set_gain",
  },

  // HiHat node events
  HiHat: {
    Trigger: "trigger",
    SetGain: "set_gain",
    SetLength: "set_length",
  },

  // Chord node events
  Chord: {
    Trigger: "trigger",
    SetGain: "set_gain",
    SetBaseFrequency: "set_base_frequency",
    SetModulationIndex: "set_modulation_index",
    SetFeedback: "set_feedback",
    SetAttack: "set_attack",
    SetRelease: "set_release",
  },

  // Supersaw node events
  Supersaw: {
    Trigger: "trigger",
    SetGain: "set_gain",
    SetBaseFrequency: "set_base_frequency",
    SetDetune: "set_detune",
    SetStereoWidth: "set_stereo_width",
    SetFilterCutoff: "set_filter_cutoff",
    SetFilterResonance: "set_filter_resonance",
    SetFilterEnvAmount: "set_filter_env_amount",
    SetAmpAttack: "set_amp_attack",
    SetAmpRelease: "set_amp_release",
    SetFilterAttack: "set_filter_attack",
    SetFilterRelease: "set_filter_release",
  },

  // Reverb node events
  Reverb: {
    SetSize: "set_size",
    SetModulationDepth: "set_modulation_depth",
    SetFeedback: "set_feedback",
  },
} as const

// ============================================================================
// TRANCE RIFF SYSTEM
// ============================================================================
export const TranceRiff = {
  // System node events
  System: {
    SetBpm: "set_bpm",
    SetPaused: "set_paused",
    SetSequence: "set_sequence",
    ResetSequence: "reset_sequence",
  },

  // Supersaw node events
  Supersaw: {
    Trigger: "trigger",
    SetGain: "set_gain",
    SetBaseFrequency: "set_base_frequency",
    SetDetune: "set_detune",
    SetStereoWidth: "set_stereo_width",
    SetFilterCutoff: "set_filter_cutoff",
    SetFilterResonance: "set_filter_resonance",
    SetFilterEnvAmount: "set_filter_env_amount",
    SetAmpAttack: "set_amp_attack",
    SetAmpRelease: "set_amp_release",
    SetFilterAttack: "set_filter_attack",
    SetFilterRelease: "set_filter_release",
  },
} as const

// ============================================================================
// SYSTEM AND NODE NAMES
// ============================================================================
export const SystemNames = {
  Auditioner: "auditioner",
  TranceRiff: "trance_riff",
} as const

export const NodeNames = {
  System: "system",
  Kick: "kick",
  Clap: "clap",
  HiHat: "hihat",
  Chord: "chord",
  Supersaw: "supersaw",
  Reverb: "reverb",
} as const

// ============================================================================
// TAURI COMMANDS
// ============================================================================
export const Commands = {
  SendClientEvent: "send_client_event",
  SwitchAudioSystem: "switch_audio_system",
} as const

// ============================================================================
// COMMON EVENT NAMES
// ============================================================================
export const CommonEvents = {
  Trigger: "trigger",
} as const
