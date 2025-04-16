import React, { useEffect, useState } from "react";
import { useApi } from "../lib/api";
import { Link2, Link2Off } from 'lucide-react';

const DROPBOX_CLIENT_ID = "bt0bmbyf7usblq4"

export function Settings() {
  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [error, setError] = useState("");
  const [success, setSuccess] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [dropboxUrl, setDropboxUrl] = useState("");
  const [isDropboxLinked, setIsDropboxLinked] = useState(false);
  const [isDropboxLoading, setIsDropboxLoading] = useState(false);
  const api = useApi();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setSuccess("");

    if (newPassword !== confirmPassword) {
      setError("New passwords do not match");
      return;
    }

    setIsLoading(true);

    try {
      await api.updatePassword(currentPassword, newPassword);
      setSuccess("Password updated successfully");
      setError("");
      setCurrentPassword("");
      setNewPassword("");
      setConfirmPassword("");
    } catch (err) {
      setError("Failed to update password");
    } finally {
      setIsLoading(false);
    }
  };

  const handleDropboxLink = async () => {
    setIsDropboxLoading(true);

    try {
      if (isDropboxLinked) {
        // await api.unlinkDropbox();
        setIsDropboxLinked(false);
      } else {
        window.location.href = dropboxUrl;
        // window.location.href = dropboxUrl;
        // await api.linkDropbox();
        setIsDropboxLinked(true);
      }
    } catch (err) {
      console.error('Failed to handle Dropbox link:', err);
    } finally {
      setIsDropboxLoading(false);
    }
  };

  useEffect(() => {
    api.getDropboxLink().then(response => {
      setDropboxUrl(response);
    });
  }, []);

  return (
    <div className="max-w-2xl mx-auto space-y-6">
    <form onSubmit={handleSubmit} className="bg-white p-6 rounded-lg shadow-md space-y-6">
      <h2 className="text-lg font-semibold mb-4">Change Password</h2>

      {error && (
        <div className="text-red-500 text-sm">{error}</div>
      )}

      {success && (
        <div className="text-green-500 text-sm">{success}</div>
      )}

      <div>
        <label className="block text-sm font-medium text-gray-700">
          Current Password
        </label>
        <input
          type="password"
          required
          value={currentPassword}
          onChange={(e) => setCurrentPassword(e.target.value)}
          className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
        />
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-700">
          New Password
        </label>
        <input
          type="password"
          required
          value={newPassword}
          onChange={(e) => setNewPassword(e.target.value)}
          className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
        />
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-700">
          Confirm New Password
        </label>
        <input
          type="password"
          required
          value={confirmPassword}
          onChange={(e) => setConfirmPassword(e.target.value)}
          className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
        />
      </div>

      <div className="flex justify-end">
        <button
          type="submit"
          disabled={isLoading}
          className="bg-blue-500 text-white px-4 py-2 rounded-md hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50"
        >
          {isLoading ? 'Updating...' : 'Update Password'}
        </button>
      </div>
    </form>

    <div className="bg-white p-6 rounded-lg shadow-md">
      <h2 className="text-lg font-semibold mb-4">Dropbox Integration</h2>
      <p className="text-gray-600 mb-4">
        Link your Dropbox account to automatically backup your configuration and logs.
      </p>
      
      <div className="flex items-center justify-between">
        <div className="flex items-center">
          {isDropboxLinked ? (
            <Link2 className="h-5 w-5 text-green-500 mr-2" />
          ) : (
            <Link2Off className="h-5 w-5 text-gray-400 mr-2" />
          )}
          <span className="text-sm font-medium">
            {isDropboxLinked ? 'Connected to Dropbox' : 'Not connected'}
          </span>
        </div>
        
        <button
          onClick={handleDropboxLink}
          disabled={isDropboxLoading}
          className={`px-4 py-2 rounded-md text-sm font-medium focus:outline-none focus:ring-2 focus:ring-offset-2 ${
            isDropboxLinked
              ? 'bg-red-100 text-red-700 hover:bg-red-200 focus:ring-red-500'
              : 'bg-blue-500 text-white hover:bg-blue-600 focus:ring-blue-500'
          } disabled:opacity-50`}
        >
          {isDropboxLoading
            ? 'Processing...'
            : isDropboxLinked
            ? 'Disconnect'
            : 'Connect to Dropbox'}
        </button>
      </div>
    </div>
  </div>
  );
}
