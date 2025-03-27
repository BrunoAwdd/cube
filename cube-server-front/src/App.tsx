import { FluentProvider, webLightTheme } from "@fluentui/react-components";
import { PhotoGrid } from "./PhotoGrid";

const App = () => (
  <FluentProvider
    theme={webLightTheme}
    style={{ height: "100vh", padding: 24 }}
  >
    <PhotoGrid />
  </FluentProvider>
);

export default App;
