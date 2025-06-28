interface StepGridProps {
  pattern: boolean[];
  currentStep: number;
  audioStarted: boolean;
  onStepToggle: (index: number) => void;
  label: string;
}

export function StepGrid({
  pattern,
  currentStep,
  audioStarted,
  onStepToggle,
  label,
}: StepGridProps) {
  return (
    <div className="bg-gray-800 rounded-xl p-6 border border-gray-700">
      <h2 className="text-2xl font-bold mb-4 text-white">{label}</h2>
      <div className="grid grid-cols-4 gap-3 max-w-md mx-auto">
        {/* First row: steps 1-4 */}
        {pattern.slice(0, 4).map((active, index) => (
          <button
            key={`${label}-${index}`}
            className={`
              aspect-square rounded-lg border-2 font-bold text-sm transition-all hover:scale-110 relative
              ${
                active
                  ? "bg-black border-gray-400 text-white shadow-lg"
                  : "bg-gray-600 border-gray-500 text-gray-300 hover:bg-gray-500"
              }
              ${currentStep === index && audioStarted ? "border-b-4 border-b-blue-400" : ""}
            `}
            onClick={() => onStepToggle(index)}
          >
            {index + 1}
          </button>
        ))}

        {/* Second row: steps 5-8 */}
        {pattern.slice(4, 8).map((active, index) => (
          <button
            key={`${label}-${index + 4}`}
            className={`
              aspect-square rounded-lg border-2 font-bold text-sm transition-all hover:scale-110 relative
              ${
                active
                  ? "bg-black border-gray-400 text-white shadow-lg"
                  : "bg-gray-600 border-gray-500 text-gray-300 hover:bg-gray-500"
              }
              ${currentStep === index + 4 && audioStarted ? "border-b-4 border-b-blue-400" : ""}
            `}
            onClick={() => onStepToggle(index + 4)}
          >
            {index + 5}
          </button>
        ))}

        {/* Third row: steps 9-12 */}
        {pattern.slice(8, 12).map((active, index) => (
          <button
            key={`${label}-${index + 8}`}
            className={`
              aspect-square rounded-lg border-2 font-bold text-sm transition-all hover:scale-110 relative
              ${
                active
                  ? "bg-black border-gray-400 text-white shadow-lg"
                  : "bg-gray-600 border-gray-500 text-gray-300 hover:bg-gray-500"
              }
              ${currentStep === index + 8 && audioStarted ? "border-b-4 border-b-blue-400" : ""}
            `}
            onClick={() => onStepToggle(index + 8)}
          >
            {index + 9}
          </button>
        ))}

        {/* Fourth row: steps 13-16 */}
        {pattern.slice(12, 16).map((active, index) => (
          <button
            key={`${label}-${index + 12}`}
            className={`
              aspect-square rounded-lg border-2 font-bold text-sm transition-all hover:scale-110 relative
              ${
                active
                  ? "bg-black border-gray-400 text-white shadow-lg"
                  : "bg-gray-600 border-gray-500 text-gray-300 hover:bg-gray-500"
              }
              ${currentStep === index + 12 && audioStarted ? "border-b-4 border-b-blue-400" : ""}
            `}
            onClick={() => onStepToggle(index + 12)}
          >
            {index + 13}
          </button>
        ))}
      </div>
    </div>
  );
}
