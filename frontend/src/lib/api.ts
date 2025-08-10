import { useAuthStore } from "./auth-store";
import { useNavigate } from "react-router-dom";

export function useApi() {
  const authStore = useAuthStore();
  const navigate = useNavigate();
  const API_BASE = `https://${authStore.ipAddress}`;
  const headers = {
    Authorization: `Bearer ${authStore.jwt}`,
  };

  const handleAuthError = (response: Response) => {
    if (response.status === 401 || response.status === 403) {
      authStore.logout();
      navigate("/login");
      return true;
    }
    return false;
  };
  return {
    async getNetworkStatus() {
      const res = await fetch(`${API_BASE}/status/network`, { headers });
      if (handleAuthError(res)) return;
      return res.json();
    },

    async getCellularConfig() {
      const res = await fetch(`${API_BASE}/config/cellular`, { headers });
      if (handleAuthError(res)) return;
      return res.json();
    },

    async updateCellularConfig(apn: string) {
      const res = await fetch(`${API_BASE}/config/cellular`, {
        method: "POST",
        headers: { ...headers, "Content-Type": "application/json" },
        body: JSON.stringify({ apn }),
      });
      if (handleAuthError(res)) return;
      return res.status === 200;
    },

    async getAvailableWifi() {
      const res = await fetch(`${API_BASE}/config/wifi/available`, { headers });
      if (handleAuthError(res)) return;
      return res.json();
    },

    async updateWifiConfig(ssid: string, password: string) {
      const res = await fetch(`${API_BASE}/config/wifi/connect`, {
        method: "POST",
        headers: { ...headers, "Content-Type": "application/json" },
        body: JSON.stringify({ ap: ssid, password }),
      });
      if (handleAuthError(res)) return;
      return res.status === 201;
    },

    async connectToWifi(ap_path: string, password?: string) {
      const res = await fetch(`${API_BASE}/config/wifi/connect`, {
        method: "POST",
        headers: { ...headers, "Content-Type": "application/json" },
        body: JSON.stringify({ ap: ap_path, password }),
      });
      if (handleAuthError(res)) return;
      return res.status === 201;
    },

    async getAudioStatus() {
      const res = await fetch(`${API_BASE}/status/audio`, { headers });
      if (handleAuthError(res)) return;
      return res.json();
    },

    async getAudioConfig() {
      const res = await fetch(`${API_BASE}/config/audio`, { headers });
      if (handleAuthError(res)) return;
      return res.json();
    },

    async updateAudioConfig(config: any) {
      const res = await fetch(`${API_BASE}/config/audio`, {
        method: "POST",
        headers: { "Content-Type": "application/json", ...headers },
        body: JSON.stringify(config),
      });
      if (handleAuthError(res)) return;
      return res.json();
    },

    async getIcecastConfig() {
      const res = await fetch(`${API_BASE}/config/icecast`, { headers });
      if (handleAuthError(res)) return;
      return res.json();
    },

    async updateIcecastConfig(config: any) {
      const res = await fetch(`${API_BASE}/config/icecast`, {
        method: "POST",
        headers: { "Content-Type": "application/json", ...headers },
        body: JSON.stringify(config),
      });
      if (handleAuthError(res)) return;
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

      if (handleAuthError(response)) return;
      if (!response.ok) {
        throw new Error("Failed to update password");
      }
    },

    subscribeToVolumeUpdates(callback: (volume: number) => void) {
      const eventSource = new EventSource(`${API_BASE}/status/meter`);
      eventSource.onmessage = (event) => {
        callback(parseFloat(event.data));
      };
      eventSource.onerror = () => {
        // EventSource doesn't provide status codes, so we'll check if auth is still valid
        fetch(`${API_BASE}/status/meter`, { headers })
          .then(res => {
            if (handleAuthError(res)) {
              eventSource.close();
            }
          })
          .catch(() => {
            // Network error, close the connection
            eventSource.close();
          });
      };
      return () => eventSource.close();
    },

    async getRedundancyConfig() {
      const res = await fetch(`${API_BASE}/config/redundancy`, { headers });
      if (handleAuthError(res)) return;
      return res.json();
    },

    async updateRedundancyConfig(config: any) {
      const res = await fetch(`${API_BASE}/config/redundancy`, {
        method: "POST",
        headers: { "Content-Type": "application/json", ...headers },
        body: JSON.stringify(config),
      });
      if (handleAuthError(res)) return;
      return res.json();
    },

    async getDropboxLink() {
      const res = await fetch(`${API_BASE}/config/dropbox-link`, {
        method: "GET",
        headers: { "Content-Type": "application/json", ...headers },
      });
      if (handleAuthError(res)) return;
      const reply = await res.json();
      return reply.url;
    },

    async linkDropbox(code: string) {
      const res = await fetch(`${API_BASE}/config/dropbox-link`, {
        method: "POST",
        headers: { "Content-Type": "application/json", ...headers },
        body: JSON.stringify({ code }),
      });
      if (handleAuthError(res)) return;
      return res.status == 201;
    },

    async getDropboxStatus() {
      const res = await fetch(`${API_BASE}/config/dropbox-status`, { headers });
      if (handleAuthError(res)) return;
      return res.json();
    },

    async getWebhookConfig() {
      const res = await fetch(`${API_BASE}/config/webhook`, { headers });
      if (handleAuthError(res)) return;
      return res.json();
    },

    async updateWebhookConfig(config: any) {
      const res = await fetch(`${API_BASE}/config/webhook`, {
        method: "POST",
        headers: { "Content-Type": "application/json", ...headers },
        body: JSON.stringify(config),
      });
      if (handleAuthError(res)) return;
      return res.json();
    },
  };
}
