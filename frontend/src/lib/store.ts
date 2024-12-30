import { create } from 'zustand';
import type { NetworkConfig, AudioConfig, IcecastConfig, SystemStatus } from '../types';

interface AppState {
  networkConfig: NetworkConfig;
  audioConfig: AudioConfig;
  icecastConfig: IcecastConfig;
  systemStatus: SystemStatus;
  setNetworkConfig: (config: NetworkConfig) => void;
  setAudioConfig: (config: AudioConfig) => void;
  setIcecastConfig: (config: IcecastConfig) => void;
  updateSystemStatus: (status: Partial<SystemStatus>) => void;
}

export const useStore = create<AppState>((set) => ({
  networkConfig: {
    wifi: { ssid: '', password: '' },
    apn: { name: '', username: '', password: '' },
  },
  audioConfig: {
    silenceDuration: 30,
    silenceThreshold: -50,
    audioThreshold: -40,
  },
  icecastConfig: {
    host: '',
    port: 8000,
    mountPoint: '',
  },
  systemStatus: {
    network: {
      wifiConnected: false,
      cellularConnected: false,
      signalStrength: 0,
    },
    audio: {
      currentVolume: -60,
      isActive: false,
    },
    icecast: {
      connected: false,
      listeners: 0,
    },
  },
  setNetworkConfig: (config) => set({ networkConfig: config }),
  setAudioConfig: (config) => set({ audioConfig: config }),
  setIcecastConfig: (config) => set({ icecastConfig: config }),
  updateSystemStatus: (status) =>
    set((state) => ({
      systemStatus: { ...state.systemStatus, ...status },
    })),
}));