import React from "react";
import { AlertTriangle } from "lucide-react";

interface ConfirmModalProps {
    isOpen: boolean;
    onConfirm: () => void;
    onCancel: () => void;
}

export function ConfirmModal({ isOpen, onConfirm, onCancel }: ConfirmModalProps) {
    if (!isOpen) return null;

    return (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
            <div className="bg-white rounded-lg max-w-md w-full p-6 shadow-xl">
                <div className="flex items-center space-x-3 text-amber-500 mb-4">
                    <AlertTriangle className="w-6 h-6" />
                    <h3 className="text-lg font-semibold">Confirm Connection</h3>
                </div>
                <p className="text-gray-600 mb-6">
                    You are about to pair your device with a Wi-Fi connection. You will lose access
                    to this page and will need to sign in to the unit from the same network at its
                    new IP address. Continue?
                </p>
                <div className="flex space-x-3 justify-end">
                    <button
                        onClick={onCancel}
                        className="px-4 py-2 text-gray-600 hover:bg-gray-100 rounded-md transition-colors"
                    >
                        Cancel
                    </button>
                    <button
                        onClick={onConfirm}
                        className="px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700 transition-colors"
                    >
                        Connect
                    </button>
                </div>
            </div>
        </div>
    );
}
