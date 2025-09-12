const dbg = document.getElementById('debug');

function setTab(hash){
    document.querySelectorAll('nav.tabs a').forEach(a=>{
        a.classList.toggle('active', a.getAttribute('href') === hash);
    });
    document.querySelectorAll('section[id^="page-"]').forEach(s=>s.classList.add('hidden'));
    if(hash==='#events') document.getElementById('page-events').classList.remove('hidden');
    else if(hash==='#queue') document.getElementById('page-queue').classList.remove('hidden');
    else document.getElementById('page-companies').classList.remove('hidden');
}


async function loadMe(){
    const r = await api('/api/v1/me');
    if(!r.ok){
        location.href='/login.html';
        return;
    }
    const me = await r.json();
    document.getElementById('meBadge').innerHTML =
        `${badge(me.role, me.role==='dean'?'ok':'')}&nbsp;${me.email}`;
    return me;
}

async function loadCompanies() {
    const incArchived = document.getElementById('incArchived').checked;
    const q = encodeURIComponent(document.getElementById('companySearch').value || '');
    const url = `/api/v1/companies/admin?includeArchived=${incArchived}&q=${q}`;
    const r = await api(url);
    const list = r.ok ? await r.json() : [];

    const rows = list.map(c => {
        const archived = c.status === 'archived';
        return `<tr>
      <td>${c.name}</td>
      <td>${badge(archived ? 'archived':'active', archived?'err':'ok')}</td>
      <td>${c.eventCount ?? 0}</td>
      <td class="row">
        <button onclick="viewCompanyEvents('${c.id}')">ивенты</button>
        <button onclick="viewCompanyManagers('${c.id}','${c.name}')">менеджеры</button>
        <button onclick="toggleCompany('${c.id}', ${archived})">${archived?'разархивировать':'в архив'}</button>
      </td>
    </tr>`;
    });

    document.getElementById('companiesTableWrap').innerHTML =
        table(['Компания','Статус','Ивентов','Действия'], rows);
}

async function toggleCompany(id, archived){
    const status = archived ? 'active' : 'archived';
    const r = await api(`/api/v1/companies/${id}/status/${status}`, {method:'POST'});
    if(r.ok) loadCompanies();
}

async function createCompanyFlow(){
    const name = prompt('Название компании:');
    if(!name) return;
    const r = await api('/api/v1/companies', {
        method:'POST',
        body: JSON.stringify({ name })
    });
    if(r.ok) loadCompanies();
    else alert('Ошибка создания компании');
}

async function viewCompanyManagers(companyId, companyName){
    const r = await api(`/api/v1/companies/${companyId}/managers`);
    if(!r.ok){
        alert('Не удалось загрузить менеджеров');
        return;
    }
    const list = await r.json();

    const rows = list.map(m=>{
        const uid = m.user_id ?? m.userId;
        return `<tr>
      <td>${m.name}</td><td>${m.email}</td>
      <td>${badge(m.status, m.status==='pending'?'warn': m.status==='rejected'?'err':'ok')}</td>
      <td class="row">
        ${m.status!=='confirmed' && uid ? `<button onclick="setManagerStatus('${encodeURIComponent(companyId)}','${encodeURIComponent(uid)}','confirmed')">подтвердить</button>`:''}
        ${m.status!=='rejected'  && uid ? `<button onclick="setManagerStatus('${encodeURIComponent(companyId)}','${encodeURIComponent(uid)}','rejected')">отклонить</button>`:''}
      </td>
    </tr>`;
    });

    const html = `<div class="card">
    <div class="toolbar"><strong>Менеджеры компании: ${companyName}</strong></div>
    ${table(['Имя','Email','Статус','Действия'], rows)}
  </div>`;
    document.getElementById('companiesTableWrap').innerHTML = html;
}

async function setManagerStatus(companyId, user_id, status){
    if(!user_id){ alert('Не удалось определить user_id менеджера'); return; }
    const r = await api(`/api/v1/companies/${companyId}/managers/${user_id}/status/${status}`, {method:'POST'});
    if(r.ok) loadCompanies();
    else alert('Не удалось изменить статус менеджера');
}

async function viewCompanyEvents(companyId){
    // переключим на вкладку «Ивенты» и подставим фильтр
    history.replaceState(null,'','#events');
    setTab('#events');
    document.getElementById('filterCompany').value = companyId;
    loadEvents();
}

// ---- Ивенты ----
async function fillCompanyFilter(){
    const r = await api('/api/v1/companies/admin?includeArchived=false');
    const list = r.ok ? await r.json() : [];
    const sel = document.getElementById('filterCompany');
    sel.innerHTML = `<option value="">все компании</option>` + list.map(c=>`<option value="${c.id}">${c.name}</option>`).join('');
}

async function loadEvents(){
    const cid = document.getElementById('filterCompany').value;
    const pub = document.getElementById('filterPublished').value;
    const qs = [];
    if(cid) qs.push(`company_id=${encodeURIComponent(cid)}`);
    if(pub) qs.push(`published=${pub}`);
    const url = `/api/v1/events` + (qs.length?`?${qs.join('&')}`:'');
    const r = await api(url);
    const list = r.ok ? await r.json() : [];

    const rows = list.map(e=>`<tr>
    <td>${e.title}<br/><span class="badge">${new Date(e.starts_at).toLocaleString()}</span></td>
    <td>${badge(e.is_published?'published':'draft', e.is_published?'ok':'warn')}</td>
    <td>${e.registered_count ?? 0}/${e.capacity ?? '∞'}</td>
    <td class="row">
      <button onclick="viewRegistrations('${e.id}','${e.title.replace(/"/g,'&quot;')}')">записи</button>
      ${e.is_published
        ? `<button onclick="unpublishEvent('${e.id}')">снять с публикации</button>`
        : `<button onclick="publishEvent('${e.id}')">опубликовать</button>`}
    </td>
  </tr>`);

    document.getElementById('eventsTableWrap').innerHTML =
        table(['Ивент','Статус','Места','Действия'], rows);
}

