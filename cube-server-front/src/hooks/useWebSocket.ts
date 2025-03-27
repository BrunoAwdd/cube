import { useEffect } from "react";

export function useWebSocket(onTokenReceived: (token: string) => void) {
  useEffect(() => {
    const ws = new WebSocket("ws://bruno-linux:8080/ws");

    ws.onopen = () => {
      console.log("🟢 WebSocket conectado");
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        if (data.token) {
          console.log("📥 Token recebido via WS:", data.token);
          onTokenReceived(data.token);
        }
      } catch (err) {
        console.error("❌ Erro ao processar WS:", err);
      }
    };

    ws.onerror = (err) => {
      console.error("🔴 WebSocket erro:", err);
    };

    ws.onclose = () => {
      console.warn("🟡 WebSocket desconectado");
    };

    return () => {
      ws.close();
    };
  }, [onTokenReceived]);
}
