let token = null;

// Login
async function login() {
    const email = document.getElementById("email").value;
    const password = document.getElementById("password").value;

    const resp = await fetch("/api/v1/auth/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email, password })
    });

    if (!resp.ok) {
        document.getElementById("result").innerText = "Login failed";
        return;
    }

    const data = await resp.json();
    token = data.tokens.access_token;
    document.getElementById("result").innerText = "Logged in!";
    localStorage.setItem("access_token", token);
}

// Logout
async function logout() {
    const resp = await fetch("/api/v1/auth/logout", {
        method: "POST",
        headers: { Authorization: "Bearer " + localStorage.getItem("access_token") }
    });
    if (resp.ok) {
        localStorage.removeItem("access_token");
        document.getElementById("me").innerText = "Logged out.";
    }
}

// Get current user
async function loadMe() {
    const resp = await fetch("/api/v1/me", {
        headers: { Authorization: "Bearer " + localStorage.getItem("access_token") }
    });
    if (!resp.ok) {
        document.getElementById("me").innerText = "Not authorized";
        return;
    }
    const data = await resp.json();
    document.getElementById("me").innerText = JSON.stringify(data, null, 2);
}

// Load events
async function loadEvents() {
    const resp = await fetch("/api/v1/events");
    const data = await resp.json();
    const list = document.getElementById("events");
    list.innerHTML = "";
    data.forEach(ev => {
        const li = document.createElement("li");
        li.innerText = ev.title + " (" + ev.starts_at + ")";
        list.appendChild(li);
    });
}