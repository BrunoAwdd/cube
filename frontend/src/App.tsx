import "./App.css";

import { FluentProvider, Label, webLightTheme } from "@fluentui/react-components";
import { useEffect, useRef, useState } from "react";

import { ConnectionStatus } from "./components/ConnectionStatus";
import Database from "@tauri-apps/plugin-sql";
import { Menu } from '@tauri-apps/api/menu';
import { PhotoGrid } from "./PhotoGrid";
import { QrCodePanel } from "./qr/qrCodePanel";
import { TrayIcon } from '@tauri-apps/api/tray';
import { defaultWindowIcon } from '@tauri-apps/api/app';
import { invoke } from "@tauri-apps/api/core";
import { open } from '@tauri-apps/plugin-dialog';

let trayStarted = false;

function App() {
  const [token, setToken] = useState<string | null>(null);
  const [isOnline, setIsOnline] = useState(false);
  const [db, setDb] = useState<any | null>(null);
  
  const menuRef = useRef<TrayIcon | null>(null);

  async function sendConfig (): Promise<string> {
    try {
      const path = await open({
        multiple: false,
        directory: true,
      });

      return await invoke<string>("set_config", { payload: {upload_dir: path} });
    } catch (error) {
      console.error("❌ Falha ao enviar config:", error); 
      return "Erro ao enviar configuração";
    }
  }

  useEffect(() => {
    if (trayStarted) return;
      trayStarted = true;
    if (menuRef.current) return;

    async function startMenu() {
      const menu = await Menu.new({
        items: [
          {
            id: 'quit',
            text: 'Quit',
          },
        ],
      });
      
      const options = {
        "title": "Cube",
        "tooltip": "Cube",
        "menu": menu,
        "icon": await defaultWindowIcon() || "https://raw.githubusercontent.com/cube-app/cube/main/src/assets/icon.png",
      };

      try {
        const tray = await TrayIcon.new(options);
        menuRef.current = tray;
        
        console.log("✅ Menu configurado com sucesso.");
      } catch (error) {
        console.error("❌ Erro ao configurar o menu:", error);
      }
    }

    startMenu();

  }, []);

  // Carrega o banco de dados ao iniciar o app
  useEffect(() => {
    async function initDb() {
      try {
        const dbInstance = await Database.load("sqlite:uploads.db");
        setDb(dbInstance);
        console.log("✅ Banco de dados carregado com sucesso.");
      } catch (error) {
        console.error("❌ Erro ao carregar banco de dados:", error);
      }
    }

    initDb();
  }, []);

  return (
    <main className="container">
      <FluentProvider
        theme={webLightTheme}
        style={{ height: "100vh", padding: 24, position: "relative" }}
      >
        <button onClick={() => sendConfig()}>Enviar</button>
        <div style={{ paddingBottom: 16 }}><ConnectionStatus online={isOnline} /></div>
        {!token ? <QrCodePanel /> : <PhotoGrid />}
      </FluentProvider>
    </main>
  );
}

export default App;
