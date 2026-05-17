const fileInput = document.getElementById('file-input');
const uploadList = document.getElementById('upload-list');
const serverName = document.getElementById('server-name');
const token = new URLSearchParams(window.location.search).get('token') || '';

function describeDevice() {
  const ua = navigator.userAgent;
  let family = '移动设备';
  let browser = '浏览器';

  if (ua.includes('iPhone')) {
    family = 'iPhone';
  } else if (ua.includes('iPad')) {
    family = 'iPad';
  } else if (ua.includes('Android') && ua.includes('Mobile')) {
    family = 'Android 手机';
  } else if (ua.includes('Android')) {
    family = 'Android 设备';
  }

  if (ua.includes('EdgiOS') || ua.includes('EdgA') || ua.includes('Edg/')) {
    browser = 'Edge';
  } else if (ua.includes('CriOS') || ua.includes('Chrome/')) {
    browser = 'Chrome';
  } else if (ua.includes('FxiOS') || ua.includes('Firefox/')) {
    browser = 'Firefox';
  } else if (ua.includes('Version/') && ua.includes('Safari/')) {
    browser = 'Safari';
  }

  return `${family} · ${browser}`;
}

function createUploadItem(file) {
  const root = document.createElement('article');
  root.className = 'upload-item';
  root.innerHTML = `
    <div class="upload-item__row">
      <strong>${file.name}</strong>
      <span data-role="percent">0%</span>
    </div>
    <div class="upload-progress"><div data-role="bar"></div></div>
    <div class="upload-item__row">
      <span data-role="status">等待上传</span>
      <button type="button" data-role="cancel">取消</button>
    </div>
  `;
  uploadList.prepend(root);
  return root;
}

async function loadInfo() {
  const response = await fetch(`/api/info?token=${encodeURIComponent(token)}`);
  if (!response.ok) {
    throw new Error('读取服务信息失败');
  }

  const info = await response.json();
  serverName.textContent = `${info.name} ${info.version} · ${info.ip}:${info.port}`;
}

async function updatePresence() {
  await fetch(`/api/presence?token=${encodeURIComponent(token)}`, {
    method: 'POST',
    headers: {
      'x-device-name': describeDevice(),
    },
    keepalive: true,
  });
}

function disconnectPresence() {
  if (!navigator.sendBeacon) {
    return;
  }

  navigator.sendBeacon(`/api/disconnect?token=${encodeURIComponent(token)}`);
}

function uploadFile(file, item, attempt = 0) {
  return new Promise((resolve, reject) => {
    const percent = item.querySelector('[data-role="percent"]');
    const bar = item.querySelector('[data-role="bar"]');
    const status = item.querySelector('[data-role="status"]');
    const cancelButton = item.querySelector('[data-role="cancel"]');
    const xhr = new XMLHttpRequest();
    let abortedByUser = false;

    const formData = new FormData();
    formData.append('device', describeDevice());
    formData.append('filename', file.name);
    formData.append('filesize', String(file.size));
    formData.append('file', file);

    xhr.open('POST', `/api/upload?token=${encodeURIComponent(token)}`);

    xhr.upload.onprogress = (event) => {
      if (!event.lengthComputable) {
        return;
      }

      const value = Math.floor((event.loaded / event.total) * 100);
      percent.textContent = `${value}%`;
      bar.style.width = `${value}%`;
      status.textContent = `上传中（第 ${attempt + 1} 次）`;
    };

    xhr.onload = () => {
      if (xhr.status >= 200 && xhr.status < 300) {
        percent.textContent = '100%';
        bar.style.width = '100%';
        status.textContent = '上传完成';
        cancelButton.remove();
        resolve(undefined);
        return;
      }

      if (xhr.status < 500 || attempt >= 9) {
        status.textContent = '上传失败';
        status.classList.add('upload-error');
        reject(new Error(xhr.responseText || 'upload failed'));
        return;
      }

      status.textContent = '网络波动，正在重试...';
      setTimeout(() => {
        uploadFile(file, item, attempt + 1).then(resolve).catch(reject);
      }, 1000);
    };

    xhr.onerror = () => {
      if (abortedByUser) {
        status.textContent = '已取消';
        resolve(undefined);
        return;
      }

      if (attempt >= 9) {
        status.textContent = '上传失败';
        status.classList.add('upload-error');
        reject(new Error('network error'));
        return;
      }

      status.textContent = '连接断开，正在重试...';
      setTimeout(() => {
        uploadFile(file, item, attempt + 1).then(resolve).catch(reject);
      }, 1000);
    };

    xhr.onabort = () => {
      status.textContent = '已取消';
      resolve(undefined);
    };

    cancelButton.onclick = () => {
      abortedByUser = true;
      xhr.abort();
    };

    xhr.send(formData);
  });
}

fileInput.addEventListener('change', () => {
  const files = Array.from(fileInput.files || []);
  files.forEach((file) => {
    const item = createUploadItem(file);
    uploadFile(file, item).catch(() => {
      const status = item.querySelector('[data-role="status"]');
      status.textContent = '上传失败';
      status.classList.add('upload-error');
    });
  });
  fileInput.value = '';
});

loadInfo().catch(() => {
  serverName.textContent = '服务信息读取失败，请重新扫码';
});

updatePresence().catch(() => {
  console.warn('presence update failed');
});

window.addEventListener('pagehide', disconnectPresence);
window.addEventListener('beforeunload', disconnectPresence);
