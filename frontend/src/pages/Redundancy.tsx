import React, { useEffect } from "react";
import { useStore } from "../lib/store";
import { useApi } from "../lib/api";

export function Redundancy() {
  const { redundancyConfig, setRedundancyConfig } = useStore();
  const api = useApi();

  useEffect(() => {
    api.getRedundancyConfig().then((response) => {
      setRedundancyConfig({
        serverIp: response.server_ip,
        port: response.port,
        publicKey: response.public_key,
      });
    });
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    await api.updateRedundancyConfig({
      server_ip: redundancyConfig.serverIp,
      port: redundancyConfig.port,
      public_key: redundancyConfig.publicKey,
    });
  };

  return (
    <div className="max-w-2xl mx-auto">
      <form
        onSubmit={handleSubmit}
        className="bg-white p-6 rounded-lg shadow-md space-y-6"
      >
        <h2 className="text-lg font-semibold mb-4">Redundancy Configuration</h2>

        <div>
          <label className="block text-sm font-medium text-gray-700">
            Server IP
          </label>
          <input
            type="text"
            value={redundancyConfig.serverIp}
            onChange={(e) =>
              setRedundancyConfig({
                ...redundancyConfig,
                serverIp: e.target.value,
              })
            }
            className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
            placeholder="10.88.7.101"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700">
            Port
          </label>
          <input
            type="number"
            value={redundancyConfig.port}
            onChange={(e) =>
              setRedundancyConfig({
                ...redundancyConfig,
                port: parseInt(e.target.value),
              })
            }
            className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
            placeholder="56882"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700">
            Public Key
          </label>
          <textarea
            value={redundancyConfig.publicKey}
            onChange={(e) =>
              setRedundancyConfig({
                ...redundancyConfig,
                publicKey: e.target.value,
              })
            }
            rows={3}
            className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 font-mono text-sm"
            placeholder="Enter WireGuard public key..."
          />
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
