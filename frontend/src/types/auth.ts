export interface AuthCredentials {
  ipAddress: string;
  password: string;
}

export interface AuthState {
  isAuthenticated: boolean;
  ipAddress: string;
}