import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface WindowInfo {
  id: any;
  data: {
    title: String;
    path: String;
    process_id: number;
    class_name: String;
    desktop_index: number;
  };
}

function App() {
  const [greetMsg, setGreetMsg] = useState({});
  const [name, setName] = useState("");
  const [configId, setConfigId] = useState("");
  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  async function getOpenWindows() {
    const openWindows: WindowInfo = await invoke("send_open_windows");
    setConfigId(openWindows.id);
    setGreetMsg(`${openWindows.id} + ${typeof openWindows.id}`);

    // setGreetMsg(openWindows);
  }

  async function startConfig() {
    console.log("configId", configId);
    const response = await invoke("start_config", {
      configId: configId.toString(),
    });
    setGreetMsg(JSON.stringify(response));
  }

  return (
    <main className="container">
      <button onClick={getOpenWindows}>Get open windows</button>
      <button onClick={startConfig}>Start config</button>
      <p>{JSON.stringify(greetMsg)}</p>
    </main>
  );
}

export default App;
