<script lang="ts">
  // Uncomment the commented lines below to use MessagePack serialization

  import { io, type Socket } from "socket.io-client";
  // import msgpackParser from "socket.io-msgpack-parser";

  type Message = {
    id: string;
    content: string;
    timestamp: number;
  };

  let inputMessage = $state<string>("");

  let socket = $state<Socket>(
    io("http://localhost:8081/chat", {
      // parser: msgpackParser,
    })
  );
  let messages = $state<Message[]>([]);

  socket.on("connect", () => {
    console.log("Connected to server");
  });

  socket.on("messages", (receivedMessages: Message[]) => {
    console.log("Message received:", receivedMessages);
    messages = receivedMessages;
  });

  async function sendMessage() {
    if (inputMessage.trim() === "") return;

    socket.emit("message", { content: inputMessage });
    inputMessage = "";
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      event.preventDefault();
      sendMessage();
    }
  }
</script>

<main>
  <h1>Sword Socket.IO Adapter Chat</h1>

  <div class="chat-container">
    {#each messages as message}
      <div class="message">
        <strong>{new Date(message.timestamp).toLocaleTimeString()}:</strong>
        <span>{message.content}</span>
      </div>
    {/each}
  </div>

  <div class="input-container">
    <input
      type="text"
      bind:value={inputMessage}
      placeholder="Type your message..."
      onkeydown={handleKeydown}
    />

    <button onclick={sendMessage}>Send</button>
  </div>
</main>

<style>
  main {
    font-family: Arial, sans-serif;
    max-width: 600px;
    margin: 0 auto;
    padding: 20px;
    background-color: #f5f5f5;
    border-radius: 10px;
    box-shadow: 0 0 10px rgba(0, 0, 0, 0.1);
  }

  h1 {
    text-align: center;
    color: #333;
  }

  .chat-container {
    height: 400px;
    overflow-y: auto;
    background-color: #fff;
    border: 1px solid #ddd;
    border-radius: 5px;
    padding: 10px;
    margin-bottom: 20px;
  }

  .message {
    margin-bottom: 10px;
    padding: 8px 12px;
    background-color: #e1f5fe;
    border-radius: 10px;
    max-width: 70%;
  }

  .input-container {
    display: flex;
    gap: 10px;
  }

  input {
    flex: 1;
    padding: 10px;
    border: 1px solid #ddd;
    border-radius: 5px;
    font-size: 16px;
  }

  button {
    padding: 10px 20px;
    background-color: #007bff;
    color: white;
    border: none;
    border-radius: 5px;
    cursor: pointer;
    font-size: 16px;
  }

  button:hover {
    background-color: #0056b3;
  }
</style>
