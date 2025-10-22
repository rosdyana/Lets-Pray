<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface PrayerTime {
  name: string;
  time: string;
  datetime: string;
}

interface AppSettings {
  location: string;
  play_sound: boolean;
  enabled_prayers: string[];
  run_at_startup: boolean;
}

const settings = ref<AppSettings>({
  location: "New Taipei City",
  play_sound: true,
  enabled_prayers: ["Fajr", "Dhuhr", "Asr", "Maghrib", "Isha"],
  run_at_startup: false,
});

const prayerTimes = ref<PrayerTime[]>([]);
const loading = ref(false);
const error = ref("");
const soundState = ref<'idle' | 'playing' | 'stopping'>('idle');
const currentAudio = ref<HTMLAudioElement | null>(null);
const activePrayer = ref<string | null>(null);
const activeTab = ref<'prayers' | 'settings'>('prayers');
const activeSettingsSubMenu = ref<'general' | 'prayer' | 'system'>('general');
const locationInputTimeout = ref<number | null>(null);
const detectingLocation = ref(false);

const prayerNames = ["Fajr", "Dhuhr", "Asr", "Maghrib", "Isha"];

async function loadSettings() {
  try {
    settings.value = await invoke("get_settings");
  } catch (err) {
    console.error("Failed to load settings:", err);
  }
}

async function saveSettings() {
  try {
    await invoke("save_settings", { settings: settings.value });
    await fetchPrayerTimes();
  } catch (err) {
    console.error("Failed to save settings:", err);
  }
}

function handleLocationChange() {
  // Clear existing timeout
  if (locationInputTimeout.value) {
    clearTimeout(locationInputTimeout.value);
  }
  
  // Set new timeout to debounce the API call
  locationInputTimeout.value = setTimeout(async () => {
    await saveSettings();
  }, 1000); // Wait 1 second after user stops typing
}

async function autoDetectLocation() {
  detectingLocation.value = true;
  error.value = "";
  try {
    const systemInfo = await invoke("get_system_info");
    if (systemInfo && typeof systemInfo === 'object' && 'location' in systemInfo) {
      const info = systemInfo as any;
      settings.value.location = info.location;
      await saveSettings();
      console.log(`Auto-detected location: ${info.location}, timezone: ${info.timezone}`);
    }
  } catch (err) {
    console.error("Failed to detect location:", err);
    error.value = "Failed to detect system location";
  } finally {
    detectingLocation.value = false;
  }
}

async function fetchPrayerTimes() {
  loading.value = true;
  error.value = "";
  try {
    prayerTimes.value = await invoke("fetch_prayer_times", { 
      location: settings.value.location 
    });
  } catch (err) {
    error.value = `Failed to fetch prayer times: ${err}`;
    console.error("Failed to fetch prayer times:", err);
  } finally {
    loading.value = false;
  }
}

function togglePrayer(prayerName: string) {
  const index = settings.value.enabled_prayers.indexOf(prayerName);
  if (index > -1) {
    settings.value.enabled_prayers.splice(index, 1);
  } else {
    settings.value.enabled_prayers.push(prayerName);
  }
}

function playAdhan() {
  // Stop any currently playing audio
  if (currentAudio.value) {
    currentAudio.value.pause();
    currentAudio.value.currentTime = 0;
    currentAudio.value = null;
  }
  
  // Create new audio element and play adhan sound
  const audio = new Audio();
  currentAudio.value = audio;
  soundState.value = 'playing';
  
  // Try to load the adhan audio file, fallback to a simple beep if not available
  audio.src = "/adhan.mp3";
  audio.onerror = () => {
    // Fallback to a simple beep sound if adhan.mp3 is not available
    audio.src = "data:audio/wav;base64,UklGRnoGAABXQVZFZm10IBAAAAABAAEAQB8AAEAfAAABAAgAZGF0YQoGAACBhYqFbF1fdJivrJBhNjVgodDbq2EcBj+a2/LDciUFLIHO8tiJNwgZaLvt559NEAxQp+PwtmMcBjiR1/LMeSwFJHfH8N2QQAoUXrTp66hVFApGn+DyvmwhBSuBzvLZiTYIG2m98OScTgwOUarm7blmGgU7k9n1unEiBC13yO/eizEIHWq+8+OWT";
    audio.play().catch(console.error);
  };
  
  // Handle audio end
  audio.onended = () => {
    soundState.value = 'idle';
    currentAudio.value = null;
  };
  
  audio.play().catch(console.error);
}

