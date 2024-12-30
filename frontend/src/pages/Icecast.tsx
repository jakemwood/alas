import React from 'react';
import { useStore } from '../lib/store';
import { api } from '../lib/api';

export function Icecast() {
  const { icecastConfig, setIcecastConfig } = useStore();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    await api.updateIcecastConfig(icecastConfig);
  };

  return (
    <div className="max-w-2xl mx-auto">
      <form onSubmit={handleSubmit} className="bg-white p-6 rounded-lg shadow-md space-y-6">
        <h2 className="text-lg font-semibold mb-4">Icecast Configuration</h2>

        <div>
          <label className="block text-sm font-medium text-gray-700">Host</label>
          <input
            type="text"
            value={icecastConfig.host}
            onChange={(e) =>
              setIcecastConfig({
                ...icecastConfig,
                host: e.target.value,
              })
            }
            className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
            placeholder="icecast.example.com"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700">Port</label>
          <input
            type="number"
            value={icecastConfig.port}
            onChange={(e) =>
              setIcecastConfig({
                ...icecastConfig,
                port: parseInt(e.target.value),
              })
            }
            className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
            placeholder="8000"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700">Mount Point</label>
          <div className="mt-1 flex rounded-md shadow-sm">
            <span className="inline-flex items-center px-3 rounded-l-md border border-r-0 border-gray-300 bg-gray-50 text-gray-500 sm:text-sm">
              /
            </span>
            <input
              type="text"
              value={icecastConfig.mountPoint.replace(/^\//, '')}
              onChange={(e) =>
                setIcecastConfig({
                  ...icecastConfig,
                  mountPoint: `/${e.target.value}`,
                })
              }
              className="flex-1 min-w-0 block w-full px-3 py-2 rounded-none rounded-r-md border-gray-300 focus:border-blue-500 focus:ring-blue-500 sm:text-sm"
              placeholder="stream"
            />
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