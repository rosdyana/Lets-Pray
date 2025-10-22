use chrono::{ DateTime, Local, NaiveTime };
use chrono_tz::Tz;
use std::str::FromStr;
use reqwest;
use serde::{ Deserialize, Serialize };
use std::collections::HashMap;
use std::time::Duration;
use tauri::{ AppHandle, Emitter, WindowEvent, Manager };
// Notification functionality will be implemented using system notifications
use tokio::time::interval;
use auto_launch::AutoLaunch;
use tauri::{
    menu::{ Menu, MenuItem },
    tray::{ MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent },
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrayerTime {
    pub name: String,
    pub time: String,
    pub datetime: DateTime<Local>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrayerTimesResponse {
    pub data: PrayerData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrayerData {
    pub timings: HashMap<String, String>,
    pub date: PrayerDate,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrayerDate {
    pub readable: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub location: String,
    pub play_sound: bool,
    pub enabled_prayers: Vec<String>,
    pub run_at_startup: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            location: "New Taipei City".to_string(),
            play_sound: true,
            enabled_prayers: vec![
                "Fajr".to_string(),
                "Dhuhr".to_string(),
                "Asr".to_string(),
                "Maghrib".to_string(),
                "Isha".to_string()
            ],
            run_at_startup: false,
        }
    }
}

#[tauri::command]
async fn fetch_prayer_times(location: String) -> Result<Vec<PrayerTime>, String> {
    let today = Local::now().format("%d-%m-%Y").to_string();

    // Try to get tz for this location. Fallback to system's local timezone name.
    let timezone_string = match Tz::from_str(&location) {
        Ok(tz) => tz.name().to_string(),
        Err(_) => {
            // fallback: derive from system offset
            let offset_seconds = (*Local::now().offset()).local_minus_utc();
            match offset_seconds {
                28800 => "Asia/Taipei".to_string(), // UTC+8
                25200 => "Asia/Jakarta".to_string(), // UTC+7
                32400 => "Asia/Tokyo".to_string(), // UTC+9
                0 => "UTC".to_string(),
                _ => "UTC".to_string(),
            }
        }
    };

    let url = format!(
        "https://api.aladhan.com/v1/timingsByAddress/{}?address={}&method=3&shafaq=general&tune=5%2C3%2C5%2C7%2C9%2C-1%2C0%2C8%2C-6&timezonestring={}&calendarMethod=UAQ",
        today,
        urlencoding::encode(&location),
        urlencoding::encode(&timezone_string)
    );

    println!("URL: {}", url);

    let response = reqwest
        ::get(&url).await
        .map_err(|e| format!("Failed to fetch prayer times: {}", e))?;

    let prayer_response: PrayerTimesResponse = response
        .json().await
        .map_err(|e| format!("Failed to parse prayer times: {}", e))?;

    let mut prayer_times = Vec::new();
    let prayer_names = ["Fajr", "Dhuhr", "Asr", "Maghrib", "Isha"];

    for name in &prayer_names {
        if let Some(time_str) = prayer_response.data.timings.get(*name) {
            if let Ok(time) = NaiveTime::parse_from_str(time_str, "%H:%M") {
                let datetime = Local::now()
                    .date_naive()
                    .and_time(time)
                    .and_local_timezone(Local)
                    .unwrap();
                prayer_times.push(PrayerTime {
                    name: name.to_string(),
                    time: time_str.clone(),
                    datetime,
                });
            }
        }
    }

    Ok(prayer_times)
}

#[tauri::command]
async fn get_settings() -> Result<AppSettings, String> {
    // In a real app, you'd load from persistent storage
    // For now, return default settings
    Ok(AppSettings::default())
}

#[tauri::command]
async fn save_settings(settings: AppSettings) -> Result<(), String> {
    // In a real app, you'd save to persistent storage
    println!("Settings saved: {:?}", settings);

    // Handle auto-start setting
    let auto_launch = AutoLaunch::new("Lets Pray", "", &[] as &[&str]);

    if settings.run_at_startup {
        auto_launch.enable().map_err(|e| format!("Failed to enable auto-start: {}", e))?;
    } else {
        auto_launch.disable().map_err(|e| format!("Failed to disable auto-start: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
async fn show_notification(_title: String, _body: String) -> Result<(), String> {
    // This will be handled by the notification plugin
    Ok(())
}

#[tauri::command]
async fn test_adhan_sound() -> Result<(), String> {
    // Emit event to frontend to play test sound
    Ok(())
}

fn create_tray_icon(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Create menu items
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    // Create menu
    let menu = Menu::with_items(app, &[&settings_item, &quit_item])?;

    // Create tray icon
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "settings" => {
                    println!("Settings menu item was clicked");
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    println!("Quit menu item was clicked");
                    app.exit(0);
                }
                _ => {
                    println!("Menu item {:?} not handled", event.id);
                }
            }
        })
        .on_tray_icon_event(|tray, event| {
            match event {
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } => {
                    println!("Left click pressed and released");
                    // Show and focus the main window when tray is clicked
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                _ => {
                    println!("Unhandled tray event {event:?}");
                }
            }
        })
        .build(app)?;

    Ok(())
}
async fn check_prayer_reminders(app_handle: AppHandle) {
    let settings = get_settings().await.unwrap_or_default();

    match fetch_prayer_times(settings.location).await {
        Ok(prayer_times) => {
            let now = Local::now();

            for prayer in prayer_times {
                if settings.enabled_prayers.contains(&prayer.name) {
                    let prayer_time = prayer.datetime;
                    let time_diff = prayer_time.signed_duration_since(now);

                    // Check if prayer time is now (within the current minute)
                    if
                        time_diff.num_minutes() == 0 &&
                        time_diff.num_seconds() >= 0 &&
                        time_diff.num_seconds() < 60
                    {
                        let title = format!("Prayer Time: {}", prayer.name);
                        let body = format!(
                            "It's time for {} prayer at {}",
                            prayer.name,
                            prayer.time
                        );

                        println!("Prayer time notification: {} at {}", prayer.name, prayer.time);

                        // Show the main window
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                            let _ = window.unminimize();
                        }

                        // Send notification event to frontend
                        let _ = app_handle.emit(
                            "prayer-reminder",
                            serde_json::json!({
                             "title": title,
                             "body": body
                         })
                        );

                        // Play adhan sound if enabled
                        if settings.play_sound {
                            // Sound playing will be implemented in the frontend
                            let _ = app_handle.emit("play-adhan", ());
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch prayer times: {}", e);
        }
    }
}

fn setup_prayer_reminder_timer(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut interval = interval(Duration::from_secs(60)); // Check every minute

        loop {
            interval.tick().await;
            check_prayer_reminders(app_handle.clone()).await;
        }
    });
}

// Window event handling is now done inline in the main function

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder
        ::default()
        .plugin(tauri_plugin_opener::init())
        .on_window_event(|app, event| {
            // Handle window events for main window
            match event {
                WindowEvent::CloseRequested { api, .. } => {
                    // Hide window instead of closing - minimize to tray
                    api.prevent_close();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.hide();
                    }
                }
                _ => {}
            }
        })
        .setup(|app| {
            let app_handle = app.handle().clone();
            setup_prayer_reminder_timer(app_handle);

            // Create system tray icon
            if let Err(e) = create_tray_icon(app) {
                eprintln!("Failed to create tray icon: {}", e);
            }

            Ok(())
        })
        .invoke_handler(
            tauri::generate_handler![
                fetch_prayer_times,
                get_settings,
                save_settings,
                show_notification,
                test_adhan_sound
            ]
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
