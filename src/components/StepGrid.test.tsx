// import { render, screen, fireEvent } from '@testing-library/react';
// import { describe, it, expect, vi } from 'vitest';
// import { StepGrid } from './StepGrid';

// describe('StepGrid', () => {
//   const mockPattern = [
//     true, false, false, false,   // steps 1-4
//     false, false, true, false,   // steps 5-8
//     false, false, false, false,  // steps 9-12
//     false, false, true, false    // steps 13-16
//   ];

//   const defaultProps = {
//     pattern: mockPattern,
//     currentStep: 0,
//     audioStarted: false,
//     onStepToggle: vi.fn(),
//     label: 'Test Pattern'
//   };

//   it('renders all 16 steps', () => {
//     render(<StepGrid {...defaultProps} />);

//     // Check that all 16 step buttons are rendered
//     for (let i = 1; i <= 16; i++) {
//       expect(screen.getByText(i.toString())).toBeInTheDocument();
//     }
//   });

//   it('shows active steps correctly', () => {
//     render(<StepGrid {...defaultProps} />);

//     // Step 1 should be active (black background)
//     const step1 = screen.getByText('1');
//     expect(step1).toHaveClass('bg-black');

//     // Step 2 should be inactive (gray background)
//     const step2 = screen.getByText('2');
//     expect(step2).toHaveClass('bg-gray-600');

//     // Step 7 should be active
//     const step7 = screen.getByText('7');
//     expect(step7).toHaveClass('bg-black');
//   });

//   it('highlights current step when audio is started', () => {
//     render(<StepGrid {...defaultProps} currentStep={2} audioStarted={true} />);

//     // Step 3 (index 2) should have blue underline
//     const step3 = screen.getByText('3');
//     expect(step3).toHaveClass('border-b-4', 'border-b-blue-400');
//   });

//   it('does not highlight current step when audio is stopped', () => {
//     render(<StepGrid {...defaultProps} currentStep={2} audioStarted={false} />);

//     // Step 3 should not have blue underline when audio is stopped
//     const step3 = screen.getByText('3');
//     expect(step3).not.toHaveClass('border-b-4');
//   });

//   it('calls onStepToggle with correct index when step is clicked', () => {
//     const mockToggle = vi.fn();
//     render(<StepGrid {...defaultProps} onStepToggle={mockToggle} />);

//     // Click step 1 (index 0)
//     fireEvent.click(screen.getByText('1'));
//     expect(mockToggle).toHaveBeenCalledWith(0);

//     // Click step 8 (index 7)
//     fireEvent.click(screen.getByText('8'));
//     expect(mockToggle).toHaveBeenCalledWith(7);

//     // Click step 16 (index 15)
//     fireEvent.click(screen.getByText('16'));
//     expect(mockToggle).toHaveBeenCalledWith(15);
//   });

//   it('shows beat indicators on downbeats', () => {
//     render(<StepGrid {...defaultProps} />);

//     // Steps 1, 5, 9, 13 should have yellow ring (downbeats)
//     const step1 = screen.getByText('1');
//     const step5 = screen.getByText('5');
//     const step9 = screen.getByText('9');
//     const step13 = screen.getByText('13');

//     expect(step1).toHaveClass('ring-2', 'ring-yellow-500');
//     expect(step5).toHaveClass('ring-2', 'ring-yellow-500');
//     expect(step9).toHaveClass('ring-2', 'ring-yellow-500');
//     expect(step13).toHaveClass('ring-2', 'ring-yellow-500');

//     // Other steps should not have yellow ring
//     const step2 = screen.getByText('2');
//     expect(step2).not.toHaveClass('ring-yellow-500');
//   });

//   it('handles pattern changes correctly', () => {
//     const { rerender } = render(<StepGrid {...defaultProps} />);

//     // Initially step 1 is active
//     expect(screen.getByText('1')).toHaveClass('bg-black');
//     expect(screen.getByText('2')).toHaveClass('bg-gray-600');

//     // Change pattern - make step 2 active, step 1 inactive
//     const newPattern = [false, true, ...mockPattern.slice(2)];
//     rerender(<StepGrid {...defaultProps} pattern={newPattern} />);

//     expect(screen.getByText('1')).toHaveClass('bg-gray-600');
//     expect(screen.getByText('2')).toHaveClass('bg-black');
//   });

//   it('displays label correctly', () => {
//     render(<StepGrid {...defaultProps} label="Kick Pattern" />);
//     expect(screen.getByText('Kick Pattern')).toBeInTheDocument();
//   });
// });
