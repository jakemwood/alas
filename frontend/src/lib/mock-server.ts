import { createServer, Model, Response } from "miragejs";

export function makeServer({ environment = "development" } = {}) {
  return createServer({
    environment,

    models: {
      network: Model,
      audio: Model,
      icecast: Model,
    },

    seeds(server) {
      server.db.loadData({
        networks: [
          {
            wifi: { ssid: "MyNetwork", password: "password123" },
            apn: { name: "internet", username: "user", password: "pass" },
          },
        ],
        audios: [
          {
            silenceDuration: 30,
            silenceThreshold: -50,
            audioThreshold: -40,
          },
        ],
        icecasts: [
          {
            host: "icecast.example.com",
            port: 8000,
            mountPoint: "/stream",
          },
        ],
      });
    },

    routes() {
      this.urlPrefix = "http://ridgeline-live.local:8000";

      // Auth endpoints
      this.post("/auth/login", (schema, request) => {
        const attrs = JSON.parse(request.requestBody);
        // Mock authentication - in development, accept any credentials
        if (attrs.password) {
          return { success: true };
        }
        return new Response(401, {}, { error: "Invalid credentials" });
      });

      this.put("/auth/password", (schema, request) => {
        const attrs = JSON.parse(request.requestBody);
        // Mock password update - always succeed in development
        if (attrs.currentPassword && attrs.newPassword) {
          return { success: true };
        }
        return new Response(400, {}, { error: "Invalid password" });
      });

      // Existing routes...
      this.get("/network", (schema) => schema.db.networks[0]);
      this.put("/network", (schema, request) => {
        const attrs = JSON.parse(request.requestBody);
        schema.db.networks.update(1, attrs);
        return schema.db.networks[0];
      });

      this.get("/audio", (schema) => schema.db.audios[0]);
      this.put("/audio", (schema, request) => {
        const attrs = JSON.parse(request.requestBody);
        schema.db.audios.update(1, attrs);
        return schema.db.audios[0];
      });

      this.get("/icecast", (schema) => schema.db.icecasts[0]);
      this.put("/icecast", (schema, request) => {
        const attrs = JSON.parse(request.requestBody);
        schema.db.icecasts.update(1, attrs);
        return schema.db.icecasts[0];
      });

      this.get("/audio/volume", () => {
        return new Response(
          200,
          {
            "Content-Type": "text/event-stream",
            "Cache-Control": "no-cache",
            Connection: "keep-alive",
          },
          () => {
            let counter = 0;
            const stream = new ReadableStream({
              start(controller) {
                setInterval(() => {
                  const volume = -60 + Math.random() * 30;
                  controller.enqueue(`data: ${JSON.stringify({ volume })}\n\n`);
                  counter++;
                }, 1000);
              },
            });
            return stream;
          }
        );
      });
    },
  });
}
