// Token 管理功能
function saveAuthToken(token) {
  const expiryTime = new Date().getTime() + (24 * 60 * 60 * 1000); // 24小时后过期
  localStorage.setItem('authToken', token);
  localStorage.setItem('authTokenExpiry', expiryTime);
}

function getAuthToken() {
  const token = localStorage.getItem('authToken');
  const expiry = localStorage.getItem('authTokenExpiry');

  if (!token || !expiry) {
    return null;
  }

  if (new Date().getTime() > parseInt(expiry)) {
    localStorage.removeItem('authToken');
    localStorage.removeItem('authTokenExpiry');
    return null;
  }

  return token;
}

// 消息显示功能
function showMessage(elementId, text, isError = false) {
  const msg = document.getElementById(elementId);
  msg.className = `message ${isError ? 'error' : 'success'}`;
  msg.textContent = text;
}

function showGlobalMessage(text, isError = false) {
  showMessage('message', text, isError);
  // 3秒后自动清除消息
  setTimeout(() => {
    const msg = document.getElementById('message');
    msg.textContent = '';
    msg.className = 'message';
  }, 3000);
}

// Token 输入框自动填充和事件绑定
function initializeTokenHandling(inputId) {
  document.addEventListener('DOMContentLoaded', () => {
    const authToken = getAuthToken();
    if (authToken) {
      document.getElementById(inputId).value = authToken;
    }
  });

  document.getElementById(inputId).addEventListener('change', (e) => {
    if (e.target.value) {
      saveAuthToken(e.target.value);
    } else {
      localStorage.removeItem('authToken');
      localStorage.removeItem('authTokenExpiry');
    }
  });
}

// API 请求通用处理
async function makeAuthenticatedRequest(url, options = {}) {
  const tokenId = options.tokenId || 'authToken';
  const token = document.getElementById(tokenId).value;

  if (!token) {
    showGlobalMessage('请输入 AUTH_TOKEN', true);
    return null;
  }

  const defaultOptions = {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json'
    }
  };

  try {
    const response = await fetch(url, { ...defaultOptions, ...options });

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    return await response.json();
  } catch (error) {
    showGlobalMessage(`请求失败: ${error.message}`, true);
    return null;
  }
}

/**
 * 从字符串解析布尔值
 * @param {string} str - 要解析的字符串
 * @param {boolean|null} defaultValue - 解析失败时的默认值
 * @returns {boolean|null} 解析结果，如果无法解析则返回默认值
 */
function parseBooleanFromString(str, defaultValue = null) {
  if (typeof str !== 'string') {
    return defaultValue;
  }

  const lowercaseStr = str.toLowerCase().trim();

  if (lowercaseStr === 'true' || lowercaseStr === '1') {
    return true;
  } else if (lowercaseStr === 'false' || lowercaseStr === '0') {
    return false;
  } else {
    return defaultValue;
  }
}

/**
 * 将布尔值转换为字符串
 * @param {boolean|undefined|null} value - 要转换的布尔值
 * @param {string} defaultValue - 转换失败时的默认值
 * @returns {string} 转换结果，如果输入无效则返回默认值
 */
function parseStringFromBoolean(value, defaultValue = null) {
  if (typeof value !== 'boolean') {
    return defaultValue;
  }

  return value ? 'true' : 'false';
}

/**
 * 解析对话内容
 * @param {string} promptStr - 原始prompt字符串
 * @returns {Array<{role: string, content: string}>} 解析后的对话数组
 */
function parsePrompt(promptStr) {
  if (!promptStr) return [];

  const messages = [];
  const lines = promptStr.split('\n');
  let currentRole = '';
  let currentContent = '';

  const roleMap = {
    'BEGIN_SYSTEM': 'system',
    'BEGIN_USER': 'user',
    'BEGIN_ASSISTANT': 'assistant'
  };

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];

    // 检查是否是角色标记行
    let foundRole = false;
    for (const [marker, role] of Object.entries(roleMap)) {
      if (line.includes(marker)) {
        // 保存之前的消息（如果有）
        if (currentRole && currentContent.trim()) {
          messages.push({
            role: currentRole,
            content: currentContent.trim()
          });
        }
        // 设置新角色
        currentRole = role;
        currentContent = '';
        foundRole = true;
        break;
      }
    }

    // 如果不是角色标记行且不是END标记行，则添加到当前内容
    if (!foundRole && !line.includes('END_')) {
      currentContent += line + '\n';
    }
  }

  // 添加最后一条消息
  if (currentRole && currentContent.trim()) {
    messages.push({
      role: currentRole,
      content: currentContent.trim()
    });
  }

  return messages;
}

/**
 * 格式化对话内容为HTML表格
 * @param {Array<{role: string, content: string}>} messages - 对话消息数组
 * @returns {string} HTML表格字符串
 */
function formatPromptToTable(messages) {
  if (!messages || messages.length === 0) {
    return '<p>无对话内容</p>';
  }

  const roleLabels = {
    'system': '系统',
    'user': '用户',
    'assistant': '助手'
  };

  function escapeHtml(content) {
    // 先转义HTML特殊字符
    const escaped = content
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#039;');

    // 将HTML标签文本用引号包裹，使其更易读
    // return escaped.replace(/&lt;(\/?[^>]+)&gt;/g, '"<$1>"');
    return escaped;
  }

  return `<table class="message-table"><thead><tr><th>角色</th><th>内容</th></tr></thead><tbody>${messages.map(msg => `<tr><td>${roleLabels[msg.role] || msg.role}</td><td>${escapeHtml(msg.content).replace(/\n/g, '<br>')}</td></tr>`).join('')}</tbody></table>`;
}

/**
 * 安全地显示prompt对话框
 * @param {string} promptStr - 原始prompt字符串
 */
function showPromptModal(promptStr) {
  try {
    const modal = document.getElementById('promptModal');
    const content = document.getElementById('promptContent');

    if (!modal || !content) {
      console.error('Modal elements not found');
      return;
    }

    const messages = parsePrompt(promptStr);
    content.innerHTML = formatPromptToTable(messages);
    modal.style.display = 'block';
  } catch (e) {
    console.error('显示prompt对话框失败:', e);
    console.error('原始prompt:', promptStr);
  }
}