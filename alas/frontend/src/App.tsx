import { useCallback, useEffect, useState, useRef } from "react";
import { Wifi, WifiOff, Loader2 } from "lucide-react";
import { Network } from "./types";
import { NetworkCard } from "./components/NetworkCard";
import { ConfirmModal } from "./components/ConfirmModal";
import { LoadingScreen } from "./components/LoadingScreen";
import { fakeApiServer } from "./server.ts";

if (process.env.NODE_ENV === "development") {
    fakeApiServer();
}

function useNetworks(isConnecting: boolean) {
    console.log("useNetworks: isConnecting is", isConnecting);
    const [isLoading, setIsLoading] = useState(true);
    const [networks, setNetworks] = useState<Network[]>([]);
    const timeoutRef = useRef<number>();

    const loadWiFi = useCallback(
        async (isConnecting: boolean) => {
            if (isConnecting) {
                return;
            }
            console.log("loadwiFi: isCOnnecting is", isConnecting);
            const results = await fetch("/config/wifi/available");
            const json = await results.json();
            setNetworks(json.networks);
            setIsLoading(false);
            timeoutRef.current = setTimeout(() => loadWiFi(isConnecting), 1000);
        },
        [setNetworks, setIsLoading, isConnecting],
    );

    useEffect(() => {
        console.log("use effect: isCOnnecting is", isConnecting);
        if (!isConnecting) {
            loadWiFi(isConnecting).then(() => {});
        }
    }, [loadWiFi, isConnecting]);

    useEffect(() => {
        if (!isConnecting && timeoutRef.current) {
            clearTimeout(timeoutRef.current);
        }
    }, [isConnecting]);

    return { isLoading, networks };
}

function App() {
    const [selectedNetwork, setSelectedNetwork] = useState<Network | null>(null);
    const [password, setPassword] = useState("");
    const [isConnecting, setIsConnecting] = useState(false);
    const { isLoading, networks } = useNetworks(isConnecting);

    const [showConfirmModal, setShowConfirmModal] = useState(false);
    const [showLoadingScreen, setShowLoadingScreen] = useState(false);

    const handleConnect = async () => {
        setShowConfirmModal(true);
    };

    const handleConfirmConnect = async () => {
        if (!selectedNetwork) {
            alert("You didn't select a network, silly goose");
            return;
        }

        setShowConfirmModal(false);
        setShowLoadingScreen(true);
        setIsConnecting(true);

        await fetch("/config/wifi/connect", {
            method: "POST",
            body: JSON.stringify({
                ap: selectedNetwork.ap_path,
                password: password,
            }),
            headers: {
                "Content-Type": "application/json",
            },
        });
    };

    return (
        <>
            <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-50 p-4 sm:p-6 md:p-8">
                <div className="max-w-md mx-auto bg-white rounded-2xl shadow-xl overflow-hidden">
                    <div className="bg-indigo-600 p-6 text-white">
                        <div className="flex items-center justify-between">
                            <h1 className="text-2xl font-semibold">WiFi Networks</h1>
                            <Wifi className="w-6 h-6" />
                        </div>
                        <p className="mt-2 text-indigo-100">Select a network to connect</p>
                    </div>

                    <div className="p-6">
                        {isLoading ? (
                            <div className="flex items-center justify-center py-8">
                                <Loader2 className="w-8 h-8 animate-spin text-indigo-600" />
                            </div>
                        ) : networks.length === 0 ? (
                            <div className="text-center py-8">
                                <WifiOff className="w-12 h-12 mx-auto text-gray-400" />
                                <p className="mt-4 text-gray-500">No networks found</p>
                            </div>
                        ) : (
                            <div className="space-y-4">
                                {networks.map((network) => (
                                    <NetworkCard
                                        key={network.ap_path}
                                        network={network}
                                        isSelected={selectedNetwork?.ap_path === network.ap_path}
                                        password={password}
                                        isConnecting={isConnecting}
                                        onSelect={(network) =>
                                            setSelectedNetwork(
                                                selectedNetwork?.ap_path === network.ap_path
                                                    ? null
                                                    : network,
                                            )
                                        }
                                        onPasswordChange={setPassword}
                                        onConnect={handleConnect}
                                    />
                                ))}
                            </div>
                        )}
                    </div>
                </div>
            </div>

            <ConfirmModal
                isOpen={showConfirmModal}
                onConfirm={handleConfirmConnect}
                onCancel={() => setShowConfirmModal(false)}
            />

            {showLoadingScreen && selectedNetwork && <LoadingScreen ssid={selectedNetwork.ssid} />}
        </>
    );
}

export default App;
