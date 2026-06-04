<script setup>
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';

const videoUrl = ref('');
const targetFolder = ref('');
const statusMessage = ref('');
const isLoading = ref(false);

// State untuk Loading Progress
const progressPercent = ref(0);
const downloadSpeed = ref('');
const downloadEta = ref('');
const currentStatus = ref('');

let unlistenProgress = null;

// Daftarkan listener saat komponen Vue di-mount
onMounted(async () => {
  unlistenProgress = await listen('download-progress', (event) => {
    const data = event.payload;
    
    // Konversi string persentase ke angka float untuk progress bar css
    const parsedPercent = parseFloat(data.percent);
    progressPercent.value = isNaN(parsedPercent) ? 0 : parsedPercent;
    
    downloadSpeed.value = data.speed;
    downloadEta.value = data.eta;
    currentStatus.value = data.status;
  });
});

// Bersihkan listener saat aplikasi ditutup/ganti halaman demi efisiensi memori
onUnmounted(() => {
  if (unlistenProgress) unlistenProgress();
});

async function selectFolder() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Pilih Folder Penyimpanan Video'
    });
    if (selected) {
      targetFolder.value = selected;
    }
  } catch (error) {
    statusMessage.value = `Gagal membuka dialog: ${error}`;
  }
}

async function handleDownload() {
  if (!videoUrl.value) {
    statusMessage.value = "Masukkan URL terlebih dahulu!";
    return;
  }
  if (!targetFolder.value) {
    statusMessage.value = "Silakan pilih folder penyimpanan terlebih dahulu!";
    return;
  }

  isLoading.value = true;
  statusMessage.value = "";
  progressPercent.value = 0;
  downloadSpeed.value = '';
  downloadEta.value = '';
  currentStatus.value = 'Memulai unduhan...';

  try {
    const res = await invoke('download_video', {
      url: videoUrl.value,
      downloadPath: targetFolder.value
    });
    statusMessage.value = res;
    currentStatus.value = 'Selesai';
    videoUrl.value = ''; 
  } catch (error) {
    statusMessage.value = `Gagal: ${error}`;
    currentStatus.value = 'Error';
  } {
    isLoading.value = false;
  }
}
</script>

<template>
  <div class="container">
    <header class="header">
      <h1>Tamtek Downloader</h1>
      <p class="subtitle">Sederhana, Ringan & Cepat</p>
    </header>
    
    <div class="card">
      <div class="form-group">
        <label for="url-input">URL Video (YT, TikTok, Instagram & X):</label>
        <input 
          id="url-input"
          v-model="videoUrl" 
          type="text" 
          placeholder="Paste link video di sini..." 
          :disabled="isLoading"
        />
      </div>

      <div class="form-group">
        <label>Lokasi Penyimpanan:</label>
        <div class="folder-picker-row">
          <input 
            v-model="targetFolder" 
            type="text" 
            placeholder="Belum ada folder yang dipilih..." 
            readonly
            class="folder-input"
          />
          <button @click="selectFolder" class="btn-select" :disabled="isLoading">Pilih Folder</button>
        </div>
      </div>

      <button @click="handleDownload" class="btn-download" :disabled="isLoading">
        <span v-if="isLoading">Proses Mengunduh...</span>
        <span v-else>Mulai Download</span>
      </button>

      <div v-if="isLoading || progressPercent > 0" class="progress-section">
        <div class="progress-info">
          <span class="status-text">{{ currentStatus }}</span>
          <span class="percent-text">{{ progressPercent }}%</span>
        </div>
        
        <div class="progress-bar-container">
          <div class="progress-bar-fill" :style="{ width: progressPercent + '%' }"></div>
        </div>

        <div class="progress-meta" v-if="downloadSpeed && downloadSpeed !== '-'">
          <span>⚡ Kecepatan: <strong>{{ downloadSpeed }}</strong></span>
          <span>⏳ Estimasi (ETA): <strong>{{ downloadEta }}</strong></span>
        </div>
      </div>

      <div v-if="statusMessage" class="status-box" :class="{ 'success-box': currentStatus === 'Selesai' }">
        <p class="status">{{ statusMessage }}</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
:global(body) {
  margin: 0;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  background-color: #f4f6f9;
  color: #333;
}

.container {
  max-width: 550px;
  margin: 50px auto;
  padding: 0 20px;
}

.header {
  text-align: center;
  margin-bottom: 30px;
}

.header h1 {
  margin: 0;
  font-size: 2rem;
  color: #2c3e50;
}

.subtitle {
  margin: 5px 0 0;
  color: #7f8c8d;
  font-size: 0.95rem;
}

.card {
  background: #ffffff;
  padding: 30px;
  border-radius: 12px;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.05), 0 1px 3px rgba(0, 0, 0, 0.1);
}

.form-group {
  margin-bottom: 20px;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 600;
  color: #4a5568;
  font-size: 0.9rem;
}

input[type="text"] {
  width: 100%;
  padding: 12px;
  box-sizing: border-box;
  border: 2px solid #e2e8f0;
  border-radius: 8px;
  font-size: 1rem;
  transition: border-color 0.2s;
}

input[type="text"]:focus:not([readonly]) {
  outline: none;
  border-color: #42b883;
}

.folder-picker-row {
  display: flex;
  gap: 10px;
}

.folder-input {
  background-color: #f8fafc;
  cursor: default;
}

.btn-select {
  padding: 0 16px;
  background-color: #475569;
  color: white;
  border: none;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition: background-color 0.2s;
}

.btn-select:hover:not(:disabled) {
  background-color: #334155;
}

.btn-download {
  width: 100%;
  padding: 14px;
  background-color: #42b883;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 1rem;
  font-weight: bold;
  cursor: pointer;
  margin-top: 10px;
  transition: background-color 0.2s;
}

.btn-download:hover:not(:disabled) {
  background-color: #33a06f;
}

.btn-download:disabled, .btn-select:disabled {
  background-color: #cbd5e1;
  cursor: not-allowed;
}

/* KOTAK PROGRESS BAR */
.progress-section {
  margin-top: 25px;
  padding: 15px;
  background-color: #f8fafc;
  border-radius: 8px;
  border: 1px solid #e2e8f0;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
  font-size: 0.85rem;
}

.status-text {
  font-weight: 600;
  color: #475569;
}

.percent-text {
  font-weight: bold;
  color: #42b883;
}

.progress-bar-container {
  width: 100%;
  height: 10px;
  background-color: #e2e8f0;
  border-radius: 5px;
  overflow: hidden;
}

.progress-bar-fill {
  height: 100%;
  background-color: #42b883;
  width: 0%;
  transition: width 0.2s ease-out;
}

.progress-meta {
  display: flex;
  justify-content: space-between;
  margin-top: 10px;
  font-size: 0.8rem;
  color: #64748b;
}

/* KOTAK STATUS AKHIR */
.status-box {
  margin-top: 20px;
  padding: 12px;
  background-color: #fff1f2;
  border-left: 4px solid #ef4444;
  border-radius: 4px;
}

.success-box {
  background-color: #f0fdf4;
  border-left-color: #22c55e;
}

.status {
  margin: 0;
  font-size: 0.9rem;
  line-height: 1.4;
  color: #334155;
  word-break: break-all;
}
</style>