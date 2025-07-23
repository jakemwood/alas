export interface Network {
    ssid: string;
    strength: number;
    ap_path: string;
    frequency: number;
    security: string;
}

export interface DropboxStatus {
    is_connected: boolean;
}

export interface DropboxUrl {
    url: string;
}
