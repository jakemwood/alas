import React from "react";
import { Loader2 } from "lucide-react";

interface LoadingScreenProps {
    ssid: string;
}

export function LoadingScreen({ ssid }: LoadingScreenProps) {
    return (
        <div className="fixed inset-0 bg-white z-50 flex flex-col items-center justify-center p-4">
            <Loader2 className="w-12 h-12 text-indigo-600 animate-spin mb-4" />
            <h2 className="text-xl font-semibold text-gray-900 mb-2">Connecting to {ssid}</h2>
            <p className="text-gray-500 text-center max-w-sm">
                Please wait while we establish the connection. This may take a few moments...
            </p>
        </div>
    );
}
