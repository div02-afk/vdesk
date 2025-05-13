import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import "./App.css";
import LiveWindowCarousel from "./components/live-window-carousel";

interface WindowInfo {
  title: String;
  path: String;
  process_id: number;
  class_name: String;
  desktop_index: number;
}
interface Config {
  id: any;
  data: WindowInfo[];
}

export type { WindowInfo, Config };

function App() {
  const [live_windows, setLiveWindows] = useState<WindowInfo[]>([]);

  const [configId, setConfigId] = useState("");

  async function getOpenWindows() {
    const openWindows: Config = await invoke("send_open_windows");
    setConfigId(openWindows.id);
    setLiveWindows(openWindows.data);
  }

  async function startConfig() {
    console.log("configId", configId);
    const response = await invoke("start_config", {
      configId: configId.toString(),
    });
  }

  return (
    <main className="text-black">
      <LiveWindowCarousel live_windows={live_windows} />
      <button
        onClick={async () => {
          const res = await invoke("save_config", {
            configId: configId.toString(),
          });
        }}
      >
        Save
      </button>
      <button
        onClick={async () => {
          const res = await invoke("read_configs_from_save");
        }}
      >
        Read
      </button>

      <button className="border-2 p-2 rounded-2xl" onClick={getOpenWindows}>
        Get open windows
      </button>
      <button className="border-2 p-2 rounded-2xl" onClick={startConfig}>
        Start config
      </button>
    </main>
  );
}

export default App;
