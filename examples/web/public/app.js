const API = "/api";

const createForm = document.getElementById("create-user-form");
const usersTableBody = document.querySelector("#users tbody");
const messageEl = document.getElementById("message");
const sseLog = document.getElementById("sse-log");

function showMessage(text, isError = false) {
	messageEl.textContent = text;
	messageEl.className = isError ? "error" : "ok";
}

async function api(path, options = {}) {
	const res = await fetch(API + path, {
		headers: { "Content-Type": "application/json" },
		...options,
	});

	let body;
	try {
		body = await res.json();
	} catch {
		throw new Error(`HTTP ${res.status}: ${res.statusText}`);
	}

	if (!res.ok || body.success === false) {
		const detail = body.message || body.error?.message || JSON.stringify(body);
		throw new Error(detail);
	}

	return body;
}

async function loadUsers() {
	try {
		const res = await api("/users");
		usersTableBody.innerHTML = "";

		for (const user of res.data) {
			const row = document.createElement("tr");

			row.innerHTML = `
				<td>${user.id}</td>
				<td>${user.username}</td>
				<td><button data-delete="${user.id}">Delete</button></td>`;

			usersTableBody.appendChild(row);
		}

		showMessage(`Loaded ${res.data.length} users`);
	} catch (err) {
		showMessage(`Failed to load users: ${err.message}`, true);
	}
}

async function createUser(ev) {
	ev.preventDefault();
	const formData = new FormData(ev.target);
	const payload = {
		username: formData.get("username"),
		password: formData.get("password"),
	};

	try {
		const res = await api("/users", {
			method: "POST",
			body: JSON.stringify(payload),
		});
		showMessage(`User "${res.data.username}" created`);
		ev.target.reset();
		await loadUsers();
	} catch (err) {
		showMessage(`Create failed: ${err.message}`, true);
	}
}

async function deleteUser(id) {
	try {
		const res = await api(`/users/${id}`, { method: "DELETE" });
		showMessage(`User deleted (${res.message})`);
		await loadUsers();
	} catch (err) {
		showMessage(`Delete failed: ${err.message}`, true);
	}
}

function startCountdown() {
	const source = new EventSource(`${API}/sse/countdown`);

	source.addEventListener("countdown", (ev) => appendSse(`countdown: ${ev.data}`));
	source.addEventListener("done", (ev) => {
		appendSse(`done: ${ev.data}`);
		source.close();
	});
	source.onerror = () => appendSse("SSE connection error", true);
}

function appendSse(text, isError = false) {
	const line = document.createElement("div");
	line.textContent = text;
	line.className = isError ? "error" : "ok";
	sseLog.appendChild(line);
}

usersTableBody.addEventListener("click", (ev) => {
	const btn = ev.target.closest("button[data-delete]");
	if (btn) {
		deleteUser(btn.dataset.delete);
	}
});

createForm.addEventListener("submit", createUser);

loadUsers();
startCountdown();
