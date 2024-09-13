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
      setItems((prev) => [message, ...prev]);
    };

    try {
      await invoke("stream_items", { onEvent });
      console.log("Subscribed:", onEvent);
    } catch (error) {
      console.error("Error invoking stream_items:", error);
    }
  };

  streamItems();

  onCleanup(() => {
    invoke("stream_items_stop", { id: onEvent.id });
  });

  return (
    <main>
      <p>stream</p>
      <ul>
        <For each={items()}>
          {(item) => <li>{item.topic}</li>}
        </For>
      </ul>
    </main>
  );
}

export default App;
