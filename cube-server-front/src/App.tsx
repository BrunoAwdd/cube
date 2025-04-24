import React, { useState } from "react";
import { FluentProvider, webLightTheme } from "@fluentui/react-components";
import { QrCodePanel } from "./qr/qrCodePanel";
import { PhotoGrid } from "./PhotoGrid";
import { useWebSocket } from "./hooks/useWebSocket";
import { ConnectionStatus } from "./components/ConnectionStatus";

const App = () => {
  const [token, setToken] = useState<string | null>(null);
  const [isOnline, setIsOnline] = useState(false);

  const { send } = useWebSocket({
    onTokenReceived: setToken,
    onStatusChange: setIsOnline,
  });

  return (
    <FluentProvider
      theme={webLightTheme}
      style={{ height: "100vh", padding: 24, position: "relative" }}
    >
      <ConnectionStatus online={isOnline} />
      {!token ? <QrCodePanel /> : <PhotoGrid send={send} />}
    </FluentProvider>
  );
};

export default App;
