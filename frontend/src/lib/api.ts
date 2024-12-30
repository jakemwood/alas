import { useAuthStore } from "./auth-store";

export function useApi() {
  const authStore = useAuthStore();
  const API_BASE = `http://${authStore.ipAddress}`;
  return {
    async getNetworkConfig() {
      const res = await fetch(`${API_BASE}/network`);
      return res.json();
    },

    async updateNetworkConfig(config: any) {
      const res = await fetch(`${API_BASE}/network`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config),
      });
      return res.json();
    },

    async getAudioConfig() {
      const res = await fetch(`${API_BASE}/audio`);
      return res.json();
    },

    async updateAudioConfig(config: any) {
      const res = await fetch(`${API_BASE}/audio`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config),
      });
      return res.json();
    },

    async getIcecastConfig() {
      const res = await fetch(`${API_BASE}/icecast`);
      return res.json();
    },

    async updateIcecastConfig(config: any) {
      const res = await fetch(`${API_BASE}/icecast`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config),
      });
      return res.json();
    },

    subscribeToVolumeUpdates(callback: (volume: number) => void) {
      const eventSource = new EventSource(`${API_BASE}/audio/volume`);
      eventSource.onmessage = (event) => {
        callback(event.data);
      };
      return () => eventSource.close();
    },
  };
}