function showSystemNotification(title: string, body: string) {
  // Request notification permission if not already granted
  if (Notification.permission === "default") {
    Notification.requestPermission().then(permission => {
      if (permission === "granted") {
        new Notification(title, { body });
      }
    });
  } else if (Notification.permission === "granted") {
    new Notification(title, { body });
  }
}

function stopAdhan() {
  if (currentAudio.value) {
    currentAudio.value.pause();
    currentAudio.value.currentTime = 0;
    currentAudio.value = null;
    soundState.value = 'idle';
  }
}

async function testAdhanSound() {
  try {
    if (soundState.value === 'playing') {
      // If currently playing, stop it
      stopAdhan();
    } else {
      // If idle, start playing
      await invoke("test_adhan_sound");
      playAdhan();
    }
  } catch (err) {
    console.error("Failed to test adhan sound:", err);
  }
}

onMounted(async () => {
  await loadSettings();
  await fetchPrayerTimes();
  
  // Listen for prayer reminder events from backend
  const unlistenReminder = await listen("prayer-reminder", (event) => {
    const { title, body } = event.payload as { title: string; body: string };
    showSystemNotification(title, body);
    
    // Extract prayer name from title and set as active
    const prayerName = title.replace("Prayer Time: ", "");
    activePrayer.value = prayerName;
    
    // Clear active prayer after 5 minutes
    setTimeout(() => {
      activePrayer.value = null;
    }, 5 * 60 * 1000);
  });
  
  // Listen for adhan play event from backend
  const unlistenAdhan = await listen("play-adhan", () => {
    if (settings.value.play_sound) {
      playAdhan();
    }
  });
  
  onUnmounted(() => {
    unlistenReminder();
    unlistenAdhan();
    // Clear any pending location change timeout
    if (locationInputTimeout.value) {
      clearTimeout(locationInputTimeout.value);
    }
  });
});
</script>

