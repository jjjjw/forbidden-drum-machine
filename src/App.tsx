import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { DrumMachinePage } from "./components/DrumMachinePage";
import { AuditionerPage } from "./components/AuditionerPage";
import { EuclideanPage } from "./components/EuclideanPage";
import "./App.css";

function App() {
  // Navigation state
  const [currentSystem, setCurrentSystem] = useState<
    "drum_machine" | "auditioner" | "euclidean"
  >("drum_machine");

  // Handle system switching
  const switchSystem = async (
    systemName: "drum_machine" | "auditioner" | "euclidean",
  ) => {
    try {
      await invoke("switch_audio_system", { systemName });
      setCurrentSystem(systemName);
    } catch (error) {
      console.error(`Error switching to ${systemName}:`, error);
    }
  };

  return (
    <main className="min-h-screen bg-gray-900 text-white p-8 font-mono">
      <div className="max-w-6xl mx-auto">
        <div className="mb-4">
          <h1 className="text-lg text-neutral-300 mb-6">
            Forbidden Drum Machine
          </h1>

          {/* System Tabs */}
          <div className="flex gap-2 mb-6">
            <button
              onClick={() => switchSystem("drum_machine")}
              className={`px-4 py-2 rounded-md transition-all ${
                currentSystem === "drum_machine"
                  ? "bg-blue-600 text-white"
                  : "bg-gray-700 text-gray-300 hover:bg-gray-600"
              }`}
            >
              Drum Machine
            </button>
            <button
              onClick={() => switchSystem("euclidean")}
              className={`px-4 py-2 rounded-md transition-all ${
                currentSystem === "euclidean"
                  ? "bg-green-600 text-white"
                  : "bg-gray-700 text-gray-300 hover:bg-gray-600"
              }`}
            >
              Euclidean
            </button>
            <button
              onClick={() => switchSystem("auditioner")}
              className={`px-4 py-2 rounded-md transition-all ${
                currentSystem === "auditioner"
                  ? "bg-purple-600 text-white"
                  : "bg-gray-700 text-gray-300 hover:bg-gray-600"
              }`}
            >
              Auditioner
            </button>
          </div>
        </div>

        {/* Render system-specific content */}
        {currentSystem === "drum_machine" && <DrumMachinePage />}

        {/* Auditioner System */}
        {currentSystem === "auditioner" && <AuditionerPage />}

        {/* Euclidean System */}
        {currentSystem === "euclidean" && <EuclideanPage />}
      </div>
    </main>
  );
}

export default App;
