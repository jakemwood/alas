import React from "react";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { Layout } from "./components/Layout";
import { LoginForm } from "./components/LoginForm";
import { PrivateRoute } from "./components/PrivateRoute";
import { Dashboard } from "./pages/Dashboard";
import { Network } from "./pages/Network";
import { Audio } from "./pages/Audio";
import { Icecast } from "./pages/Icecast";
import { Redundancy } from "./pages/Redundancy";
import { Settings } from "./pages/Settings";
import { makeServer } from "./lib/mock-server";

// @ts-ignore
// if (process.env.NODE_ENV === 'development') {
//   makeServer();
// }

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/login" element={<LoginForm />} />
        <Route
          path="/"
          element={
            <PrivateRoute>
              <Layout />
            </PrivateRoute>
          }
        >
          <Route index element={<Dashboard />} />
          <Route path="network" element={<Network />} />
          <Route path="audio" element={<Audio />} />
          <Route path="icecast" element={<Icecast />} />
          <Route path="redundancy" element={<Redundancy />} />
          <Route path="settings" element={<Settings />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}
