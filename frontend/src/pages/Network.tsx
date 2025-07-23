import React, { useEffect, useState, useCallback, useRef } from "react";
import { useStore } from "../lib/store";
import { useApi } from "../lib/api";
import { NetworkCard } from "../components/NetworkCard";
import { AvailableNetwork } from "../types";
import { Wifi, WifiOff, Loader2 } from "lucide-react";

function useNetworks(isConnecting: boolean) {
  const [isLoading, setIsLoading] = useState(true);
  const [networks, setNetworks] = useState<AvailableNetwork[]>([]);
  const api = useApi();
  const intervalRef = useRef<number>();

  useEffect(() => {
    const loadWiFi = async () => {
      try {
        const results = await api.getAvailableWifi();
        setNetworks(results.networks);
        setIsLoading(false);
      } catch (error) {
        console.error('Failed to load WiFi networks:', error);
        setIsLoading(false);
      }
    };

    // Initial load
    loadWiFi();

    // Set up polling only if not connecting
    if (!isConnecting) {
      intervalRef.current = setInterval(loadWiFi, 5000);
    }

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, [isConnecting]); // Only depend on isConnecting, not api or loadWiFi

  return { isLoading, networks };
}

export function Network() {
  const api = useApi();
  const { networkConfig, setNetworkConfig } = useStore();
  const [selectedNetwork, setSelectedNetwork] = useState<AvailableNetwork | null>(null);
  const [password, setPassword] = useState("");
  const [isConnecting, setIsConnecting] = useState(false);
  const { isLoading, networks } = useNetworks(isConnecting);

  useEffect(() => {
    // Load network status and cellular config
    Promise.all([
      api.getNetworkStatus(),
      api.getCellularConfig()
    ]).then(([networkStatus, cellularConfig]) => {
      setNetworkConfig({
        ...networkConfig,
        imei: networkStatus.imei,
        apn: { ...networkConfig.apn, name: cellularConfig.apn }
      });
    });
  }, []);

  const handleWifiSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    await api.updateWifiConfig(networkConfig.wifi.ssid, networkConfig.wifi.password);
  };

  const handleConnect = async () => {
    if (!selectedNetwork) {
      return;
    }

    setIsConnecting(true);
    try {
      await api.connectToWifi(selectedNetwork.ap_path, selectedNetwork.security ? password : undefined);
      // Update the network config to reflect the new connection
      setNetworkConfig({
        ...networkConfig,
        wifi: { ssid: selectedNetwork.ssid, password: selectedNetwork.security ? password : "" }
      });
    } catch (error) {
      console.error('Failed to connect to WiFi:', error);
    } finally {
      setIsConnecting(false);
      setSelectedNetwork(null);
      setPassword("");
    }
  };

  const handleCellularSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    await api.updateCellularConfig(networkConfig.apn.name);
  };

  return (
    <div className="max-w-2xl mx-auto space-y-8">
      <div className="bg-white p-6 rounded-lg shadow-md">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-lg font-semibold">WiFi Networks</h2>
          <Wifi className="w-6 h-6 text-gray-400" />
        </div>
        
        {isLoading ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="w-8 h-8 animate-spin text-indigo-600" />
            <span className="ml-2 text-gray-600">Loading networks...</span>
          </div>
        ) : networks.length === 0 ? (
          <div className="text-center py-8">
            <WifiOff className="w-12 h-12 mx-auto text-gray-400 mb-4" />
            <p className="text-gray-500">No networks found</p>
          </div>
        ) : (
          <div className="max-h-96 overflow-y-auto space-y-4">
            {networks.map((network) => (
              <NetworkCard
                key={network.ap_path}
                network={network}
                isSelected={selectedNetwork?.ap_path === network.ap_path}
                isCurrentNetwork={networkConfig.wifi.ssid === network.ssid}
                password={password}
                isConnecting={isConnecting}
                onSelect={(network) =>
                  setSelectedNetwork(
                    selectedNetwork?.ap_path === network.ap_path
                      ? null
                      : network,
                  )
                }
                onPasswordChange={setPassword}
                onConnect={handleConnect}
              />
            ))}
          </div>
        )}
        
        {networkConfig.wifi.ssid && (
          <div className="mt-6 p-4 bg-gray-50 rounded-lg">
            <h3 className="text-sm font-medium text-gray-700 mb-2">Current Connection</h3>
            <p className="text-sm text-gray-600">Connected to: <span className="font-medium">{networkConfig.wifi.ssid}</span></p>
          </div>
        )}
      </div>
      
      <form onSubmit={handleWifiSubmit}>
        <div className="bg-white p-6 rounded-lg shadow-md">
          <h2 className="text-lg font-semibold mb-4">Manual WiFi Configuration</h2>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700">
                SSID
              </label>
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
              <label className="block text-sm font-medium text-gray-700">
                Password
              </label>
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
          <div className="flex justify-end mt-4">
            <button
              type="submit"
              className="bg-blue-500 text-white px-4 py-2 rounded-md hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
            >
              Save WiFi
            </button>
          </div>
        </div>
      </form>

      <form onSubmit={handleCellularSubmit}>
        <div className="bg-white p-6 rounded-lg shadow-md">
          <h2 className="text-lg font-semibold mb-4">APN Configuration</h2>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700">
                IMEI
              </label>
            </div>
            <div>{networkConfig.imei}</div>
            <div>
              <label className="block text-sm font-medium text-gray-700">
                Name
              </label>
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
          </div>
          <div className="flex justify-end mt-4">
            <button
              type="submit"
              className="bg-green-500 text-white px-4 py-2 rounded-md hover:bg-green-600 focus:outline-none focus:ring-2 focus:ring-green-500 focus:ring-offset-2"
            >
              Save APN
            </button>
          </div>
        </div>
      </form>
    </div>
  );
}
