import { useEffect, useRef, useState } from "react";

interface UseWebSocketOptions {
  onTokenReceived: (token: string) => void;
  onStatusChange?: (connected: boolean) => void;
}

export function useWebSocket({
  onTokenReceived,
  onStatusChange,
}: UseWebSocketOptions) {
  const wsRef = useRef<WebSocket | null>(null);
  const [connected, setConnected] = useState(false);
  const reconnectDelay = useRef(1000); // ms

  const connect = () => {
    if (wsRef.current?.readyState === WebSocket.OPEN) return;

    const ws = new WebSocket("ws://bruno-linux:8080/ws");
    wsRef.current = ws;

    ws.onopen = () => {
      console.log("🟢 WebSocket conectado");
      setConnected(true);
      reconnectDelay.current = 1000; // reset delay
      onStatusChange?.(true);
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
      setConnected(false);
      onStatusChange?.(false);

      setTimeout(() => {
        reconnectDelay.current = Math.min(reconnectDelay.current * 2, 15000);
        connect();
      }, reconnectDelay.current);
    };
  };

  useEffect(() => {
    connect();
    return () => {
      wsRef.current?.close();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return {
    send: (data: any) => {
      if (connected && wsRef.current?.readyState === WebSocket.OPEN) {
        wsRef.current.send(JSON.stringify(data));
      } else {
        console.warn("🔌 WS não está conectado.");
      }
    },
    connected,
  };
}
