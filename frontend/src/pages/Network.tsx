import React from 'react';
import { useStore } from '../lib/store';
import { api } from '../lib/api';

export function Network() {
  const { networkConfig, setNetworkConfig } = useStore();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    await api.updateNetworkConfig(networkConfig);
  };

  return (
    <div className="max-w-2xl mx-auto">
      <form onSubmit={handleSubmit} className="space-y-8">
        <div className="bg-white p-6 rounded-lg shadow-md">
          <h2 className="text-lg font-semibold mb-4">WiFi Configuration</h2>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700">SSID</label>
              <input
                type="text"
                value={networkConfig.wifi.ssid}
                onChange={(e) =>
                  setNetworkConfig({
                    ...networkConfig,
                    wifi: { ...networkConfig.wifi, ssid: e.target.value },
                  })
                }
                className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700">Password</label>
              <input
                type="password"
                value={networkConfig.wifi.password}
                onChange={(e) =>
                  setNetworkConfig({
                    ...networkConfig,
                    wifi: { ...networkConfig.wifi, password: e.target.value },
                  })
                }
                className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
              />
            </div>
          </div>
        </div>

        <div className="bg-white p-6 rounded-lg shadow-md">
          <h2 className="text-lg font-semibold mb-4">APN Configuration</h2>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700">Name</label>
              <input
                type="text"
                value={networkConfig.apn.name}
                onChange={(e) =>
                  setNetworkConfig({
                    ...networkConfig,
                    apn: { ...networkConfig.apn, name: e.target.value },
                  })
                }
                className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700">Username</label>
              <input
                type="text"
                value={networkConfig.apn.username}
                onChange={(e) =>
                  setNetworkConfig({
                    ...networkConfig,
                    apn: { ...networkConfig.apn, username: e.target.value },
                  })
                }
                className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700">Password</label>
              <input
                type="password"
                value={networkConfig.apn.password}
                onChange={(e) =>
                  setNetworkConfig({
                    ...networkConfig,
                    apn: { ...networkConfig.apn, password: e.target.value },
                  })
                }
                className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
              />
            </div>
          </div>
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