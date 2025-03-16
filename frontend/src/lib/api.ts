import { useAuthStore } from "./auth-store";

export function useApi() {
  const authStore = useAuthStore();
  const API_BASE = `http://${authStore.ipAddress}`;
  return {
    async getNetworkConfig() {
      const res = await fetch(`${API_BASE}/config/network`);
      return res.json();
    },

    async getNetworkStatus() {
      const res = await fetch(`${API_BASE}/status/network`);
      return res.json();
    },

    async updateNetworkConfig(config: any) {
      const res = await fetch(`${API_BASE}/config/network`, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(config),
      });
      return res.json();
    },

    async getAudioStatus() {
      const res = await fetch(`${API_BASE}/status/audio`);
      return res.json();
    },

    async getAudioConfig() {
      const res = await fetch(`${API_BASE}/config/audio`);
      return res.json();
    },

    async updateAudioConfig(config: any) {
      const res = await fetch(`${API_BASE}/config/audio`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(config),
      });
      return res.json();
    },

    async getIcecastConfig() {
      const res = await fetch(`${API_BASE}/config/icecast`);
      return res.json();
    },

    async updateIcecastConfig(config: any) {
      const res = await fetch(`${API_BASE}/config/icecast`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(config),
      });
      return res.json();
    },

    async updatePassword(currentPassword: string, newPassword: string) {
      const response = await fetch(`${API_BASE}/auth/change-password`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          old_password: currentPassword,
          new_password: newPassword,
        }),
      });

      if (!response.ok) {
        throw new Error("Failed to update password");
      }
    },

    subscribeToVolumeUpdates(callback: (volume: number) => void) {
      const eventSource = new EventSource(`${API_BASE}/status/meter`);
      eventSource.onmessage = (event) => {
        callback(parseFloat(event.data));
      };
      return () => eventSource.close();
    },

    async getRedundancyConfig() {
      const res = await fetch(`${API_BASE}/config/redundancy`);
      return res.json();
    },

    async updateRedundancyConfig(config: any) {
      const res = await fetch(`${API_BASE}/config/redundancy`, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(config),
      });
      return res.json();
    },
  };
}
