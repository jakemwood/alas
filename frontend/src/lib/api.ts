import { useAuthStore } from "./auth-store";

export function useApi() {
  const authStore = useAuthStore();
  const API_BASE = `http://${authStore.ipAddress}`;
  const headers = {
    Authorization: `Bearer ${authStore.jwt}`,
  };
  return {
    async getNetworkStatus() {
      const res = await fetch(`${API_BASE}/status/network`, { headers });
      return res.json();
    },

    async getCellularConfig() {
      const res = await fetch(`${API_BASE}/config/cellular`, { headers });
      return res.json();
    },

    async updateCellularConfig(apn: string) {
      const res = await fetch(`${API_BASE}/config/cellular`, {
        method: "POST",
        headers: { ...headers, "Content-Type": "application/json" },
        body: JSON.stringify({ apn }),
      });
      return res.status === 200;
    },

    async getAvailableWifi() {
      const res = await fetch(`${API_BASE}/config/wifi/available`, { headers });
      return res.json();
    },

    async updateWifiConfig(ssid: string, password: string) {
      const res = await fetch(`${API_BASE}/config/wifi/connect`, {
        method: "POST",
        headers: { ...headers, "Content-Type": "application/json" },
        body: JSON.stringify({ ap: ssid, password }),
      });
      return res.status === 201;
    },

    async connectToWifi(ap_path: string, password?: string) {
      const res = await fetch(`${API_BASE}/config/wifi/connect`, {
        method: "POST",
        headers: { ...headers, "Content-Type": "application/json" },
        body: JSON.stringify({ ap: ap_path, password }),
      });
      return res.status === 201;
    },

    async getAudioStatus() {
      const res = await fetch(`${API_BASE}/status/audio`, { headers });
      return res.json();
    },

    async getAudioConfig() {
      const res = await fetch(`${API_BASE}/config/audio`, { headers });
      return res.json();
    },

    async updateAudioConfig(config: any) {
      const res = await fetch(`${API_BASE}/config/audio`, {
        method: "POST",
        headers: { "Content-Type": "application/json", ...headers },
        body: JSON.stringify(config),
      });
      return res.json();
    },

    async getIcecastConfig() {
      const res = await fetch(`${API_BASE}/config/icecast`, { headers });
      return res.json();
    },

    async updateIcecastConfig(config: any) {
      const res = await fetch(`${API_BASE}/config/icecast`, {
        method: "POST",
        headers: { "Content-Type": "application/json", ...headers },
        body: JSON.stringify(config),
      });
      return res.json();
    },

    async updatePassword(currentPassword: string, newPassword: string) {
      const response = await fetch(`${API_BASE}/auth/change-password`, {
        method: "POST",
        headers: { "Content-Type": "application/json", ...headers },
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
      const res = await fetch(`${API_BASE}/config/redundancy`, { headers });
      return res.json();
    },

    async updateRedundancyConfig(config: any) {
      const res = await fetch(`${API_BASE}/config/redundancy`, {
        method: "POST",
        headers: { "Content-Type": "application/json", ...headers },
        body: JSON.stringify(config),
      });
      return res.json();
    },

    async getDropboxLink() {
      const res = await fetch(`${API_BASE}/config/dropbox-link`, {
        method: "GET",
        headers: { "Content-Type": "application/json", ...headers },
      });
      const reply = await res.json();
      return reply.url;
    },

    async linkDropbox(code: string) {
      const res = await fetch(`${API_BASE}/config/dropbox-link`, {
        method: "POST",
        headers: { "Content-Type": "application/json", ...headers },
        body: JSON.stringify({ code }),
      });
      return res.status == 201;
    },

    async getDropboxStatus() {
      const res = await fetch(`${API_BASE}/config/dropbox-status`, { headers });
      return res.json();
    },
  };
}
