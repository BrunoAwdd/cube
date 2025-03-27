import { FluentProvider, webLightTheme } from "@fluentui/react-components";
import { PhotoGrid } from "./PhotoGrid";
import { QrCodePanel } from "./qr/qrCodePanel";
import { useState } from "react";
import { useWebSocket } from "./hooks/useWebSocket";

const SERVER_URL = "bruno-linux:8080"; // ou localhost

const App = () => {
  const [token, setToken] = useState<string | null>(null);

  useWebSocket((receivedToken) => {
    setToken(receivedToken);
  });

  return (
    <FluentProvider
      theme={webLightTheme}
      style={{ height: "100vh", padding: 24 }}
    >
      {!token ? <QrCodePanel /> : <PhotoGrid />}
    </FluentProvider>
  );
};

export default App;
