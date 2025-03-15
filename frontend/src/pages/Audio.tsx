import React, { useEffect } from "react";
import { useStore } from "../lib/store";
import { useApi } from "../lib/api";

export function Audio() {
  const { audioConfig, setAudioConfig } = useStore();
  const api = useApi();

  useEffect(() => {
    /*
    pub silence_duration_before_deactivation: u32,
    pub silence_threshold: f32,
    */
    api.getAudioConfig().then((response) => {
      setAudioConfig({
        silenceDuration: response.silence_duration_before_deactivation,
        silenceThreshold: response.silence_threshold,
      });
    });
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    await api.updateAudioConfig({
      silence_duration_before_deactivation: parseInt(audioConfig.silenceDuration),
      silence_threshold: parseInt(audioConfig.silenceThreshold),
    });
  };

  return (
    <div className="max-w-2xl mx-auto">
      <form
        onSubmit={handleSubmit}
        className="bg-white p-6 rounded-lg shadow-md space-y-6"
      >
        <h2 className="text-lg font-semibold mb-4">Audio Configuration</h2>

        <div>
          <label className="block text-sm font-medium text-gray-700">
            Silence Duration for Deactivation (seconds)
          </label>
          <input
            type="number"
            min="0"
            value={audioConfig.silenceDuration}
            onChange={(e) =>
              setAudioConfig({
                ...audioConfig,
                silenceDuration: e.target.value,
              })
            }
            className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700">
            Silence Threshold (dB)
          </label>
          <input
            type="number"
            max="0"
            value={audioConfig.silenceThreshold}
            onChange={(e) =>
              setAudioConfig({
                ...audioConfig,
                silenceThreshold: e.target.value,
              })
            }
            className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
          />
          <p className="mt-1 text-sm text-gray-500">
            Audio levels below this threshold will be considered silence
          </p>
        </div>

        <div className="flex justify-end">
          <button
            type="submit"
            className="bg-blue-500 text-white px-4 py-2 rounded-md hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
          >
            Save Changes
          </button>
        </div>
      </form>
    </div>
  );
}
