# 🌙 Lets Pray - Prayer Reminder App

A beautiful, simple prayer reminder application built with Tauri 2.0 and Vue 3. Get timely notifications for your daily prayers with a clean, Muslim-themed interface.

## ✨ Features

- **🕐 Daily Prayer Times**: Automatically fetches prayer times from Aladhan API based on your location
- **🔔 Smart Reminders**: Get notified 5 minutes before each prayer time
- **🔊 Adhan Sound**: Optional audio notifications for prayer times
- **⚙️ Customizable Settings**: Choose which prayers to be reminded of
- **🚀 Auto-Start**: Option to run the app at system startup
- **🎨 Beautiful UI**: Clean, Muslim-themed interface with green and gold colors
- **💻 System Tray**: Runs quietly in the background with tray icon access

## 🚀 Getting Started

### Prerequisites

- Node.js (v16 or higher)
- Rust (latest stable version)
- pnpm package manager

### Installation

1. **Clone the repository**
```bash
git clone <your-repo-url>
cd lets-pray
```

2. **Install dependencies**
```bash
pnpm install
```

3. **Run in development mode**
```bash
pnpm tauri dev
```

4. **Build the application**
```bash
pnpm tauri build
```

## 📱 How to Use

### First Time Setup

1. **Launch the app** - It will appear in your system tray
2. **Click the tray icon** to open settings
3. **Set your location** - Enter your city name for accurate prayer times
4. **Configure reminders** - Choose which prayers you want to be reminded of:
   - Fajr (Dawn)
   - Dhuhr (Midday)
   - Asr (Afternoon)
   - Maghrib (Sunset)
   - Isha (Night)
5. **Enable/disable Adhan sound** - Toggle audio notifications
6. **Set auto-start** - Choose whether to run at system startup

### Daily Usage

- The app runs quietly in the background
- Click the tray icon anytime to view today's prayer times and adjust settings
- The app automatically fetches new prayer times daily

## 🛠️ Technical Details

### Architecture

- **Frontend**: Vue 3 + TypeScript + Vite
- **Backend**: Rust with Tauri 2.0
- **API**: Aladhan Prayer Times API
- **Styling**: Custom CSS with Muslim-themed design

### Key Components

- **Prayer Times API Integration**: Fetches daily prayer times based on location
- **Notification System**: System notifications 5 minutes before prayer times
- **Settings Management**: Persistent storage of user preferences
- **Auto-Start Functionality**: System startup integration
- **Audio System**: Adhan sound playback (requires audio file)

### API Configuration

The app uses the Aladhan API with the following parameters:
- **Method**: 3 (University of Islamic Sciences, Karachi)
- **Timezone**: Asia/Taipei (configurable)
- **Calculation**: UAQ (Umm al-Qura, Makkah)

## 🎨 Customization

### Adding Adhan Audio

1. Place your adhan audio file in the `public/` directory
2. Name it `adhan.mp3`
3. The app will automatically use it for audio notifications

### Styling

The app uses CSS custom properties for easy theming:
- `--primary-green`: Main green color (#2d5016)
- `--secondary-green`: Secondary green (#4a7c59)
- `--accent-gold`: Gold accent color (#d4af37)

## 📦 Building for Production

### Windows
```bash
pnpm tauri build
```
This creates both MSI and NSIS installers in `src-tauri/target/release/bundle/`

### Cross-Platform
The app supports Windows, macOS, and Linux. Use the appropriate build commands for your target platform.

## 🔧 Development

### Project Structure
```
lets-pray/
├── src/                    # Vue frontend
│   ├── App.vue            # Main application component
│   └── main.ts            # Application entry point
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── lib.rs         # Main Rust logic
│   │   └── main.rs        # Entry point
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── public/                # Static assets
└── package.json           # Node.js dependencies
```

### Key Rust Functions

- `fetch_prayer_times()`: Retrieves prayer times from API
- `check_prayer_reminders()`: Monitors for upcoming prayer times
- `save_settings()`: Persists user preferences
- `setup_prayer_reminder_timer()`: Background timer for reminders

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- **Aladhan API** for providing accurate prayer times
- **Tauri** for the excellent desktop app framework
- **Vue.js** for the reactive frontend framework
- **Islamic community** for inspiration and feedback

## 📞 Support

If you encounter any issues or have suggestions, please:
1. Check the existing issues on GitHub
2. Create a new issue with detailed information
3. Include your operating system and app version

---

**May Allah bless your prayers and accept your worship. Ameen.** 🤲