import { AuthCredentials } from "../types/auth";

export const authApi = {
  async login(credentials: AuthCredentials) {
    let url = `//${credentials.ipAddress}/auth/login`;
    if (credentials.ipAddress.startsWith("http")) {
      url = `${credentials.ipAddress}/auth/login`
    }
    const response = await fetch(url, {
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
