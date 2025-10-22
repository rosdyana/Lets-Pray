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
        <p class="subtitle">Prayer Reminder & Times</p>
      </div>
    </header>

    <main class="main-content">
      <div class="settings-section">
        <h2>Settings</h2>
        
        <div class="setting-group">
          <label for="location">Location:</label>
          <input 
            id="location"
            v-model="settings.location" 
            type="text" 
            placeholder="Enter your city"
            @blur="saveSettings"
          />
        </div>

        <div class="setting-group">
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
            <span v-if="soundState === 'idle'">🔊 Test Adhan Sound</span>
            <span v-else-if="soundState === 'playing'">⏹️ Stop Adhan Sound</span>
          </button>
        </div>

        <div class="setting-group">
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

        <div class="setting-group">
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

      <div class="prayer-times-section">
        <h2>Today's Prayer Times</h2>
        
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
              <span v-if="activePrayer === prayer.name" class="active-indicator">🔔 NOW</span>
            </div>
            <div class="prayer-time-value">{{ prayer.time }}</div>
          </div>
        </div>
      </div>
  </main>

    <footer class="footer">
      <p>May Allah bless our prayers and accept our worship</p>
    </footer>
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
  padding: 1.5rem;
  text-align: center;
  box-shadow: var(--shadow);
}

.header-content {
  max-width: 400px;
  margin: 0 auto;
}

.logo {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

.logo h1 {
  font-size: 1.8rem;
  font-weight: 600;
}

.crescent {
  font-size: 2rem;
  animation: glow 2s ease-in-out infinite alternate;
}

@keyframes glow {
  from { text-shadow: 0 0 5px var(--accent-gold); }
  to { text-shadow: 0 0 20px var(--accent-gold), 0 0 30px var(--accent-gold); }
}

.subtitle {
  font-size: 0.9rem;
  opacity: 0.9;
}

/* Main content */
.main-content {
  flex: 1;
  padding: 1.5rem;
  max-width: 400px;
  margin: 0 auto;
  width: 100%;
}

/* Card sections */
.settings-section,
.prayer-times-section {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 1.5rem;
  margin-bottom: 1rem;
  box-shadow: var(--shadow);
  border: 1px solid var(--border-color);
}

.settings-section h2,
.prayer-times-section h2 {
  color: var(--primary-green);
  font-size: 1.3rem;
  margin-bottom: 1rem;
  text-align: center;
  border-bottom: 2px solid var(--accent-gold);
  padding-bottom: 0.5rem;
}

/* Inputs */
.setting-group {
  margin-bottom: 1rem;
}

.setting-group label {
  display: block;
  font-weight: 500;
  margin-bottom: 0.5rem;
}

.setting-group input[type="text"] {
  width: 100%;
  padding: 0.75rem;
  border: 2px solid var(--border-color);
  border-radius: 8px;
  font-size: 1rem;
  transition: border-color 0.3s ease;
  background: #1f552d;
  color: var(--text-dark);
}

.setting-group input[type="text"]:focus {
  outline: none;
  border-color: var(--secondary-green);
  box-shadow: 0 0 0 3px rgba(76, 175, 80, 0.2);
}

/* Checkbox */
.checkbox-label {
  display: flex;
  align-items: center;
  cursor: pointer;
  font-weight: 500;
  color: var(--text-dark);
  margin-bottom: 0.5rem;
}

.checkbox-label input[type="checkbox"] {
  margin-right: 0.5rem;
  width: 18px;
  height: 18px;
  accent-color: var(--secondary-green);
}

/* Buttons */
.test-sound-btn {
  margin-top: 0.5rem;
  padding: 0.6rem 1rem;
  background: var(--accent-gold);
  color: #000;
  border: none;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
  width: 100%;
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
  gap: 0.5rem;
  margin-top: 0.5rem;
}

.prayer-checkbox {
  background: #0e0e0e;
  padding: 0.5rem;
  border-radius: 6px;
  border: 1px solid var(--border-color);
  font-weight: 600;
  transition: all 0.3s ease;
}

.prayer-checkbox:hover {
  background: #e8f5e9;
  border-color: var(--secondary-green);
}

/* Prayer times */
.prayer-times {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.prayer-time {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem;
  border-radius: 8px;
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
  padding: 0.2rem 0.5rem;
  border-radius: 12px;
  font-size: 0.7rem;
  font-weight: bold;
  margin-left: 0.5rem;
  animation: blink 1s infinite;
}

@keyframes blink {
  0%, 50% { opacity: 1; }
  51%, 100% { opacity: 0.5; }
}

/* Text inside prayer times */
.prayer-name {
  font-weight: 600;
  font-size: 1.1rem;
}

.prayer-time-value {
  font-weight: 500;
  font-size: 1.2rem;
  font-family: 'Courier New', monospace;
}

/* Alerts */
.loading,
.error {
  text-align: center;
  padding: 1rem;
  border-radius: 8px;
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

/* Footer */
.footer {
  background: var(--primary-green);
  color: var(--text-light);
  text-align: center;
  padding: 1rem;
  font-size: 0.9rem;
}

/* --- Responsive --- */
@media (max-width: 480px) {
  .main-content {
    padding: 1rem;
  }
  .settings-section,
  .prayer-times-section {
    padding: 1rem;
  }
  .prayer-checkboxes {
    grid-template-columns: 1fr;
  }
  .logo h1 {
    font-size: 1.5rem;
  }
  .crescent {
    font-size: 1.5rem;
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

  .settings-section,
  .prayer-times-section {
    background: var(--card-bg);
    border-color: var(--border-color);
  }

  .prayer-time.enabled {
    background: linear-gradient(135deg, #1e4620, #2e7d32);
  }

  .footer {
    background: #1b3a1e;
  }
}
</style>