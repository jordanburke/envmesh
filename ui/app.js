const { invoke } = window.__TAURI__.core;

async function loadEnvVars() {
    try {
        const vars = await invoke('list_env_vars');
        const list = document.getElementById('env-list');

        if (vars.length === 0) {
            list.innerHTML = '<div class="empty-state">No environment variables</div>';
            return;
        }

        list.innerHTML = vars.map(v => '<div class="env-item"><div><span class="env-key">' + v.key + '</span><span class="env-value">' + v.value + '</span></div><button class="delete-btn" onclick="deleteEnvVar(\'' + v.key + '\')">Delete</button></div>').join('');
    } catch (error) {
        console.error('Failed to load env vars:', error);
    }
}

async function loadPeers() {
    try {
        const peers = await invoke('get_peers');
        const list = document.getElementById('peer-list');

        if (peers.length === 0) {
            list.innerHTML = '<div class="empty-state">No connected peers</div>';
            return;
        }

        list.innerHTML = peers.map(p => '<div class="peer-item"><span class="peer-id">' + p.id + '</span><span>' + p.address + '</span></div>').join('');
    } catch (error) {
        console.error('Failed to load peers:', error);
    }
}

async function addEnvVar() {
    const key = document.getElementById('key').value.trim();
    const value = document.getElementById('value').value.trim();

    if (!key || !value) {
        alert('Please enter both key and value');
        return;
    }

    try {
        await invoke('set_env_var', { key, value });
        document.getElementById('key').value = '';
        document.getElementById('value').value = '';
        await loadEnvVars();
    } catch (error) {
        alert('Failed to add variable: ' + error);
    }
}

async function deleteEnvVar(key) {
    try {
        await invoke('delete_env_var', { key });
        await loadEnvVars();
    } catch (error) {
        alert('Failed to delete variable: ' + error);
    }
}

async function triggerSync() {
    try {
        await invoke('trigger_sync');
        await loadEnvVars();
        await loadPeers();
    } catch (error) {
        alert('Failed to sync: ' + error);
    }
}

document.getElementById('add-btn').addEventListener('click', addEnvVar);
document.getElementById('sync-btn').addEventListener('click', triggerSync);

loadEnvVars();
loadPeers();
setInterval(loadPeers, 5000);
