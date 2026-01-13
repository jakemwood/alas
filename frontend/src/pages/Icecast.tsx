import React, { useEffect, useState } from "react";
import { useStore } from "../lib/store";
import { useApi } from "../lib/api";
import { EmptyState } from "../components/EmptyState";

export function Icecast() {
  const { icecastConfig, setIcecastConfig } = useStore();
  const [isConfigured, setIsConfigured] = useState(false);
  const [showForm, setShowForm] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const api = useApi();

  useEffect(() => {
    api
      .getIcecastConfig()
      .then((response) => {
        setIcecastConfig({
          host: response.hostname,
          port: response.port,
          mountPoint: response.mount,
          password: response.password,
        });
        setIsConfigured(true);
        setShowForm(true);
      })
      .catch((error) => {
        if (error?.status === 404) {
          setIsConfigured(false);
        } else {
          console.error("Failed to fetch Icecast config:", error);
        }
      })
      .finally(() => setIsLoading(false));
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const results = await api.updateIcecastConfig({
      hostname: icecastConfig.host,
      port: icecastConfig.port,
      mount: icecastConfig.mountPoint,
      password: icecastConfig.password,
    });
    setIsConfigured(true);
    console.log(results);
  };

  const handleDelete = async () => {
    if (confirm("Are you sure you want to disable streaming?")) {
      await api.deleteIcecastConfig();
      setIsConfigured(false);
      setShowForm(false);
      setIcecastConfig({
        host: "",
        port: 8000,
        mountPoint: "",
        password: "",
      });
    }
  };

  if (isLoading) {
    return <div className="max-w-2xl mx-auto text-center py-12">Loading...</div>;
  }

  if (!isConfigured && !showForm) {
    return (
      <div className="max-w-2xl mx-auto">
        <EmptyState
          title="Streaming Not Configured"
          description="Configure Icecast streaming to broadcast your audio to listeners over the internet."
          actionLabel="Set Up Streaming"
          onAction={() => setShowForm(true)}
        />
      </div>
    );
  }

  return (
    <div className="max-w-2xl mx-auto">
      <form
        onSubmit={handleSubmit}
        className="bg-white p-6 rounded-lg shadow-md space-y-6"
      >
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-lg font-semibold">Icecast Configuration</h2>
          {isConfigured && (
            <button
              type="button"
              onClick={handleDelete}
              className="text-red-600 hover:text-red-800 text-sm"
            >
              Disable Streaming
            </button>
          )}
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700">
            Host
          </label>
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
          <label className="block text-sm font-medium text-gray-700">
            Port
          </label>
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
          <label className="block text-sm font-medium text-gray-700">
            Password
          </label>
          <input
            type="text"
            value={icecastConfig.password}
            onChange={(e) =>
              setIcecastConfig({
                ...icecastConfig,
                password: e.target.value,
              })
            }
            className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
            placeholder="super secret password"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700">
            Mount Point
          </label>
          <div className="mt-1 flex rounded-md shadow-sm">
            <span className="inline-flex items-center px-3 rounded-l-md border border-r-0 border-gray-300 bg-gray-50 text-gray-500 sm:text-sm">
              /
            </span>
            <input
              type="text"
              value={icecastConfig.mountPoint.replace(/^\//, "")}
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
