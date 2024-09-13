import { createSignal, For, onCleanup } from "solid-js";
import { Channel, invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [items, setItems] = createSignal([]);
  const onEvent = new Channel();

  const streamItems = async () => {
    const currentTime = new Date().toLocaleTimeString();
    onEvent.onmessage = (message) => {
      console.log(currentTime, "Received message:", message);
      setItems((prev) => [...prev, message]);
    };

    try {
      await invoke("stream_items", { onEvent });
    } catch (error) {
      console.error("Error invoking stream_items:", error);
    }

    console.log("Subscribed:", onEvent);
  };

  streamItems();

  onCleanup(() => {
    console.log("Cleaning up subscription", onEvent);
  });

  return (
    <main>
      <h1>Welcome to Tauri!</h1>
      <ul>
        <For each={items()}>
          {(item) => <li>{item.message}</li>}
        </For>
      </ul>
    </main>
  );
}

export default App;
