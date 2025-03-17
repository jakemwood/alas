import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { AuthState } from "../types/auth";

interface AuthStore extends AuthState {
  login: (ipAddress: string, jwt: string) => void;
  logout: () => void;
}

export const useAuthStore = create<AuthStore>()(
  persist(
    (set) => ({
      isAuthenticated: false,
      ipAddress: "",
      jwt: "",
      login: (ipAddress, jwt) => set({ isAuthenticated: true, ipAddress, jwt }),
      logout: () => set({ isAuthenticated: false, ipAddress: "" }),
    }),
    {
      name: "auth-storage",
    }
  )
);
