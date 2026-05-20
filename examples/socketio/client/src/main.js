import "./style.css";
import { io } from "socket.io-client";

const app = document.querySelector("#app");

app.innerHTML = `
  <main class="chat-app">
    <header class="header">
      <h1>Sword Socket.IO Chat</h1>
      <span id="status" class="status status-connecting">connecting...</span>
    </header>

    <section id="messages" class="messages" aria-live="polite"></section>

    <form id="composer" class="composer">
      <input id="messageInput" type="text" placeholder="Type your message..." autocomplete="off" />
      <button type="submit">Send</button>
    </form>
  </main>
`;

const statusEl = document.querySelector("#status");
const messagesEl = document.querySelector("#messages");
const composerEl = document.querySelector("#composer");
const inputEl = document.querySelector("#messageInput");

const socket = io("http://localhost:8081/chat", {
    path: "/api/socket.io",
});

function setStatus(text, cssClass) {
    statusEl.textContent = text;
    statusEl.className = `status ${cssClass}`;
}

function renderMessages(messages) {
    if (!messages.length) {
        messagesEl.innerHTML = `<p class="empty">No messages yet</p>`;
        return;
    }

    messagesEl.innerHTML = messages
        .map(
            (message) => `
        <article class="message">
          <time>${new Date(message.timestamp).toLocaleTimeString()}</time>
          <p>${message.content}</p>
        </article>
      `,
        )
        .join("");

    messagesEl.scrollTop = messagesEl.scrollHeight;
}

socket.on("connect", () => {
    setStatus("connected", "status-connected");
});

socket.on("disconnect", () => {
    setStatus("disconnected", "status-disconnected");
});

socket.on("messages", (messages) => {
    renderMessages(messages);
});

composerEl.addEventListener("submit", (event) => {
    event.preventDefault();

    const content = inputEl.value.trim();
    if (!content) {
        return;
    }

    socket.emit("message", { content });
    inputEl.value = "";
    inputEl.focus();
});

renderMessages([]);
