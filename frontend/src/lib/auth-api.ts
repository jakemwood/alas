import { AuthCredentials } from '../types/auth';

export const authApi = {
  async login(credentials: AuthCredentials) {
    const response = await fetch(`http://${credentials.ipAddress}/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({"password": credentials.password}),
    });
    
    if (!response.ok) {
      throw new Error('Invalid credentials');
    }
    
    return response.json();
  },

  async updatePassword(currentPassword: string, newPassword: string) {
    throw new Error("Not implemented");
    // const response = await fetch(`http://${credentials.ipAddress}/auth/password`, {
    //   method: 'PUT',
    //   headers: { 'Content-Type': 'application/json' },
    //   body: JSON.stringify({ currentPassword, newPassword }),
    // });

    // if (!response.ok) {
    //   throw new Error('Failed to update password');
    // }

    // return response.json();
  },
};