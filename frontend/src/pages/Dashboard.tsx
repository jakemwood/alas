import React, { useEffect } from 'react';
import { useStore } from '../lib/store';
import { useApi } from '../lib/api';
import { Wifi, WifiOff, Signal, Volume2, Radio } from 'lucide-react';

export function Dashboard() {
  const { systemStatus, updateSystemStatus } = useStore();
  const api = useApi();

  useEffect(() => {
    const unsubscribe = api.subscribeToVolumeUpdates((volume) => {
      updateSystemStatus({ audio: { ...systemStatus.audio, currentVolume: volume } });
    });

    return () => unsubscribe();
  }, []);

  useEffect(() => {
    api.getNetworkConfig().then(response => {
      updateSystemStatus({
        network: {
          wifiConnected: response.wifi_connected,
          cellularConnected: response.cell_connected,
          signalStrength: 100,
        }
      })
    });
  }, []);

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
      <div className="bg-white p-6 rounded-lg shadow-md">
        <h2 className="text-lg font-semibold mb-4 flex items-center">
          <Wifi className="mr-2" /> Network Status
        </h2>
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <span>WiFi</span>
            {systemStatus.network.wifiConnected ? (
              <Wifi className="text-green-500" />
            ) : (
              <WifiOff className="text-red-500" />
            )}
          </div>
          <div className="flex items-center justify-between">
            <span>Cellular</span>
            <div className="flex items-center">
              <Signal className={systemStatus.network.cellularConnected ? 'text-green-500' : 'text-red-500'} />
              <span className="ml-2">{systemStatus.network.signalStrength}%</span>
            </div>
          </div>
        </div>
      </div>

      <div className="bg-white p-6 rounded-lg shadow-md">
        <h2 className="text-lg font-semibold mb-4 flex items-center">
          <Volume2 className="mr-2" /> Audio Status
        </h2>
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <span>Current Volume</span>
            <div className="w-32 bg-gray-200 rounded-full h-2">
              <div
                className="bg-blue-500 rounded-full h-2"
                style={{
                  width: `${((systemStatus.audio.currentVolume + 60) / 60) * 100}%`,
                }}
              />
            </div>
          </div>
          <div className="flex items-center justify-between">
            <span>Status</span>
            <span className={systemStatus.audio.isActive ? 'text-green-500' : 'text-red-500'}>
              {systemStatus.audio.isActive ? 'Active' : 'Inactive'}
            </span>
          </div>
        </div>
      </div>

      <div className="bg-white p-6 rounded-lg shadow-md">
        <h2 className="text-lg font-semibold mb-4 flex items-center">
          <Radio className="mr-2" /> Icecast Status
        </h2>
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <span>Connection</span>
            <span className={systemStatus.icecast.connected ? 'text-green-500' : 'text-red-500'}>
              {systemStatus.icecast.connected ? 'Connected' : 'Disconnected'}
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}