<template>
  <div class="app">
    <header class="header">
      <div class="header-content">
        <div class="logo">
          <span class="crescent">🌙</span>
          <h1>Lets Pray</h1>
        </div>
      </div>
    </header>

    <nav class="tab-nav">
      <button 
        @click="activeTab = 'prayers'"
        :class="{ active: activeTab === 'prayers' }"
        class="tab-btn"
      >
        📅 Prayer Times
      </button>
      <button 
        @click="activeTab = 'settings'"
        :class="{ active: activeTab === 'settings' }"
        class="tab-btn"
      >
        ⚙️ Settings
      </button>
    </nav>

    <main class="main-content">
      <!-- Prayer Times Tab -->
      <div v-if="activeTab === 'prayers'" class="tab-content">
        <div v-if="loading" class="loading">
          Loading prayer times...
        </div>
        
        <div v-else-if="error" class="error">
          {{ error }}
        </div>
        
        <div v-else class="prayer-times">
          <div 
            v-for="prayer in prayerTimes" 
            :key="prayer.name"
            class="prayer-time"
            :class="{ 
              'enabled': settings.enabled_prayers.includes(prayer.name),
              'disabled': !settings.enabled_prayers.includes(prayer.name),
              'active': activePrayer === prayer.name
            }"
          >
            <div class="prayer-name">
              {{ prayer.name }}
              <span v-if="activePrayer === prayer.name" class="active-indicator">🔔</span>
            </div>
            <div class="prayer-time-value">{{ prayer.time }}</div>
          </div>
        </div>
      </div>

      <!-- Settings Tab -->
      <div v-if="activeTab === 'settings'" class="tab-content">
        <!-- Settings Sub-menu -->
        <nav class="settings-sub-nav">
          <button 
            @click="activeSettingsSubMenu = 'general'"
            :class="{ active: activeSettingsSubMenu === 'general' }"
            class="settings-sub-btn"
          >
            ⚙️ General
          </button>
          <button 
            @click="activeSettingsSubMenu = 'prayer'"
            :class="{ active: activeSettingsSubMenu === 'prayer' }"
            class="settings-sub-btn"
          >
            🕌 Prayer
          </button>
          <button 
            @click="activeSettingsSubMenu = 'system'"
            :class="{ active: activeSettingsSubMenu === 'system' }"
            class="settings-sub-btn"
          >
            💻 System
          </button>
        </nav>

        <!-- General Settings -->
        <div v-if="activeSettingsSubMenu === 'general'" class="settings-category">
          <div class="settings-grid">
            <div class="setting-item">
              <label for="location">Location:</label>
              <div class="location-input-group">
                <input 
                  id="location"
                  v-model="settings.location" 
                  type="text" 
                  placeholder="Enter your city"
                  @input="handleLocationChange"
                />
                <button 
                  @click="autoDetectLocation"
                  :disabled="detectingLocation"
                  class="auto-detect-btn"
                  type="button"
                >
                  <span v-if="detectingLocation">🔍 Detecting...</span>
                  <span v-else>🌍 Auto-detect</span>
                </button>
              </div>
            </div>
          </div>
        </div>

        <!-- Prayer Settings -->
        <div v-if="activeSettingsSubMenu === 'prayer'" class="settings-category">
          <div class="settings-grid">
            <div class="setting-item">
              <label class="checkbox-label">
                <input 
                  v-model="settings.play_sound" 
                  type="checkbox"
                  @change="saveSettings"
                />
                <span class="checkmark"></span>
                Play Adhan sound
              </label>
              <button 
                v-if="settings.play_sound"
                @click="testAdhanSound"
                class="test-sound-btn"
                :class="{ 'playing': soundState === 'playing' }"
                type="button"
              >
                <span v-if="soundState === 'idle'">🔊 Test</span>
                <span v-else-if="soundState === 'playing'">⏹️ Stop</span>
              </button>
            </div>

            <div class="setting-item prayer-reminders">
              <label>Enable Prayer Reminders:</label>
              <div class="prayer-checkboxes">
                <label 
                  v-for="prayer in prayerNames" 
                  :key="prayer"
                  class="checkbox-label prayer-checkbox"
                >
                  <input
                    :checked="settings.enabled_prayers.includes(prayer)"
                    type="checkbox"
                    @change="togglePrayer(prayer); saveSettings()"
                  />
                  <span class="checkmark"></span>
                  {{ prayer }}
                </label>
              </div>
            </div>
          </div>
        </div>

        <!-- System Settings -->
        <div v-if="activeSettingsSubMenu === 'system'" class="settings-category">
          <div class="settings-grid">
            <div class="setting-item">
              <label class="checkbox-label">
                <input 
                  v-model="settings.run_at_startup" 
                  type="checkbox"
                  @change="saveSettings"
                />
                <span class="checkmark"></span>
                Run at startup
              </label>
            </div>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

