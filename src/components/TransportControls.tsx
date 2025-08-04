import React from "react";
import { invoke } from "@tauri-apps/api/core";

interface TransportControlsProps {
  systemName: string;
  isPaused: boolean;
  bpm: number;
  onPausedChange: (paused: boolean) => void;
  onBpmChange: (bpm: number) => void;
  sliderWidth?: string;
}

export const TransportControls: React.FC<TransportControlsProps> = ({
  systemName,
  isPaused,
  bpm,
  onPausedChange,
  onBpmChange,
  sliderWidth = "w-32",
}) => {
  const handlePlayPause = async () => {
    try {
      await invoke("send_audio_event", {
        systemName,
        nodeName: "system",
        eventName: "set_paused",
        parameter: isPaused ? 0.0 : 1.0,
      });
      onPausedChange(!isPaused);
    } catch (error) {
      console.error("Failed to toggle playback:", error);
    }
  };


  const handleBpmChange = async (newBpm: number) => {
    try {
      await invoke("send_audio_event", {
        systemName,
        nodeName: "system",
        eventName: "set_bpm",
        parameter: newBpm,
      });
      onBpmChange(newBpm);
    } catch (error) {
      console.error("Failed to update BPM:", error);
    }
  };

  return (
    <div className="flex items-center gap-4">
      <button
        onClick={handlePlayPause}
        className={`px-6 py-2 rounded ${
          isPaused
            ? "bg-green-600 hover:bg-green-700"
            : "bg-red-600 hover:bg-red-700"
        }`}
      >
        {isPaused ? "Play" : "Pause"}
      </button>

      <div className="flex items-center gap-2">
        <label className="text-sm">BPM:</label>
        <input
          type="range"
          min="60"
          max="200"
          value={bpm}
          onChange={(e) => handleBpmChange(parseInt(e.target.value))}
          className={sliderWidth}
        />
        <span className="text-sm w-8">{bpm}</span>
      </div>
    </div>
  );
};
