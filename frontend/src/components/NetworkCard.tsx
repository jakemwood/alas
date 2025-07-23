import React from "react";
import {
  Lock,
  ChevronDown,
  ChevronUp,
  Loader2,
  SignalHigh,
  SignalMedium,
  SignalLow,
} from "lucide-react";
import { AvailableNetwork } from "../types";

interface NetworkCardProps {
  network: AvailableNetwork;
  isSelected: boolean;
  isCurrentNetwork: boolean;
  password: string;
  isConnecting: boolean;
  onSelect: (network: AvailableNetwork) => void;
  onPasswordChange: (password: string) => void;
  onConnect: () => void;
}

function Signal({ value: strength }: { value: number }) {
  if (strength >= 60) {
    return <SignalHigh className={`w-5 h-5 text-green-500`} />;
  } else if (strength >= 40) {
    return <SignalMedium className={`w-5 h-5 text-yellow-500`} />;
  }
  return <SignalLow className={`w-5 h-5 text-red-500`} />;
}

export function NetworkCard({
  network,
  isSelected,
  isCurrentNetwork,
  password,
  isConnecting,
  onSelect,
  onPasswordChange,
  onConnect,
}: NetworkCardProps) {
  const getStrengthLabel = (strength: number) => {
    if (strength >= 80) return "Excellent";
    if (strength >= 60) return "Good";
    if (strength >= 40) return "Fair";
    return "Weak";
  };

  const getFrequencyBand = (frequency: number) => {
    return frequency > 5000 ? "5 GHz" : "2.4 GHz";
  };

  return (
    <div className="rounded-lg border-2 overflow-hidden transition-all">
      <div
        className={`p-4 cursor-pointer ${
          isSelected
            ? "border-indigo-600 bg-indigo-50"
            : isCurrentNetwork
            ? "border-green-600 bg-green-50"
            : "border-gray-200 hover:border-indigo-300"
        }`}
        onClick={() => onSelect(network)}
      >
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <Signal value={network.strength} />
            <div>
              <div className="flex items-center space-x-2">
                <h3 className="font-medium">{network.ssid}</h3>
                {isCurrentNetwork && (
                  <span className="text-xs bg-green-100 text-green-800 px-2 py-1 rounded-full">
                    Connected
                  </span>
                )}
              </div>
              <p className="text-sm text-gray-500">
                {getStrengthLabel(network.strength)} ·{" "}
                {getFrequencyBand(network.frequency)}
              </p>
            </div>
          </div>
          <div className="flex items-center space-x-2">
            {network.security && <Lock className="w-4 h-4 text-gray-400" />}
            {isSelected ? (
              <ChevronUp className="w-4 h-4 text-gray-400" />
            ) : (
              <ChevronDown className="w-4 h-4 text-gray-400" />
            )}
          </div>
        </div>
      </div>

      {isSelected && (
        <div className="p-4 bg-white border-t border-gray-100">
          <div className="space-y-4">
            {network.security && !isCurrentNetwork && (
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Password
                </label>
                <input
                  type="password"
                  value={password}
                  onChange={(e) => onPasswordChange(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500"
                  placeholder="Enter network password"
                />
              </div>
            )}
            {!isCurrentNetwork && (
              <button
                onClick={onConnect}
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
            )}
          </div>
        </div>
      )}
    </div>
  );
}