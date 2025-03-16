import React from "react";
import { Link, Outlet, useLocation, useNavigate } from "react-router-dom";
import { Settings, Wifi, Volume2, Radio, LogOut, Key, Share2 } from "lucide-react";
import { useAuthStore } from "../lib/auth-store";

const navItems = [
  { path: "/", label: "Dashboard", icon: Settings },
  { path: "/network", label: "Network", icon: Wifi },
  { path: "/audio", label: "Audio", icon: Volume2 },
  { path: "/icecast", label: "Icecast", icon: Radio },
  { path: "/redundancy", label: "Redundancy", icon: Share2 },
  { path: "/settings", label: "Settings", icon: Key },
];

export function Layout() {
  const location = useLocation();
  const navigate = useNavigate();
  const logout = useAuthStore((state) => state.logout);

  const handleLogout = () => {
    logout();
    navigate("/login");
  };

  return (
    <div className="min-h-screen bg-gray-100">
      <nav className="bg-white shadow-md">
        <div className="max-w-7xl mx-auto px-4">
          <div className="flex justify-between h-16">
            <div className="flex">
              {navItems.map(({ path, label, icon: Icon }) => (
                <Link
                  key={path}
                  to={path}
                  className={`inline-flex items-center px-4 py-2 border-b-2 text-sm font-medium ${
                    location.pathname === path
                      ? "border-blue-500 text-blue-600"
                      : "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                  }`}
                >
                  <Icon className="h-5 w-5 mr-2" />
                  {label}
                </Link>
              ))}
            </div>
            <button
              onClick={handleLogout}
              className="inline-flex items-center px-4 py-2 text-sm font-medium text-gray-500 hover:text-gray-700"
            >
              <LogOut className="h-5 w-5 mr-2" />
              Logout
            </button>
          </div>
        </div>
      </nav>

      <main className="max-w-7xl mx-auto py-6 px-4 sm:px-6 lg:px-8">
        <Outlet />
      </main>
    </div>
  );
}
