import { createServer } from "miragejs";

export function fakeApiServer() {
    return createServer({
        routes() {
            // this.namespace = "api"

            this.get("/wifi/available", () => {
                return {
                    networks: [
                        {
                            ssid: "Magikarp",
                            strength: 79,
                            ap_path: "/org/freedesktop/NetworkManager/AccessPoint/567",
                            frequency: 5785,
                            security: "wpa2",
                        },
                    ],
                };
            });

            this.post(
                "/wifi/connect",
                () => {
                    return {};
                },
                { timing: Number.MAX_VALUE },
            );
        },
    });
}
