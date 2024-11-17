import React, { useEffect, useState } from 'react';
import { Wifi, WifiOff, Lock, SignalHigh, SignalMedium, SignalLow, Loader2, ChevronDown, ChevronUp } from 'lucide-react';

// Mock API response - in production, this would come from your backend
// const mockNetworks = {
//   networks: [
//     {
//       ssid: "Magikarp",
//       strength: 79,
//       ap_path: "/org/freedesktop/NetworkManager/AccessPoint/567",
//       frequency: 5785,
//       security: "wpa2"
//     }
//   ]
// };

interface Network {
  ssid: string;
  strength: number;
  ap_path: string;
  frequency: number;
  security: string;
}

function useNetworks() {
  const [isLoading, setIsLoading] = useState(true);
  const [networks, setNetworks] = useState<Network[]>([]);

  const loadWiFi = async () => {
    fetch("/available-wifi").then(async results => {
      const json = await results.json();
      setNetworks(json.networks);
      setIsLoading(false);
      setTimeout(loadWiFi, 5000);
    });
  }

  useEffect(() => {
    loadWiFi();
  }, []);

  return { isLoading, networks };
}

function Signal({ value: strength }: { value: number }) {
  if (strength >= 60) {
    return <SignalHigh className={`w-5 h-5 text-green-500`} />
  }
  else if (strength >= 40) {
    return <SignalMedium className={`w-5 h-5 text-yellow-50`} />
  }
  return <SignalLow className={`w-5 h-5 text-red-500`} />
}

function App() {
  const [selectedNetwork, setSelectedNetwork] = useState<Network | null>(null);
  const [password, setPassword] = useState('');
  const [isConnecting, setIsConnecting] = useState(false);
  const { isLoading, networks } = useNetworks();

  const handleConnect = async (network: Network) => {
    setIsConnecting(true);
    // Simulate connection attempt
    await new Promise(resolve => setTimeout(resolve, 2000));
    setIsConnecting(false);
    // In production, you would handle the actual connection here
  };

  const getStrengthLabel = (strength: number) => {
    if (strength >= 80) return 'Excellent';
    if (strength >= 60) return 'Good';
    if (strength >= 40) return 'Fair';
    return 'Weak';
  };

  const getFrequencyBand = (frequency: number) => {
    return frequency > 5000 ? '5 GHz' : '2.4 GHz';
  };

  return (
      <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-50 p-4 sm:p-6 md:p-8">
        <div className="max-w-md mx-auto bg-white rounded-2xl shadow-xl overflow-hidden">
          <div className="bg-indigo-600 p-6 text-white">
            <div className="flex items-center justify-between">
              <h1 className="text-2xl font-semibold">WiFi Networks</h1>
              <Wifi className="w-6 h-6" />
            </div>
            <p className="mt-2 text-indigo-100">Select a network to connect</p>
          </div>

          <div className="p-6">
            {isLoading ? (
                <div className="flex items-center justify-center py-8">
                  <Loader2 className="w-8 h-8 animate-spin text-indigo-600" />
                </div>
            ) : networks.length === 0 ? (
                <div className="text-center py-8">
                  <WifiOff className="w-12 h-12 mx-auto text-gray-400" />
                  <p className="mt-4 text-gray-500">No networks found</p>
                </div>
            ) : (
                <div className="space-y-4">
                  {networks.map((network) => (
                      <div key={network.ap_path} className="rounded-lg border-2 overflow-hidden transition-all">
                        <div
                            className={`p-4 cursor-pointer ${
                                selectedNetwork?.ap_path === network.ap_path
                                    ? 'border-indigo-600 bg-indigo-50'
                                    : 'border-gray-200 hover:border-indigo-300'
                            }`}
                            onClick={() => setSelectedNetwork(selectedNetwork?.ap_path === network.ap_path ? null : network)}
                        >
                          <div className="flex items-center justify-between">
                            <div className="flex items-center space-x-3">
                              <Signal value={network.strength} />
                              <div>
                                <h3 className="font-medium">{network.ssid}</h3>
                                <p className="text-sm text-gray-500">
                                  {getStrengthLabel(network.strength)} · {getFrequencyBand(network.frequency)}
                                </p>
                              </div>
                            </div>
                            <div className="flex items-center space-x-2">
                              {network.security && (
                                  <Lock className="w-4 h-4 text-gray-400" />
                              )}
                              {selectedNetwork?.ap_path === network.ap_path ? (
                                  <ChevronUp className="w-4 h-4 text-gray-400" />
                              ) : (
                                  <ChevronDown className="w-4 h-4 text-gray-400" />
                              )}
                            </div>
                          </div>
                        </div>

                        {selectedNetwork?.ap_path === network.ap_path && (
                            <div className="p-4 bg-white border-t border-gray-100">
                              <div className="space-y-4">
                                {network.security && (
                                    <div>
                                      <label className="block text-sm font-medium text-gray-700 mb-1">
                                        Password
                                      </label>
                                      <input
                                          type="password"
                                          value={password}
                                          onChange={(e) => setPassword(e.target.value)}
                                          className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500"
                                          placeholder="Enter network password"
                                      />
                                    </div>
                                )}
                                <button
                                    onClick={() => handleConnect(network)}
                                    disabled={isConnecting || (network.security && !password)}
                                    className="w-full bg-indigo-600 text-white py-2 px-4 rounded-md hover:bg-indigo-700
                                   disabled:bg-indigo-300 disabled:cursor-not-allowed transition-colors
                                   flex items-center justify-center space-x-2"
                                >
                                  {isConnecting ? (
                                      <>
                                        <Loader2 className="w-4 h-4 animate-spin" />
                                        <span>Connecting...</span>
                                      </>
                                  ) : (
                                      <span>Connect</span>
                                  )}
                                </button>
                              </div>
                            </div>
                        )}
                      </div>
                  ))}
                </div>
            )}
          </div>
        </div>
      </div>
  );
}

export default App;