async function publishEvent(id){
    const r = await api(`/api/v1/events/${id}/publish`, {method:'POST'});
    if(r.ok) loadEvents();
}
async function unpublishEvent(id){
    const r = await api(`/api/v1/events/${id}/unpublish`, {method:'POST'});
    if(r.ok) loadEvents();
}

async function viewRegistrations(eventId, title){
    const r = await api(`/api/v1/events/${eventId}/registrations`);
    const regs = r.ok ? await r.json() : [];
    const rows = regs.map(x=>`<tr><td>${x.student_id}</td><td>${new Date(x.registered_at).toLocaleString()}</td></tr>`);
    document.getElementById('eventsTableWrap').innerHTML =
        `<div class="card">
      <div class="toolbar"><strong>Записавшиеся: ${title}</strong>
        <button class="right" onclick="loadEvents()">← назад</button></div>
      ${table(['student_id','когда'], rows)}
    </div>`;
}

// ---- Лист ожидания ----
async function loadPendingManagers(){
    // Соберём по всем компаниям
    const r = await api('/api/v1/companies/admin?includeArchived=true');
    const companies = r.ok ? await r.json() : [];
    let rows = [];
    for (const c of companies){
        const m = await api(`/api/v1/companies/${c.id}/managers`);
        if(!m.ok) continue;
        const list = await m.json();
        for (const mm of list){
            if (mm.status === 'pending') {
                rows.push(`<tr>
          <td>${mm.name}<br/><span class="badge">${mm.email}</span></td>
          <td>${c.name}</td>
          <td class="row">
            <button onclick="setManagerStatus('${c.id}','${mm.user_id}','confirmed')">подтвердить</button>
            <button onclick="setManagerStatus('${c.id}','${mm.user_id}','rejected')">отклонить</button>
          </td>
        </tr>`);
            }
        }
    }
    document.getElementById('pendingManagersWrap').innerHTML =
        rows.length ? table(['Менеджер','Компания','Действия'], rows)
            : `<div class="badge">Нет заявок менеджеров</div>`;
}

async function loadPendingStudents(){
    // берём created+linked (ожидающие)
    const url = `/api/v1/dean/students?status=created`;
    const url2 = `/api/v1/dean/students?status=linked`;

    const [r1, r2] = await Promise.all([api(url), api(url2)]);
    const list1 = r1.ok ? await r1.json() : [];
    const list2 = r2.ok ? await r2.json() : [];
    const list  = [...list1, ...list2];

    if (!list.length){
        document.getElementById('pendingStudentsWrap').innerHTML =
            `<div class="badge">Нет заявок студентов</div>`;
        return;
    }

    const rows = list.map(s => {
        const statusBadgeCls = s.status === 'linked' ? 'warn' : 'warn';
        return `<tr>
          <td>${s.name}<br/><span class="badge">${s.email}</span></td>
          <td>${badge(s.status, statusBadgeCls)}</td>
          <td class="row">
            <button onclick="approveStudent('${s.id}')">подтвердить</button>
            <button onclick="rejectStudent('${s.id}')">отклонить</button>
          </td>
        </tr>`;
    });

    document.getElementById('pendingStudentsWrap').innerHTML =
        table(['Студент','Статус','Действия'], rows);
}

async function approveStudent(user_id){
    const r = await api(`/api/v1/dean/students/${user_id}/approve`, { method: 'POST' });
    if (r.ok) loadPendingStudents();
    else alert('Не удалось подтвердить студента');
}

async function rejectStudent(user_id){
    const r = await api(`/api/v1/dean/students/${user_id}/reject`, { method: 'POST' });
    if (r.ok) loadPendingStudents();
    else alert('Не удалось отклонить студента');
}

// ---- Навешиваем события и грузим данные ----
window.addEventListener('DOMContentLoaded', async () => {
    // вкладки
    document.querySelectorAll('nav.tabs a').forEach(a=>{
        a.addEventListener('click', (e)=>{ e.preventDefault(); const h=a.getAttribute('href'); history.replaceState(null,'',h); setTab(h); if(h==='#events'){ loadEvents(); } if(h==='#queue'){ loadPendingManagers(); loadPendingStudents(); }});
    });
    setTab(location.hash || '#companies');

    // выход
    document.getElementById('btnLogout').addEventListener('click', async ()=>{
        await api('/api/v1/auth/logout', {method:'POST'});
        clearTokens();
        location.href='/login.html';
    });

    // компании
    document.getElementById('btnReloadCompanies').addEventListener('click', loadCompanies);
    document.getElementById('btnNewCompany').addEventListener('click', createCompanyFlow);
    document.getElementById('incArchived').addEventListener('change', loadCompanies);
    document.getElementById('companySearch').addEventListener('input', debounce(loadCompanies, 300));

    // ивенты
    document.getElementById('btnReloadEvents').addEventListener('click', loadEvents);
    document.getElementById('filterCompany').addEventListener('change', loadEvents);
    document.getElementById('filterPublished').addEventListener('change', loadEvents);

    // загрузка начальных данных
    const me = await loadMe();
    await fillCompanyFilter();
    await loadCompanies();
});

// простой debounce
function debounce(fn, ms){
    let t; return (...args)=>{ clearTimeout(t); t=setTimeout(()=>fn(...args), ms); };
}