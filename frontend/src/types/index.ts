export interface NetworkConfig {
  wifi: {
    ssid: string;
    password: string;
  };
  apn: {
    name: string;
    username: string;
    password: string;
  };
}

export interface AudioConfig {
  silenceDuration: string;
  silenceThreshold: string;
}

export interface IcecastConfig {
  host: string;
  port: number;
  mountPoint: string;
  password: string;
}

export interface SystemStatus {
  network: {
    wifiConnected: boolean;
    cellularConnected: boolean;
    signalStrength: number;
  };
  audio: {
    currentVolume: number;
    isActive: boolean;
  };
  icecast: {
    connected: boolean;
  };
}
