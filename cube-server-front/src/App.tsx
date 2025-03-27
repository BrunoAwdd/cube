import { useState } from "react";
import { FluentProvider, webLightTheme } from "@fluentui/react-components";
import { PhotoGrid } from "./PhotoGrid";
import { QrCard } from "./QrCard";

const App = () => {
  const [isSynced, setIsSynced] = useState(false);

  const serverAddress = "http://bruno-linux:8080"; // substitua conforme necessário

  return (
    <FluentProvider
      theme={webLightTheme}
      style={{ height: "100vh", padding: 24 }}
    >
      {!isSynced ? (
        <QrCard value={serverAddress} onConfirmed={() => setIsSynced(true)} />
      ) : (
        <PhotoGrid />
      )}
    </FluentProvider>
  );
};

export default App;
