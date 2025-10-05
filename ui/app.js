// Mock Tauri API for development
const mockTauri = {
    invoke: async (cmd, args) => {
        console.log(`Tauri command: ${cmd}`, args);

        switch(cmd) {
            case 'list_env_vars':
                return [
                    { key: 'AWS_ACCESS_KEY', value: '***hidden***', timestamp: Date.now(), machine_id: 'dev-machine' },
                    { key: 'DATABASE_URL', value: '***hidden***', timestamp: Date.now(), machine_id: 'dev-machine' }
                ];
            case 'get_peers':
                return [
                    { id: 'peer-1', address: '192.168.1.100', last_seen: Date.now() },
                    { id: 'peer-2', address: '192.168.1.101', last_seen: Date.now() }
                ];
            default:
                return null;
        }
    }
};

// Use real Tauri API if available, otherwise use mock
const tauri = window.__TAURI__ || mockTauri;

// DOM elements
const varList = document.getElementById('varList');
const addModal = document.getElementById('addModal');
const peersModal = document.getElementById('peersModal');
const addBtn = document.getElementById('addBtn');
const syncBtn = document.getElementById('syncBtn');
const peersBtn = document.getElementById('peersBtn');
const saveBtn = document.getElementById('saveBtn');
const cancelBtn = document.getElementById('cancelBtn');
const closePeersBtn = document.getElementById('closePeersBtn');
const keyInput = document.getElementById('keyInput');
const valueInput = document.getElementById('valueInput');
const peersList = document.getElementById('peersList');

// Load environment variables
async function loadVars() {
    try {
        const vars = await tauri.invoke('list_env_vars');
        renderVars(vars);
    } catch (error) {
        console.error('Error loading vars:', error);
        varList.innerHTML = '<div style="padding: 20px; text-align: center; color: #999;">No variables yet. Click "Add Variable" to get started.</div>';
    }
}

// Render variables list
function renderVars(vars) {
    if (!vars || vars.length === 0) {
        varList.innerHTML = '<div style="padding: 20px; text-align: center; color: #999;">No variables yet. Click "Add Variable" to get started.</div>';
        return;
    }

    varList.innerHTML = vars.map(v => `
        <div class="var-item">
            <div class="var-info">
                <div class="var-key">${v.key}</div>
                <div class="var-value">${v.value}</div>
            </div>
            <div class="var-actions">
                <button class="btn btn-secondary" onclick="deleteVar('${v.key}')">Delete</button>
            </div>
        </div>
    `).join('');
}

// Load peers
async function loadPeers() {
    try {
        const peers = await tauri.invoke('get_peers');
        renderPeers(peers);
    } catch (error) {
        console.error('Error loading peers:', error);
        peersList.innerHTML = '<div style="padding: 20px; text-align: center; color: #999;">No peers connected</div>';
    }
}

// Render peers list
function renderPeers(peers) {
    if (!peers || peers.length === 0) {
        peersList.innerHTML = '<div style="padding: 20px; text-align: center; color: #999;">No peers connected</div>';
        return;
    }

    peersList.innerHTML = peers.map(p => `
        <div class="peer-item">
            <div><strong>ID:</strong> <span class="peer-id">${p.id}</span></div>
            <div><strong>Address:</strong> ${p.address}</div>
            <div><strong>Last Seen:</strong> ${new Date(p.last_seen).toLocaleString()}</div>
        </div>
    `).join('');
}

// Delete variable
async function deleteVar(key) {
    if (!confirm(`Delete ${key}?`)) return;

    try {
        await tauri.invoke('delete_env_var', { key });
        await loadVars();
    } catch (error) {
        console.error('Error deleting var:', error);
        alert('Failed to delete variable');
    }
}

// Event listeners
addBtn.addEventListener('click', () => {
    addModal.classList.remove('hidden');
    keyInput.focus();
});

cancelBtn.addEventListener('click', () => {
    addModal.classList.add('hidden');
    keyInput.value = '';
    valueInput.value = '';
});

saveBtn.addEventListener('click', async () => {
    const key = keyInput.value.trim();
    const value = valueInput.value.trim();

    if (!key || !value) {
        alert('Both key and value are required');
        return;
    }

    try {
        await tauri.invoke('set_env_var', { key, value });
        addModal.classList.add('hidden');
        keyInput.value = '';
        valueInput.value = '';
        await loadVars();
    } catch (error) {
        console.error('Error saving var:', error);
        alert('Failed to save variable');
    }
});

syncBtn.addEventListener('click', async () => {
    try {
        await tauri.invoke('trigger_sync');
        alert('Sync triggered successfully');
        await loadVars();
    } catch (error) {
        console.error('Error syncing:', error);
        alert('Failed to trigger sync');
    }
});

peersBtn.addEventListener('click', async () => {
    await loadPeers();
    peersModal.classList.remove('hidden');
});

closePeersBtn.addEventListener('click', () => {
    peersModal.classList.add('hidden');
});

// Initial load
loadVars();
