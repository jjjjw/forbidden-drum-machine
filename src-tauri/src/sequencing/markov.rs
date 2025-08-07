// use fastrand;

// /// Simple Markov chain for generating drum events
// #[derive(Clone)]
// pub struct MarkovChain {
//     /// Transition probability matrix [state][next_state]
//     /// state 0 = silence, state 1 = event
//     transitions: [[f32; 2]; 2],
//     current_state: usize,
//     density: f32, // Overall event density 0.0 - 1.0
// }

// impl MarkovChain {
//     pub fn new(density: f32) -> Self {
//         let density = density.clamp(0.0, 1.0);

//         // Create transition matrix based on density
//         // Higher density = more likely to stay in event state and transition to events
//         let silence_to_silence = 1.0 - density;
//         let silence_to_event = density;
//         let event_to_silence = 0.7; // Tend to not have long runs of events
//         let event_to_event = 0.3;

//         Self {
//             transitions: [
//                 [silence_to_silence, silence_to_event], // From silence
//                 [event_to_silence, event_to_event],     // From event
//             ],
//             current_state: 0, // Start in silence
//             density,
//         }
//     }

//     pub fn set_density(&mut self, density: f32) {
//         self.density = density.clamp(0.0, 1.0);

//         // Update transition matrix
//         let silence_to_silence = 1.0 - self.density;
//         let silence_to_event = self.density;
//         let event_to_silence = 0.7;
//         let event_to_event = 0.3;

//         self.transitions = [
//             [silence_to_silence, silence_to_event],
//             [event_to_silence, event_to_event],
//         ];
//     }

//     /// Generate next state (true = event, false = silence)
//     pub fn next(&mut self) -> bool {
//         let rand_val = fastrand::f32();
//         let current_transitions = &self.transitions[self.current_state];

//         // Determine next state based on probabilities
//         if rand_val < current_transitions[0] {
//             self.current_state = 0; // Silence
//         } else {
//             self.current_state = 1; // Event
//         }

//         self.current_state == 1
//     }

//     /// Generate a sequence of events
//     pub fn generate_sequence(&mut self, length: usize) -> Vec<bool> {
//         (0..length).map(|_| self.next()).collect()
//     }

//     pub fn reset(&mut self) {
//         self.current_state = 0;
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_markov_chain_creation() {
//         let chain = MarkovChain::new(0.5);
//         assert_eq!(chain.density, 0.5);
//         assert_eq!(chain.current_state, 0);
//     }

//     #[test]
//     fn test_markov_chain_density_bounds() {
//         let chain = MarkovChain::new(-0.5);
//         assert_eq!(chain.density, 0.0);

//         let chain = MarkovChain::new(1.5);
//         assert_eq!(chain.density, 1.0);
//     }

//     #[test]
//     fn test_markov_chain_sequence_generation() {
//         let mut chain = MarkovChain::new(0.5);
//         let sequence = chain.generate_sequence(16);
//         assert_eq!(sequence.len(), 16);

//         // With 50% density, we should get some events
//         let event_count = sequence.iter().filter(|&&x| x).count();
//         assert!(event_count >= 0 && event_count <= 16);
//     }

//     #[test]
//     fn test_markov_chain_set_density() {
//         let mut chain = MarkovChain::new(0.5);
//         chain.set_density(0.8);
//         assert_eq!(chain.density, 0.8);

//         // Test bounds
//         chain.set_density(2.0);
//         assert_eq!(chain.density, 1.0);
//     }
// }
