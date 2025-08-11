import { useState } from "react"
import { AuditionerPage } from "./components/AuditionerPage"
import { TranceRiffPage } from "./components/TranceRiffPage"
import "./App.css"

type SystemTab = "auditioner" | "tranceriff"

function App() {
  const [activeTab, setActiveTab] = useState<SystemTab>("auditioner")

  const handleTabChange = (tab: SystemTab) => {
    setActiveTab(tab)
  }

  return (
    <main className="min-h-screen bg-gray-900 text-white p-8 font-mono">
      <div className="max-w-6xl mx-auto">
        <div className="mb-6">
          <h1 className="text-lg text-neutral-300 mb-6">
            Forbidden Drum Machine
          </h1>

          {/* Tab Navigation */}
          <div className="flex space-x-4 mb-6">
            <button
              onClick={() => handleTabChange("auditioner")}
              className={`px-6 py-2 rounded-lg font-medium transition-colors ${
                activeTab === "auditioner"
                  ? "bg-green-600 text-white"
                  : "bg-gray-700 text-gray-300 hover:bg-gray-600"
              }`}
            >
              Auditioner
            </button>
            <button
              onClick={() => handleTabChange("tranceriff")}
              className={`px-6 py-2 rounded-lg font-medium transition-colors ${
                activeTab === "tranceriff"
                  ? "bg-green-600 text-white"
                  : "bg-gray-700 text-gray-300 hover:bg-gray-600"
              }`}
            >
              Trance Riff
            </button>
          </div>
        </div>

        {/* Tab Content */}
        {activeTab === "auditioner" && <AuditionerPage />}
        {activeTab === "tranceriff" && <TranceRiffPage />}
      </div>
    </main>
  )
}

export default App
