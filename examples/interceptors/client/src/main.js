import { io } from "socket.io-client";

fetch("http://localhost:8081/")
	.then((response) => response.json())
	.then((data) => console.log("Response from / endpoint:", data))
	.catch((error) => console.error("Error fetching / endpoint:", error));

const socket = io("http://localhost:8081/events");

console.log(socket);

socket.on("connect", () => {
	console.log("Connected to server with ID:", socket.id);
});

socket.emit("event", { content: "Hello from the client!" });

const sendResponse = await socket.emitWithAck("eventWithAck", {
	content: "Hello with Ack!",
});

console.log("Received acknowledgment from server:", sendResponse);
