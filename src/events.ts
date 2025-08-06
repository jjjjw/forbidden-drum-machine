// Event type definitions that match the Rust event structure

export enum CommonEvent {
  Trigger = "trigger",
  SetGain = "set_gain",
}

export enum InstrumentEvent {
  SetBaseFrequency = "instrument.set_base_frequency",
  SetFrequencyRatio = "instrument.set_frequency_ratio",
  SetModulationIndex = "instrument.set_modulation_index",
  SetAmpAttack = "instrument.set_amp_attack",
  SetAmpRelease = "instrument.set_amp_release",
  SetFreqAttack = "instrument.set_freq_attack",
  SetFreqRelease = "instrument.set_freq_release",
}

export enum DelayEvent {
  SetDelaySeconds = "delay.set_delay_seconds",
  SetFeedback = "delay.set_feedback",
  SetFreeze = "delay.set_freeze",
  SetHighpassFreq = "delay.set_highpass_freq",
  SetLowpassFreq = "delay.set_lowpass_freq",
}

export enum ReverbEvent {
  SetSize = "reverb.set_size",
  SetModulationDepth = "reverb.set_modulation_depth",
}

export enum SystemEvent {
  SetBpm = "set_bpm",
  SetPaused = "set_paused",
  SetMarkovDensity = "set_markov_density",
  SetKickLoopBias = "set_kick_loop_bias",
  SetClapLoopBias = "set_clap_loop_bias",
  GenerateKickPattern = "generate_kick_pattern",
  GenerateClapPattern = "generate_clap_pattern",
  SetDelaySend = "set_delay_send",
  SetReverbSend = "set_reverb_send",
  SetDelayReturn = "set_delay_return",
  SetReverbReturn = "set_reverb_return",
  // Euclidean sequencer events
  SetKickSteps = "set_kick_steps",
  SetKickBeats = "set_kick_beats",
  SetKickTempoMult = "set_kick_tempo_mult",
  SetClapSteps = "set_clap_steps",
  SetClapBeats = "set_clap_beats",
  SetClapTempoMult = "set_clap_tempo_mult",
  SetHihatSteps = "set_hihat_steps",
  SetHihatBeats = "set_hihat_beats",
  SetHihatTempoMult = "set_hihat_tempo_mult",
  SetChordSteps = "set_chord_steps",
  SetChordBeats = "set_chord_beats",
  SetChordTempoMult = "set_chord_tempo_mult",
}

export enum SequencerEvent {
  SetSteps = "sequencer.set_steps",
  SetBeats = "sequencer.set_beats",
  SetTempoMultiplier = "sequencer.set_tempo_multiplier",
  SetOffset = "sequencer.set_offset",
}

// Combine all events for convenience
export const AudioEvent = {
  Common: CommonEvent,
  Instrument: InstrumentEvent,
  Delay: DelayEvent,
  Reverb: ReverbEvent,
  System: SystemEvent,
  Sequencer: SequencerEvent,
} as const;

// System names
export enum SystemName {
  DrumMachine = "drum_machine",
  Auditioner = "auditioner",
  Euclidean = "euclidean",
}

// Node names
export enum NodeName {
  Kick = "kick",
  Clap = "clap",
  HiHat = "hihat",
  Chord = "chord",
  Delay = "delay",
  Reverb = "reverb",
  System = "system",
}