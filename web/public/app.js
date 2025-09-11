// Хранилище токенов
function saveTokens(t) {
    localStorage.setItem('accessToken', t.access_token);
    localStorage.setItem('accessExp', t.access_token_expiration);
    localStorage.setItem('refreshToken', t.refresh_token);
    localStorage.setItem('refreshExp', t.refresh_token_expiration);
}
function clearTokens() {
    localStorage.removeItem('accessToken');
    localStorage.removeItem('accessExp');
    localStorage.removeItem('refreshToken');
    localStorage.removeItem('refreshExp');
}
function getAccessToken() {
    return localStorage.getItem('accessToken');
}

// Универсальный fetch через прокси с логами
async function api(url, { method='GET', body=null, requireAuth=true, headers={} } = {}) {
    const h = {'Content-Type':'application/json', ...headers};
    if (requireAuth) {
        const token = getAccessToken();
        if (token) h['Authorization'] = 'Bearer ' + token;
    }
    console.log('➡️', method, url, body);
    return fetch(url, { method, headers:h, body });
}

// Утилиты рендера
function table(headers, rowsHtml) {
    return `
    <table>
      <thead><tr>${headers.map(h=>`<th>${h}</th>`).join('')}</tr></thead>
      <tbody>${rowsHtml.join('')}</tbody>
    </table>`;
}
function badge(text, cls='') { return `<span class="badge ${cls}">${text}</span>`; }