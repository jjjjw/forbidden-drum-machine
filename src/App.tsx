import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AuditionerPage } from "./components/AuditionerPage";
import "./App.css";

function App() {
  // Switch to auditioner system on startup
  useEffect(() => {
    const initializeAuditioner = async () => {
      try {
        await invoke("switch_audio_system", { systemName: "auditioner" });
      } catch (error) {
        console.error("Error switching to auditioner:", error);
      }
    };
    
    initializeAuditioner();
  }, []);

  return (
    <main className="min-h-screen bg-gray-900 text-white p-8 font-mono">
      <div className="max-w-6xl mx-auto">
        <div className="mb-4">
          <h1 className="text-lg text-neutral-300 mb-6">
            Forbidden Drum Machine - Auditioner
          </h1>
        </div>

        {/* Auditioner System */}
        <AuditionerPage />
      </div>
    </main>
  );
}

export default App;
