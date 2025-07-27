import { AuthCredentials } from "../types/auth";

export const authApi = {
  async login(credentials: AuthCredentials) {
    const response = await fetch(`https://${credentials.ipAddress}/auth/login`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ password: credentials.password }),
    });

    if (!response.ok) {
      throw new Error("Invalid credentials");
    }

    return response.json();
  },
};