:root {
  /* --- Core Theme Colors --- */
  --primary-green: #2e7d32;       /* Deep vibrant green */
  --secondary-green: #4caf50;     /* Balanced bright green */
  --accent-gold: #ffca28;         /* Warm golden accent */
  --light-bg: #f5fff6;            /* Soft background */
  --card-bg: #ffffff;             /* Card / section background */
  --header-bg: linear-gradient(135deg, #2e7d32, #4caf50);

  /* --- Text & Neutral --- */
  --text-dark: #1f1f1f;
  --text-light: #ffffff;
  --border-color: #d0d7d4;

  /* --- Effects --- */
  --shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
}

/* Global */
body {
  font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
  background: linear-gradient(135deg, var(--light-bg) 0%, #e9f8ec 100%);
  color: var(--text-dark);
  min-height: 100vh;
}

.app {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

/* Header */
.header {
  background: var(--header-bg);
  color: var(--text-light);
  padding: 1rem;
  text-align: center;
  box-shadow: var(--shadow);
}

.header-content {
  max-width: 350px;
  margin: 0 auto;
}

.logo {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
}

.logo h1 {
  font-size: 1.4rem;
  font-weight: 600;
}

.crescent {
  font-size: 1.5rem;
  animation: glow 2s ease-in-out infinite alternate;
}

@keyframes glow {
  from { text-shadow: 0 0 5px var(--accent-gold); }
  to { text-shadow: 0 0 20px var(--accent-gold), 0 0 30px var(--accent-gold); }
}

/* Tab Navigation */
.tab-nav {
  display: flex;
  background: var(--card-bg);
  border-bottom: 1px solid var(--border-color);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
}

.tab-btn {
  flex: 1;
  padding: 0.75rem 1rem;
  background: transparent;
  border: none;
  color: var(--text-dark);
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s ease;
  border-bottom: 3px solid transparent;
}

.tab-btn:hover {
  background: #f8f9fa;
}

.tab-btn.active {
  color: var(--primary-green);
  border-bottom-color: var(--primary-green);
  background: #f0f8f0;
}

/* Main content */
.main-content {
  flex: 1;
  padding: 1rem;
  max-width: 350px;
  margin: 0 auto;
  width: 100%;
}

/* Tab content */
.tab-content {
  min-height: 250px;
}

/* Settings Sub-menu */
.settings-sub-nav {
  display: flex;
  background: #f8f9fa;
  border-radius: 6px;
  padding: 0.2rem;
  margin-bottom: 0.75rem;
  gap: 0.2rem;
}

.settings-sub-btn {
  flex: 1;
  padding: 0.4rem 0.6rem;
  background: transparent;
  border: none;
  color: var(--text-dark);
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s ease;
  border-radius: 4px;
  font-size: 0.75rem;
  white-space: nowrap;
}

.settings-sub-btn:hover {
  background: #e9ecef;
}

.settings-sub-btn.active {
  background: var(--card-bg);
  color: var(--primary-green);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

/* Settings Category */
.settings-category {
  min-height: 150px;
}

/* Settings Grid */
.settings-grid {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.setting-item {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.setting-item.prayer-reminders {
  gap: 0.75rem;
}

/* Location input group */
.location-input-group {
  display: flex;
  gap: 0.5rem;
  align-items: stretch;
}

.location-input-group input {
  flex: 1;
}

.auto-detect-btn {
  padding: 0.6rem 0.8rem;
  background: var(--secondary-green);
  color: white;
  border: none;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
  font-size: 0.8rem;
  white-space: nowrap;
}

.auto-detect-btn:hover:not(:disabled) {
  background: var(--primary-green);
  transform: translateY(-1px);
  box-shadow: 0 2px 6px rgba(46, 125, 50, 0.3);
}

.auto-detect-btn:disabled {
  background: #ccc;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}

/* Inputs */
.setting-item label {
  font-weight: 500;
  font-size: 0.9rem;
}

.setting-item input[type="text"] {
  width: 100%;
  padding: 0.6rem;
  border: 2px solid var(--border-color);
  border-radius: 6px;
  font-size: 0.9rem;
  transition: border-color 0.3s ease;
  background: var(--card-bg);
  color: var(--text-dark);
}

.setting-item input[type="text"]:focus {
  outline: none;
  border-color: var(--secondary-green);
  box-shadow: 0 0 0 2px rgba(76, 175, 80, 0.2);
}

/* Checkbox */
.checkbox-label {
  display: flex;
  align-items: center;
  cursor: pointer;
  font-weight: 500;
  color: var(--text-dark);
  font-size: 0.9rem;
}

.checkbox-label input[type="checkbox"] {
  margin-right: 0.4rem;
  width: 14px;
  height: 14px;
  accent-color: var(--secondary-green);
}

/* Buttons */
.test-sound-btn {
  padding: 0.4rem 0.8rem;
  background: var(--accent-gold);
  color: #000;
  border: none;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
  font-size: 0.8rem;
  align-self: flex-start;
}

.test-sound-btn:hover {
  background: #e0b526;
  transform: translateY(-1px);
  box-shadow: 0 2px 6px rgba(255, 202, 40, 0.3);
}

.test-sound-btn.playing {
  background: #e74c3c;
  color: #fff;
}

.test-sound-btn.playing:hover {
  background: #c0392b;
}

/* Prayer checkboxes */
.prayer-checkboxes {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.3rem;
}

.prayer-checkbox {
  background: var(--card-bg);
  padding: 0.3rem 0.5rem;
  border-radius: 5px;
  border: 1px solid var(--border-color);
  font-weight: 600;
  font-size: 0.75rem;
  transition: all 0.3s ease;
  text-align: center;
}

.prayer-checkbox:hover {
  background: #e8f5e9;
  border-color: var(--secondary-green);
}

/* Prayer times */
.prayer-times {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.prayer-time {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem;
  border-radius: 6px;
  border: 2px solid var(--border-color);
  transition: all 0.3s ease;
}

.prayer-time.enabled {
  background: #ffffff;
  border-color: var(--secondary-green);
  box-shadow: 0 2px 6px rgba(76, 175, 80, 0.2);
}

.prayer-time.disabled {
  background: #f8f9fa;
  border-color: #e0e0e0;
  opacity: 0.6;
}

.prayer-time.active {
  background: linear-gradient(135deg, #ffeb3b, #ffc107);
  border-color: #ff9800;
  box-shadow: 0 4px 15px rgba(255, 193, 7, 0.4);
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0% { transform: scale(1); }
  50% { transform: scale(1.02); }
  100% { transform: scale(1); }
}

.active-indicator {
  background: #ff5722;
  color: white;
  padding: 0.1rem 0.3rem;
  border-radius: 8px;
  font-size: 0.6rem;
  font-weight: bold;
  margin-left: 0.3rem;
  animation: blink 1s infinite;
}

@keyframes blink {
  0%, 50% { opacity: 1; }
  51%, 100% { opacity: 0.5; }
}

/* Text inside prayer times */
.prayer-name {
  font-weight: 600;
  font-size: 1rem;
}

.prayer-time-value {
  font-weight: 500;
  font-size: 1.1rem;
  font-family: 'Courier New', monospace;
}

/* Alerts */
.loading,
.error {
  text-align: center;
  padding: 0.75rem;
  border-radius: 6px;
  font-size: 0.9rem;
}

.loading {
  background: #e8f5e9;
  color: var(--primary-green);
}

.error {
  background: #fdecea;
  color: #b71c1c;
  border: 1px solid #f5c6cb;
}

/* --- Responsive --- */
@media (max-width: 480px) {
  .main-content {
    padding: 0.75rem;
  }
  .prayer-checkboxes {
    gap: 0.25rem;
  }
  .logo h1 {
    font-size: 1.2rem;
  }
  .crescent {
    font-size: 1.3rem;
  }
  .tab-btn {
    padding: 0.6rem 0.8rem;
    font-size: 0.9rem;
  }
  .location-input-group {
    flex-direction: column;
  }
  .auto-detect-btn {
    width: 100%;
  }
  .settings-sub-nav {
    gap: 0.15rem;
    padding: 0.2rem;
  }
  .settings-sub-btn {
    padding: 0.4rem 0.5rem;
    font-size: 0.75rem;
  }
}

/* --- Dark Mode --- */
@media (prefers-color-scheme: dark) {
  :root {
    --primary-green: #81c784;
    --secondary-green: #43a047;
    --accent-gold: #ffd54f;
    --light-bg: #102913;
    --card-bg: #1b3a1e;
    --text-dark: #e0e0e0;
    --border-color: #3a5d40;
  }

  body {
    background: linear-gradient(135deg, #0d1f10, #183c1d);
    color: var(--text-dark);
  }

  .prayer-time.enabled {
    background: linear-gradient(135deg, #1e4620, #2e7d32);
  }

  .settings-sub-nav {
    background: #2a3a2e;
  }

  .settings-sub-btn:hover {
    background: #3a4a3e;
  }

  .settings-sub-btn.active {
    background: var(--card-bg);
  }
}
</style>